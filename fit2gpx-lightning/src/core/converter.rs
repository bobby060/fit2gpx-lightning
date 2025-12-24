use crate::core::error::{Fit2GpxError, Result};
use rayon::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Statistics for bulk conversion operations
#[derive(Debug, Clone, Default)]
pub struct BulkStats {
    /// Total number of FIT files found
    pub total: usize,
    /// Number of files successfully converted
    pub converted: usize,
    /// Number of files that failed to convert
    pub failed: usize,
}

/// Convert a single FIT file to GPX
///
/// This function takes a FIT file as input and converts it to GPX format using
/// the fit2gpx library. The output directory will be created if it doesn't exist.
///
/// # Arguments
/// * `f_in` - Path to the input FIT file
/// * `f_out` - Path where the output GPX file should be written
///
/// # Returns
/// * `Ok(())` if conversion succeeds
/// * `Err(Fit2GpxError)` if the file doesn't exist or conversion fails
///
/// # Example
/// ```no_run
/// use fit2gpx_lightning::fit_to_gpx;
///
/// fit_to_gpx("activity.fit", "activity.gpx").unwrap();
/// ```
pub fn fit_to_gpx(f_in: impl AsRef<Path>, f_out: impl AsRef<Path>) -> Result<()> {
    let input = f_in.as_ref();
    let output = f_out.as_ref();

    // Validate input exists
    if !input.exists() {
        return Err(Fit2GpxError::file_not_found(input.display()));
    }

    if !input.is_file() {
        return Err(Fit2GpxError::InvalidFit(format!(
            "{} is not a file",
            input.display()
        )));
    }

    // Create parent directory for output if needed
    if let Some(parent) = output.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    // Convert using fit2gpx library
    fit2gpx::Fit::file_to_gpx(input, output).map_err(|e| {
        Fit2GpxError::conversion_failed(format!("Failed to convert {}: {}", input.display(), e))
    })?;

    Ok(())
}

/// Convert all FIT files in a directory to GPX format
///
/// This function scans a directory for FIT files (immediate children only, no recursion)
/// and converts each one to GPX format in the output directory. The conversion is done
/// in parallel using all available CPU cores for maximum performance.
///
/// Files are converted with the same base name but with a .gpx extension.
/// For example, "activity.fit" becomes "activity.gpx".
///
/// # Arguments
/// * `dir_in` - Input directory containing .fit files
/// * `dir_out` - Output directory where .gpx files will be written
///
/// # Returns
/// * `Ok(BulkStats)` with conversion statistics (total, converted, failed)
/// * `Err(Fit2GpxError)` if directory operations fail
///
/// # Example
/// ```no_run
/// use fit2gpx_lightning::fit_to_gpx_bulk;
///
/// let stats = fit_to_gpx_bulk("./activities/", "./gpx_output/").unwrap();
/// println!("Converted {} out of {} files", stats.converted, stats.total);
/// ```
pub fn fit_to_gpx_bulk(dir_in: impl AsRef<Path>, dir_out: impl AsRef<Path>) -> Result<BulkStats> {
    let input_dir = dir_in.as_ref();
    let output_dir = dir_out.as_ref();

    // Validate input directory exists
    if !input_dir.exists() {
        return Err(Fit2GpxError::file_not_found(input_dir.display()));
    }

    if !input_dir.is_dir() {
        return Err(Fit2GpxError::InvalidFit(format!(
            "{} is not a directory",
            input_dir.display()
        )));
    }

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Collect FIT files (flat scan only - immediate children)
    let fit_files: Vec<_> = std::fs::read_dir(input_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_file()
                && path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("fit"))
                    .unwrap_or(false)
        })
        .collect();

    let total = fit_files.len();

    // Initialize stats with thread-safe counters
    let stats = Arc::new(Mutex::new(BulkStats {
        total,
        converted: 0,
        failed: 0,
    }));

    // Parallel processing with rayon
    fit_files.par_iter().for_each(|entry| {
        let input_path = entry.path();

        // Determine output filename (same name, .gpx extension)
        let output_filename = match input_path.file_stem() {
            Some(stem) => format!("{}.gpx", stem.to_string_lossy()),
            None => {
                eprintln!(
                    "Warning: Could not determine filename for {}",
                    input_path.display()
                );
                stats.lock().unwrap().failed += 1;
                return;
            }
        };

        let output_path = output_dir.join(output_filename);

        // Convert the file
        match fit_to_gpx(&input_path, &output_path) {
            Ok(_) => {
                stats.lock().unwrap().converted += 1;
            }
            Err(e) => {
                stats.lock().unwrap().failed += 1;
                eprintln!("Failed to convert {}: {}", input_path.display(), e);
            }
        }
    });

    // Extract final stats
    let final_stats = stats.lock().unwrap().clone();

    Ok(final_stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_fit_to_gpx_file_not_found() {
        let result = fit_to_gpx("nonexistent.fit", "output.gpx");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Fit2GpxError::FileNotFound(_)));
    }

    #[test]
    fn test_fit_to_gpx_bulk_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        let stats = fit_to_gpx_bulk(temp_dir.path(), output_dir.path()).unwrap();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.converted, 0);
        assert_eq!(stats.failed, 0);
    }

    #[test]
    fn test_fit_to_gpx_bulk_directory_not_found() {
        let output_dir = TempDir::new().unwrap();
        let result = fit_to_gpx_bulk("/nonexistent/directory", output_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Fit2GpxError::FileNotFound(_)));
    }
}
