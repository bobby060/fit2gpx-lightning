use crate::garmin::archive::{extract_all_timestamps, extract_fit_files};
use crate::garmin::metadata::{GarminActivity, linear_activity_matching, read_garmin_activities};
use crate::utils::gpx_metadata::add_metadata_to_gpx;
use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

/// Statistics for Garmin conversion operations
#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    pub total: usize,
    pub converted: usize,
    pub failed: usize,
    pub matched: usize,
    pub unmatched: usize,
}

/// Converter for processing Garmin export archives
///
/// Handles extraction of FIT files from nested ZIPs, conversion to GPX,
/// and metadata injection from summarizedActivities.json.
pub struct GarminConverter {
    input_path: PathBuf,
    activities: Vec<GarminActivity<'static>>,
    verbose: bool,
}

impl GarminConverter {
    /// Create a new GarminConverter
    ///
    /// # Arguments
    /// * `dir_in` - Path to Garmin export ZIP
    ///
    /// # Example
    /// ```no_run
    /// use fit2gpx_lightning::GarminConverter;
    ///
    /// let converter = GarminConverter::new("garmin_export.zip");
    /// ```
    pub fn new(dir_in: impl AsRef<Path>) -> Self {
        Self {
            input_path: dir_in.as_ref().to_path_buf(),
            activities: Vec::new(),
            verbose: false,
        }
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Convert all FIT files to GPX with metadata matching
    ///
    /// Processes FIT files from the Garmin archive, converts to GPX,
    /// matches with activities from summarizedActivities.json by timestamp.
    /// Uses activity id from summarizedActivities.json as file name
    ///
    /// # Arguments
    /// * `output_dir` - Directory where GPX files will be written
    ///
    /// # Returns
    /// Conversion statistics
    pub fn garmin_fit_to_gpx(&mut self, output_dir: impl AsRef<Path>) -> Result<ConversionStats> {
        let output_path = output_dir.as_ref();
        fs::create_dir_all(output_path)?;

        // Load activities metadata if not already loaded
        if self.activities.is_empty() {
            if self.verbose {
                println!("Loading activity metadata from summarizedActivities.json...");
            }

            let activities = read_garmin_activities(&self.input_path)?;
            if self.verbose {
                println!("Loaded {} activities", activities.len());
            }
            self.activities = activities;
        }

        // Extract FIT files from nested archives
        if self.verbose {
            println!("Extracting FIT files...");
        }

        let mut fit_files = extract_fit_files(&self.input_path)?;

        if self.verbose {
            println!(
                "Found {} FIT files, extracting timestamps...",
                fit_files.len()
            );
        }

        // Extract timestamps from FIT files
        extract_all_timestamps(&mut fit_files);

        let total = fit_files.len();

        let stats = Arc::new(Mutex::new(ConversionStats {
            total,
            ..Default::default()
        }));

        let matches = linear_activity_matching(&self.activities, &fit_files);

        stats.lock().unwrap().matched = matches.len();
        stats.lock().unwrap().unmatched = total - matches.len();

        matches.par_iter().for_each(|(activity_idx, fit_file_idx)| {
            let activity = &self.activities[*activity_idx];
            let fit_file = &fit_files[*fit_file_idx];
            let activity_id = activity.activity_id.unwrap_or(0);

            // Write FIT data to temp file
            let mut temp_fit = match NamedTempFile::new() {
                Ok(f) => f,
                Err(e) => {
                    stats.lock().unwrap().failed += 1;
                    if self.verbose {
                        eprintln!(
                            "Failed to create temp file for {}: {}",
                            fit_file.filename, e
                        );
                    }
                    return;
                }
            };

            if let Err(e) = temp_fit.write_all(&fit_file.data) {
                stats.lock().unwrap().failed += 1;
                if self.verbose {
                    eprintln!("Failed to write temp file for {}: {}", fit_file.filename, e);
                }
                return;
            }

            // Convert FIT to GPX
            let output_gpx = output_path.join(format!("{}.gpx", activity_id));

            match fit2gpx::Fit::file_to_gpx(temp_fit.path(), &output_gpx) {
                Ok(_) => {
                    stats.lock().unwrap().converted += 1;
                }
                Err(e) => {
                    stats.lock().unwrap().failed += 1;
                    if self.verbose {
                        eprintln!("Failed to convert {}: {}", fit_file.filename, e);
                    }
                }
            }
        });

        let final_stats = stats.lock().unwrap().clone();

        if self.verbose {
            println!(
                "Conversion complete: {} converted, {} matched, {} unmatched, {} failed",
                final_stats.converted,
                final_stats.matched,
                final_stats.unmatched,
                final_stats.failed
            );
        }

        Ok(final_stats)
    }

    /// Add metadata to already-converted GPX files
    ///
    /// Matches GPX files by activity ID and injects metadata from
    /// summarizedActivities.json.
    ///
    /// # Arguments
    /// * `gpx_dir` - Directory containing GPX files to update
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn add_metadata_to_gpx(&mut self, gpx_dir: impl AsRef<Path>) -> Result<()> {
        let gpx_path = gpx_dir.as_ref();

        // Load activities if not already loaded
        if self.activities.is_empty() {
            if self.verbose {
                println!("Loading activity metadata...");
            }

            self.activities = read_garmin_activities(&self.input_path)?;
        }

        let activities = &self.activities;

        // Build lookup by activity ID
        let activity_map: std::collections::HashMap<_, _> = activities
            .iter()
            .filter_map(|a| a.activity_id.map(|id| (id.to_string(), a)))
            .collect();

        // Find all GPX files
        let gpx_files: Vec<_> = fs::read_dir(gpx_path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("gpx"))
            .collect();

        let mut matched = 0;
        let mut failed = 0;

        for entry in gpx_files {
            let gpx_file_path = entry.path();

            // Extract activity ID from filename
            let activity_id = gpx_file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            // Find activity in metadata
            if let Some(&activity) = activity_map.get(activity_id) {
                match add_metadata_to_gpx(
                    &gpx_file_path,
                    Some(&activity.name()),
                    Some(&activity.type_string()),
                ) {
                    Ok(_) => {
                        matched += 1;
                        if self.verbose {
                            println!("Added metadata to: {}", activity_id);
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        if self.verbose {
                            eprintln!("Failed to add metadata to {}: {}", activity_id, e);
                        }
                    }
                }
            } else if self.verbose {
                eprintln!("No metadata found for: {}", activity_id);
            }
        }

        if self.verbose {
            println!("Metadata: {} matched, {} failed", matched, failed);
        }

        Ok(())
    }
}
