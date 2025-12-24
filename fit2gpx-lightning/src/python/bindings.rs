use pyo3::exceptions::{PyFileNotFoundError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::PathBuf;

/// Simple FIT to GPX conversion
///
/// Args:
///     f_in (str): Path to input FIT file
///     f_out (str): Path to output GPX file
///
/// Raises:
///     FileNotFoundError: If input file doesn't exist
///     RuntimeError: If conversion fails
///
/// Example:
///     >>> from fit2gpx_lightning import fit_to_gpx
///     >>> fit_to_gpx("activity.fit", "activity.gpx")
#[pyfunction]
#[pyo3(signature = (f_in, f_out))]
fn fit_to_gpx(f_in: String, f_out: String) -> PyResult<()> {
    crate::core::fit_to_gpx(&f_in, &f_out)
        .map_err(|e| PyRuntimeError::new_err(format!("Conversion failed: {}", e)))
}

/// Bulk convert directory of FIT files to GPX
///
/// Scans the input directory for .fit files (immediate children only) and
/// converts each to GPX format in the output directory.
///
/// Args:
///     dir_in (str): Input directory containing .fit files
///     dir_out (str): Output directory for .gpx files
///
/// Returns:
///     dict: Statistics with keys 'total', 'converted', 'failed'
///
/// Example:
///     >>> from fit2gpx_lightning import fit_to_gpx_bulk
///     >>> stats = fit_to_gpx_bulk("./activities/", "./gpx_output/")
///     >>> print(f"Converted {stats['converted']} out of {stats['total']} files")
#[pyfunction]
#[pyo3(signature = (dir_in, dir_out))]
fn fit_to_gpx_bulk(py: Python, dir_in: String, dir_out: String) -> PyResult<PyObject> {
    let stats = crate::core::fit_to_gpx_bulk(&dir_in, &dir_out)
        .map_err(|e| PyRuntimeError::new_err(format!("Bulk conversion failed: {}", e)))?;

    // Convert to Python dict
    let dict = PyDict::new_bound(py);
    dict.set_item("total", stats.total)?;
    dict.set_item("converted", stats.converted)?;
    dict.set_item("failed", stats.failed)?;
    Ok(dict.into())
}

/// Strava export converter
///
/// Handles extraction of .fit.gz files, conversion to GPX, and metadata injection
/// from activities.csv for Strava exports.
///
/// Example:
///     >>> from fit2gpx_lightning import StravaConverter
///     >>> converter = StravaConverter("strava_export.zip")
///     >>> converter.unzip_activities()
///     >>> converter.strava_fit_to_gpx("./gpx_files/")
///     >>> converter.add_metadata_to_gpx("./gpx_files/")
#[pyclass]
struct StravaConverter {
    inner: crate::strava::StravaConverter,
}

#[pymethods]
impl StravaConverter {
    /// Create a new StravaConverter
    ///
    /// Args:
    ///     dir_in (str): Path to Strava export ZIP or extracted directory
    #[new]
    fn new(dir_in: String, verbose: Option<bool>) -> Self {
        if verbose.is_none() {
            return Self {
                inner: crate::strava::StravaConverter::new(dir_in),
            };
        }

        let verbose = verbose.unwrap_or(false);
        Self {
            inner: crate::strava::StravaConverter::new(dir_in).with_verbose(verbose),
        }
    }

    /// Convert FIT files to GPX
    ///
    /// Args:
    ///     output_dir (str): Directory where GPX files will be written
    ///
    /// Returns:
    ///     dict: Statistics with keys 'total', 'converted', 'failed', 'matched'
    fn strava_fit_to_gpx(&mut self, py: Python, output_dir: String) -> PyResult<PyObject> {
        let stats = self
            .inner
            .strava_fit_to_gpx(&output_dir)
            .map_err(|e| PyRuntimeError::new_err(format!("Conversion failed: {}", e)))?;

        let dict = PyDict::new_bound(py);
        dict.set_item("total", stats.total)?;
        dict.set_item("converted", stats.converted)?;
        dict.set_item("failed", stats.failed)?;
        dict.set_item("matched", stats.matched)?;
        Ok(dict.into())
    }

    /// Add metadata from activities.csv to GPX files
    ///
    /// Args:
    ///     gpx_dir (str): Directory containing GPX files to update
    fn add_metadata_to_gpx(&mut self, gpx_dir: String) -> PyResult<()> {
        self.inner
            .add_metadata_to_gpx(&gpx_dir)
            .map_err(|e| PyRuntimeError::new_err(format!("Metadata injection failed: {}", e)))
    }
}

/// Garmin export converter
///
/// Handles extraction of FIT files from nested ZIPs, conversion to GPX,
/// and metadata injection from summarizedActivities.json for Garmin exports.
///
/// Example:
///     >>> from fit2gpx_lightning import GarminConverter
///     >>> converter = GarminConverter("garmin_export.zip", verbose=True)
///     >>> converter.garmin_fit_to_gpx("./gpx_files/")
///     >>> converter.add_metadata_to_gpx("./gpx_files/")
#[pyclass]
struct GarminConverter {
    inner: crate::garmin::GarminConverter,
}

#[pymethods]
impl GarminConverter {
    /// Create a new GarminConverter
    ///
    /// Args:
    ///     dir_in (str): Path to Garmin export ZIP
    ///     verbose (bool, optional): Enable verbose output (default: False)
    #[new]
    fn new(dir_in: String, verbose: Option<bool>) -> Self {
        if verbose.is_none() {
            return Self {
                inner: crate::garmin::GarminConverter::new(dir_in),
            };
        }

        let verbose = verbose.unwrap_or(false);
        Self {
            inner: crate::garmin::GarminConverter::new(dir_in).with_verbose(verbose),
        }
    }

    /// Convert FIT files to GPX with metadata matching from summarizedActivities.json to rename files to activity IDs
    ///
    /// Args:
    ///     output_dir (str): Directory where GPX files will be written
    ///
    /// Returns:
    ///     dict: Statistics with keys 'total', 'converted', 'failed', 'matched', 'unmatched'
    fn garmin_fit_to_gpx(&mut self, py: Python, output_dir: String) -> PyResult<PyObject> {
        let stats = self
            .inner
            .garmin_fit_to_gpx(&output_dir)
            .map_err(|e| PyRuntimeError::new_err(format!("Conversion failed: {}", e)))?;

        let dict = PyDict::new_bound(py);
        dict.set_item("total", stats.total)?;
        dict.set_item("converted", stats.converted)?;
        dict.set_item("failed", stats.failed)?;
        dict.set_item("matched", stats.matched)?;
        dict.set_item("unmatched", stats.unmatched)?;
        Ok(dict.into())
    }

    /// Add metadata to already-converted GPX files, using metadata from summarizedActivities.json
    ///
    /// Args:
    ///     gpx_dir (str): Directory containing GPX files to update
    fn add_metadata_to_gpx(&mut self, gpx_dir: String) -> PyResult<()> {
        self.inner
            .add_metadata_to_gpx(&gpx_dir)
            .map_err(|e| PyRuntimeError::new_err(format!("Metadata injection failed: {}", e)))
    }
}

/// fit2gpx-lightning: Fast FIT to GPX converter
///
/// A high-performance FIT to GPX converter written in Rust with Python bindings.
/// Provides both simple conversion functions and full-featured converters for
/// Strava and Garmin exports.
///
/// Example:
///     >>> from fit2gpx_lightning import fit_to_gpx, fit_to_gpx_bulk
///     >>> from fit2gpx_lightning import StravaConverter, GarminConverter
///     >>>
///     >>> # Simple conversion
///     >>> fit_to_gpx("activity.fit", "activity.gpx")
///     >>>
///     >>> # Bulk conversion
///     >>> stats = fit_to_gpx_bulk("./fit_files/", "./gpx_files/")
///     >>>
///     >>> # Strava export processing
///     >>> converter = StravaConverter("strava_export.zip")
///     >>> converter.strava_fit_to_gpx("./output/")
#[pymodule]
fn fit2gpx_lightning(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Functions
    m.add_function(wrap_pyfunction!(fit_to_gpx, m)?)?;
    m.add_function(wrap_pyfunction!(fit_to_gpx_bulk, m)?)?;

    // Classes
    m.add_class::<StravaConverter>()?;
    m.add_class::<GarminConverter>()?;

    Ok(())
}
