use crate::strava::archive::{StravaArchive, StravaFileType};
use crate::strava::csv::CsvHandler;
use crate::utils::gpx_metadata::add_metadata_to_gpx;
use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

/// Statistics for Strava conversion operations
#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    pub total: usize,
    pub converted: usize,
    pub failed: usize,
    pub matched: usize,
}

/// Converter for processing Strava export archives
///
/// Handles extraction of .fit.gz files, conversion to GPX, and metadata injection
/// from activities.csv.
pub struct StravaConverter {
    input_path: PathBuf,
    csv_handler: Option<CsvHandler>,
    verbose: bool,
    archive: StravaArchive,
}

impl StravaConverter {
    /// Create a new StravaConverter
    ///
    /// # Arguments
    /// * `dir_in` - Path to Strava export ZIP or extracted directory
    ///
    /// # Example
    /// ```no_run
    /// use fit2gpx_lightning::StravaConverter;
    ///
    /// let converter = StravaConverter::new("strava_export.zip");
    /// ```
    pub fn new(dir_in: impl AsRef<Path>) -> Self {
        Self {
            input_path: dir_in.as_ref().to_path_buf(),
            csv_handler: None,
            verbose: false,
            archive: StravaArchive::new(dir_in.as_ref()).unwrap(),
        }
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Convert all FIT files to GPX
    ///
    /// Processes .fit.gz files from the Strava archive, decompresses them,
    /// converts to GPX, and saves to the output directory.
    ///
    /// # Arguments
    /// * `output_dir` - Directory where GPX files will be written
    ///
    /// # Returns
    /// Conversion statistics
    pub fn strava_fit_to_gpx(&mut self, output_dir: impl AsRef<Path>) -> Result<ConversionStats> {
        let output_path = output_dir.as_ref();
        fs::create_dir_all(output_path)?;

        // Extract files from archive if it's a ZIP
        let files = &self.archive.files;

        let total = files.len();

        let stats = Arc::new(Mutex::new(ConversionStats {
            total,
            ..Default::default()
        }));

        // Parallel conversion
        files.par_iter().for_each(|file_info| {
            // Handle fit, fit.gz, and gpx differently
            let fit_data = match file_info.file_type {
                StravaFileType::FitGz => {
                    // Decompress .fit.gz to FIT bytes
                    let fit_data = match file_info.decompress_gzip() {
                        Ok(data) => data,
                        Err(e) => {
                            stats.lock().unwrap().failed += 1;
                            if self.verbose {
                                eprintln!("Failed to decompress {}: {}", file_info.filename, e);
                            }
                            return;
                        }
                    };
                    fit_data
                }
                StravaFileType::GpxGz => {
                    // Decompress .gpx.gz to GPX bytes
                    let gpx_data = match file_info.decompress_gzip() {
                        Ok(data) => data,
                        Err(e) => {
                            stats.lock().unwrap().failed += 1;
                            if self.verbose {
                                eprintln!("Failed to decompress {}: {}", file_info.filename, e);
                            }
                            return;
                        }
                    };
                    gpx_data
                }
                StravaFileType::Fit | StravaFileType::Gpx => {
                    // Use FIT data directly
                    file_info.data.clone()
                }
                _ => {
                    stats.lock().unwrap().failed += 1;
                    if self.verbose {
                        eprintln!("Unknown file type: {}", file_info.filename);
                    }
                    return;
                }
            };

            // Extract activity ID from filename (e.g., "1234567890.fit.gz" -> "1234567890")
            let activity_id = match file_info.file_type {
                StravaFileType::Gpx => file_info.filename.trim_end_matches(".gpx").to_string(),
                StravaFileType::GpxGz => file_info.filename.trim_end_matches(".gpx.gz").to_string(),
                StravaFileType::Fit => file_info.filename.trim_end_matches(".fit").to_string(),
                StravaFileType::FitGz => file_info.filename.trim_end_matches(".fit.gz").to_string(),
                _ => {
                    stats.lock().unwrap().failed += 1;
                    if self.verbose {
                        eprintln!("Cannot extract activity ID from: {}", file_info.filename);
                    }
                    return;
                }
            };

            // Create temp file for data
            let mut temp_fit = match NamedTempFile::new() {
                Ok(f) => f,
                Err(e) => {
                    stats.lock().unwrap().failed += 1;
                    if self.verbose {
                        eprintln!(
                            "Failed to create temp file for {}: {}",
                            file_info.filename, e
                        );
                    }
                    return;
                }
            };

            if let Err(e) = temp_fit.write_all(&fit_data) {
                stats.lock().unwrap().failed += 1;
                if self.verbose {
                    eprintln!(
                        "Failed to write temp file for {}: {}",
                        file_info.filename, e
                    );
                }
                return;
            }

            // Convert FIT or FIT GZ to GPX, copy GPX directly
            let output_gpx = output_path.join(format!("{}.gpx", activity_id));

            if file_info.file_type == StravaFileType::Gpx
                || file_info.file_type == StravaFileType::GpxGz
            {
                // Directly write GPX data
                match fs::write(&output_gpx, &fit_data) {
                    Ok(_) => {
                        stats.lock().unwrap().converted += 1;
                        if self.verbose {
                            println!("Copied GPX: {}", activity_id);
                        }
                    }
                    Err(e) => {
                        stats.lock().unwrap().failed += 1;
                        if self.verbose {
                            eprintln!("Failed to write GPX {}: {}", activity_id, e);
                        }
                    }
                }
                return;
            }

            // Convert FIT or FIT GZ to GPX

            match fit2gpx::Fit::file_to_gpx(temp_fit.path(), &output_gpx) {
                Ok(_) => {
                    stats.lock().unwrap().converted += 1;
                    if self.verbose {
                        println!("Converted: {}", activity_id);
                    }
                }
                Err(e) => {
                    stats.lock().unwrap().failed += 1;
                    if self.verbose {
                        eprintln!("Failed to convert {}: {}", file_info.filename, e);
                    }
                }
            }
        });

        let final_stats = stats.lock().unwrap().clone();
        Ok(final_stats)
    }

    /// Add metadata from activities.csv to GPX files
    ///
    /// Injects activity name and type into GPX files based on activities.csv data.
    ///
    /// # Arguments
    /// * `gpx_dir` - Directory containing GPX files to update
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn add_metadata_to_gpx(&mut self, gpx_dir: impl AsRef<Path>) -> Result<()> {
        let gpx_path = gpx_dir.as_ref();

        // Load CSV if not already loaded
        if self.csv_handler.is_none() {
            self.csv_handler = Some(
                if self.input_path.extension().and_then(|s| s.to_str()) == Some("zip") {
                    CsvHandler::extract_from_archive(&self.input_path)?
                } else {
                    let csv_file = self.input_path.join("activities.csv");
                    CsvHandler::from_file(&csv_file)?
                },
            );
        }

        let csv_handler = self.csv_handler.as_ref().unwrap();

        // Find all GPX files
        let gpx_files: Vec<_> = fs::read_dir(gpx_path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("gpx"))
            .collect();

        let mut matched = 0;
        let mut failed = 0;

        for entry in gpx_files {
            let gpx_file_path = entry.path();

            // Extract activity ID from filename (e.g., "1234567890.gpx" -> "1234567890")
            let activity_id = gpx_file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            // Find activity in CSV
            if let Some(activity) = csv_handler.find_by_id(activity_id) {
                match add_metadata_to_gpx(
                    &gpx_file_path,
                    Some(&activity.activity_name),
                    Some(&activity.activity_type),
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
                eprintln!("No CSV entry found for: {}", activity_id);
            }
        }

        if self.verbose {
            println!("Metadata: {} matched, {} failed", matched, failed);
        }

        Ok(())
    }
}
