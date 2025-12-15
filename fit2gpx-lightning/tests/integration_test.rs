use fit2gpx_lightning::{fit_to_gpx, fit_to_gpx_bulk};
use tempfile::TempDir;

#[test]
fn test_fit_to_gpx_bulk_with_empty_directory() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let stats = fit_to_gpx_bulk(input_dir.path(), output_dir.path()).unwrap();

    assert_eq!(stats.total, 0);
    assert_eq!(stats.converted, 0);
    assert_eq!(stats.failed, 0);
}

#[test]
fn test_fit_to_gpx_file_not_found() {
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().join("output.gpx");

    let result = fit_to_gpx("/nonexistent/file.fit", &output_path);
    assert!(result.is_err());
}

#[test]
fn test_bulk_conversion_creates_output_directory() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().join("subdir");

    // Output directory doesn't exist yet
    assert!(!output_path.exists());

    // Should create it automatically
    let _ = fit_to_gpx_bulk(input_dir.path(), &output_path);

    assert!(output_path.exists());
    assert!(output_path.is_dir());
}

// Note: Real conversion tests would require sample FIT files.
// The sample data (garmin.zip and strava.zip) can be used for manual testing.
