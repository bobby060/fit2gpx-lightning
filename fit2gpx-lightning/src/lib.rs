pub mod core;
pub mod strava;
pub mod garmin;
pub mod utils;

#[cfg(feature = "python")]
pub mod python;

// Re-export main types and functions
pub use core::{fit_to_gpx, fit_to_gpx_bulk, BulkStats};
pub use core::error::Fit2GpxError;
pub use strava::StravaConverter;
pub use garmin::GarminConverter;
