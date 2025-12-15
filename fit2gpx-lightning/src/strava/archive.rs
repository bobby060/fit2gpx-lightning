use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// Type of Strava activity file
#[derive(Debug, Clone, PartialEq)]
pub enum StravaFileType {
    /// Gzipped FIT file (.fit.gz)
    FitGz,
    /// Pre-converted GPX file
    Gpx,
    /// Gzipped GPX file (.gpx.gz)
    GpxGz,
    /// Training Center XML (not currently converted)
    Tcx,
    /// Unzipped fit
    Fit,
}

/// Information about a Strava activity file extracted from the archive
#[derive(Debug, Clone)]
pub struct StravaFile {
    /// Base filename (e.g., "12345678.fit.gz")
    pub filename: String,
    /// Full path in ZIP (e.g., "activities/12345678.fit.gz")
    pub zip_path: String,
    /// Type of file
    pub file_type: StravaFileType,
    /// File contents
    pub data: Vec<u8>,
}

impl StravaFile {
    /// Unzips Gzip compressed data
    ///
    /// Returns new StravaFile with decompressed data
    ///
    /// # Arguments
    /// * `compressed_data` - Gzip-compressed bytes
    ///
    /// # Returns
    /// * `Vec<u8>` - Decompressed bytes
    pub fn decompress_gzip(&self) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(&self.data[..]);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .context("Failed to decompress gzip data")?;

        Ok(decompressed)
    }
}

// Represents a strava archive with its activity files
#[derive(Debug, Clone)]
pub struct StravaArchive {
    pub files: Vec<StravaFile>,
    pub path: String,
}

impl StravaArchive {
    /// Create a new StravaArchive by extracting files from the given ZIP path
    pub fn new(archive_path: &Path) -> Result<Self> {
        let files = extract_strava_files(archive_path)?;

        Ok(StravaArchive {
            files,
            path: archive_path.to_string_lossy().to_string(),
        })
    }
}

/// Extract all activity files from a Strava export archive
///
/// Scans the archive for files in the `activities/` folder and extracts
/// .fit.gz, .gpx, and .tcx files.
///
/// # Arguments
/// * `archive_path` - Path to the Strava export ZIP file
///
/// # Returns
/// * `Vec<StravaFileInfo>` - List of extracted activity files with metadata
fn extract_strava_files(archive_path: &Path) -> Result<Vec<StravaFile>> {
    let file = std::fs::File::open(archive_path).context(format!(
        "Failed to open archive: {}",
        archive_path.display()
    ))?;
    let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

    let mut files = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Only process files in activities/ folder (not directories)
        if !name.starts_with("activities/") || name.ends_with('/') {
            continue;
        }

        // Extract base filename
        let filename = name.rsplit('/').next().unwrap_or(&name).to_string();

        // Determine file type by extension
        let file_type = if filename.ends_with(".fit.gz") {
            StravaFileType::FitGz
        } else if filename.ends_with(".gpx") {
            StravaFileType::Gpx
        } else if filename.ends_with(".tcx") {
            StravaFileType::Tcx
        } else if filename.ends_with(".fit") {
            StravaFileType::Fit
        } else if filename.ends_with(".gpx.gz") {
            StravaFileType::GpxGz
        } else {
            // Skip unknown file types
            continue;
        };

        // Read file data into memory
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .context(format!("Failed to read file from archive: {}", name))?;

        files.push(StravaFile {
            filename,
            zip_path: name,
            file_type,
            data,
        });
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_detection() {
        // Test that we correctly identify file types
        let test_cases = vec![
            ("12345.fit.gz", StravaFileType::FitGz),
            ("activity.gpx", StravaFileType::Gpx),
            ("workout.tcx", StravaFileType::Tcx),
            ("session.fit", StravaFileType::Fit),
            ("route.gpx.gz", StravaFileType::GpxGz),
        ];

        for (filename, expected_type) in test_cases {
            let actual_type = if filename.ends_with(".fit.gz") {
                StravaFileType::FitGz
            } else if filename.ends_with(".gpx") {
                StravaFileType::Gpx
            } else if filename.ends_with(".tcx") {
                StravaFileType::Tcx
            } else if filename.ends_with(".fit") {
                StravaFileType::Fit
            } else if filename.ends_with(".gpx.gz") {
                StravaFileType::GpxGz
            } else {
                panic!("Unknown file type: {}", filename);
            };

            assert_eq!(actual_type, expected_type, "Failed for {}", filename);
        }
    }

    #[test]
    fn test_decompress_gzip() {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        // Create test data
        let test_data = b"Hello, this is test data for gzip compression!";

        // Compress it
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(test_data).unwrap();
        let compressed = encoder.finish().unwrap();

        let strava_file = StravaFile {
            filename: "test.fit.gz".to_string(),
            zip_path: "activities/test.fit.gz".to_string(),
            file_type: StravaFileType::FitGz,
            data: compressed.clone(),
        };

        // Decompress it using our function
        let decompressed = strava_file.decompress_gzip().unwrap();

        assert_eq!(decompressed, test_data);
    }
}
