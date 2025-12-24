use thiserror::Error;

/// Custom error types for fit2gpx-lightning
#[derive(Error, Debug)]
pub enum Fit2GpxError {
    /// Input file not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid FIT file format
    #[error("Invalid FIT file: {0}")]
    InvalidFit(String),

    /// FIT to GPX conversion failed
    #[error("Conversion failed: {0}")]
    ConversionFailed(String),

    /// Archive processing error
    #[error("Archive error: {0}")]
    ArchiveError(String),

    /// GPX metadata error
    #[error("GPX metadata error: {0}")]
    GpxError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP error
    #[error("ZIP error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    /// CSV error
    #[error("CSV parsing error: {0}")]
    CsvParseError(#[from] csv::Error),

    /// JSON error
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

impl Fit2GpxError {
    /// Create a file not found error
    pub fn file_not_found(path: impl std::fmt::Display) -> Self {
        Self::FileNotFound(path.to_string())
    }

    /// Create a conversion failed error
    pub fn conversion_failed(msg: impl std::fmt::Display) -> Self {
        Self::ConversionFailed(msg.to_string())
    }

    /// Create an archive error
    pub fn archive_error(msg: impl std::fmt::Display) -> Self {
        Self::ArchiveError(msg.to_string())
    }
}

/// Result type alias for fit2gpx-lightning
pub type Result<T> = std::result::Result<T, Fit2GpxError>;
