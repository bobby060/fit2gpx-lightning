pub mod core;
pub mod garmin;
pub mod strava;
pub mod utils;

#[cfg(feature = "python")]
pub mod python;

// Re-export main types and functions
pub use core::error::Fit2GpxError;
pub use core::{BulkStats, fit_to_gpx, fit_to_gpx_bulk};
pub use garmin::GarminConverter;
pub use strava::StravaConverter;
