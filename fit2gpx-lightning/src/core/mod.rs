pub mod converter;
pub mod error;

pub use converter::{fit_to_gpx, fit_to_gpx_bulk, BulkStats};
pub use error::Fit2GpxError;
