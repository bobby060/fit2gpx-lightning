# Python Integration Tests

This directory contains integration tests for the fit2gpx-lightning Python bindings.

## Test Coverage

The test suite covers:

- **fit_to_gpx**: Single FIT to GPX file conversion
- **fit_to_gpx_bulk**: Bulk directory conversion
- **StravaConverter**: Complete Strava export processing workflow
- **GarminConverter**: Complete Garmin export processing workflow

## Required Test Data

Tests require the following data files (not included in git):

```
testdata/           # Directory with sample .fit files
  ├── 1637314040.fit
  ├── 1644116107.fit
  └── ... (more .fit files)
strava.zip          # Strava bulk export (download from Strava)
garmin.zip          # Garmin bulk export (download from Garmin)
```

## Running Tests

### Prerequisites

```bash
# Install development dependencies
pip install -r requirements-dev.txt

# Build the Python package
cd fit2gpx-lightning
maturin develop --release
cd ..
```

### Run Tests

```bash
# Run all tests
pytest tests/ -v

# Run specific test class
pytest tests/test_python_bindings.py::TestFitToGpx -v

# Run specific test
pytest tests/test_python_bindings.py::TestFitToGpx::test_single_file_conversion -v

# Run with coverage
pytest tests/ -v --cov=fit2gpx_lightning --cov-report=html
```

## Test Organization

Tests are organized into classes by functionality:

- `TestFitToGpx`: Tests for single file conversion
- `TestFitToGpxBulk`: Tests for bulk directory conversion
- `TestStravaConverter`: Tests for Strava export processing
- `TestGarminConverter`: Tests for Garmin export processing
- `TestEndToEnd`: End-to-end integration tests

## CI/CD

These tests run automatically in GitHub Actions CI on all platforms (Ubuntu, macOS, Windows) as part of the test job.
