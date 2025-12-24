pub mod converter;
pub mod error;

pub use converter::{BulkStats, fit_to_gpx, fit_to_gpx_bulk};
pub use error::Fit2GpxError;
