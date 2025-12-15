use anyhow::{Context, Result};
use fitparser::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

/// Information about a FIT file extracted from Garmin archive
#[derive(Debug, Clone)]
pub struct FitFileInfo {
    /// Original filename from nested archive
    pub filename: String,
    /// FIT file data
    pub data: Vec<u8>,
    /// Extracted timestamp (seconds since epoch), if available
    pub timestamp: Option<f64>,
}

/// Extract all FIT files from Garmin export archive
///
/// Garmin exports contain nested ZIP files in the structure:
/// DI_CONNECT/DI-Connect-Uploaded-Files/*.zip
///
/// Each nested ZIP contains individual FIT files.
///
/// # Arguments
/// * `archive_path` - Path to the main Garmin export ZIP file
///
/// # Returns
/// * Vector of FitFileInfo structures with file data and metadata
pub fn extract_fit_files(archive_path: &Path) -> Result<Vec<FitFileInfo>> {
    let file = File::open(archive_path).context(format!(
        "Failed to open archive: {}",
        archive_path.display()
    ))?;

    let mut archive =
        ZipArchive::new(BufReader::new(file)).context("Failed to read ZIP archive")?;

    let mut fit_files = Vec::new();

    // Scan main archive for nested ZIP files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Look for nested ZIP files in UploadedFiles directory
        if name.contains("UploadedFiles") && name.ends_with(".zip") {
            // Read nested ZIP into memory
            let mut nested_data = Vec::new();
            file.read_to_end(&mut nested_data)
                .context(format!("Failed to read nested ZIP: {}", name))?;

            drop(file);

            // Open nested ZIP archive
            let cursor = std::io::Cursor::new(nested_data);
            let mut nested_archive =
                ZipArchive::new(cursor).context(format!("Failed to open nested ZIP: {}", name))?;

            // Extract FIT files from nested archive
            for j in 0..nested_archive.len() {
                let mut nested_file = nested_archive.by_index(j)?;
                let nested_name = nested_file.name().to_string();

                if nested_name.ends_with(".fit") {
                    let mut fit_data = Vec::new();
                    nested_file
                        .read_to_end(&mut fit_data)
                        .context(format!("Failed to read FIT file: {}", nested_name))?;

                    fit_files.push(FitFileInfo {
                        filename: nested_name.clone(),
                        data: fit_data,
                        timestamp: None,
                    });
                }
            }
        }
    }

    Ok(fit_files)
}

/// Extract timestamp from FIT file data
///
/// Parses the FIT file and extracts the timestamp from the first Record message.
/// Timestamps are returned in seconds since epoch.
///
/// # Arguments
/// * `fit_data` - Raw FIT file data
///
/// # Returns
/// * Some(timestamp) if found, None if parsing fails or no timestamp available
pub fn extract_fit_timestamp(fit_data: &[u8]) -> Option<f64> {
    let mut cursor = std::io::Cursor::new(fit_data);

    match fitparser::from_reader(&mut cursor) {
        Ok(fit_records) => {
            // Look for timestamp in Record messages
            for record in fit_records.iter() {
                if record.kind() == fitparser::profile::MesgNum::Record {
                    for field in record.fields() {
                        if field.name() == "timestamp"
                            && let Value::Timestamp(dt) = field.value()
                        {
                            return Some(dt.timestamp() as f64);
                        }
                    }
                }
            }

            // If not in Record, try Session message
            for record in fit_records.iter() {
                if record.kind() == fitparser::profile::MesgNum::Session {
                    for field in record.fields() {
                        if (field.name() == "start_time" || field.name() == "timestamp")
                            && let Value::Timestamp(dt) = field.value()
                        {
                            return Some(dt.timestamp() as f64);
                        }
                    }
                }
            }

            None
        }
        Err(_) => None,
    }
}

/// Extract timestamps for all FIT files
///
/// Updates the FitFileInfo structures with extracted timestamps.
///
/// # Arguments
/// * `fit_files` - Mutable vector of FitFileInfo
pub fn extract_all_timestamps(fit_files: &mut [FitFileInfo]) {
    for fit_file in fit_files.iter_mut() {
        fit_file.timestamp = extract_fit_timestamp(&fit_file.data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_file_info_creation() {
        let info = FitFileInfo {
            filename: "test.fit".to_string(),
            data: vec![0, 1, 2, 3],
            timestamp: Some(1609459200.0),
        };

        assert_eq!(info.filename, "test.fit");
        assert_eq!(info.data.len(), 4);
        assert_eq!(info.timestamp, Some(1609459200.0));
    }
}
