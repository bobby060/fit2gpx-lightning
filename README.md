# fit2gpx-lightning

 Simple replacement for the [fit2gpx](https://pypi.org/project/fit2gpx/) Python library with **400x performance improvements** thanks to Rust and parallel processing. I closely mirrored the original fit2gpx API for easy adoption (with a few adaptations and additions).

**[View Full API Reference →](API_REFERENCE.md)** 


 ![PyPI](https://img.shields.io/pypi/v/fit2gpx-lightning?style=flat-square) ![PyPI - Python Version](https://img.shields.io/pypi/pyversions/fit2gpx-lightning?style=flat-square) 

My motivation here was to both expose the wonderful [Rust fit2gpx](https://crates.io/crates/fit2gpx) in python to make it more accessible for those of us who are more comfortable scripting in python. The orignal python `fit2gpx` library is quite slow for bulk conversions and also doesn't have a simple way to process Garmin bulk exports, only Strava. Here, we fix both those issues

> AI Disclaimer: I did use Claude Code for significant portions of the coding and documentation, though I made quite a few manual changes to the final output. I would categorize this as "AI assisted coding" not "Vibecoded."  Could I have done this all myself? Yes. Would it have taken too long for me to bother doing. Also yes.

## Features
- Convert `.fit` files to GPX format, one at a time or an entire folder at once
- Extract all activity tracks as GPX from Strava bulk export ZIPs
  - Supports `.fit`, `.fit.gz`, `.gpx.gz`, and `.gpx` files in source ZIP
  - Inject activity names and types from `activities.csv`
- Extract and convert FIT files to GPX from Garmin bulk export ZIPs
  - Matches FIT files to activity metadata from `summarizedActivities.json` (Activity title, distance)

## Limitations
- We don't support output to pandas Dataframes, which the original fit2gpx library does. 

## Installation

```bash
pip install fit2gpx-lightning
```

I have tried to include pre-built wheels for common platforms (Linux x86_64, Linux aarch64, macOS x86_64, macOS arm64, Windows x86_64). If you have trouble installing, please open an issue with your platform details and I will see if I can add support.

## Quick Start

### Simple Conversion

```python
from fit2gpx_lightning import fit_to_gpx, fit_to_gpx_bulk

# Convert a single file
fit_to_gpx('activity.fit', 'activity.gpx')

# Convert an entire directory 
# (processes immediate children only)
stats = fit_to_gpx_bulk('./fit_files/', './gpx_files/')
print(f"✓ Converted {stats['converted']} out of {stats['total']} files")
print(f"✗ Failed: {stats['failed']}")
```

### Strava Export Processing

The `strava_fit_to_gpx` function not only converts `.fit` files, but also unzips and converts `.fit.gz` files, extracts `.gpx.gz` files, and copies `.gpx`.

Designed to process the archive downloaded from Strava's ["Download or Delete Your Data" page](https://www.strava.com/athlete/delete_your_account).

Complete example processing a Strava export:

```python
from fit2gpx_lightning import StravaConverter

# Initialize converter with your Strava export ZIP path
converter = StravaConverter('strava_export.zip')

# Step 1: Convert FIT files to GPX
stats = converter.strava_fit_to_gpx('./gpx_output/')
print(f"✓ Converted: {stats['converted']}")
print(f"✗ Failed: {stats['failed']}")

# Step 2: Add metadata (activity names, types) from activities.csv. In place modification
converter.add_metadata_to_gpx('./gpx_output/')
```

### Garmin Export Processing

Complete example processing a Garmin export. You can download your data from [Garmin's Data Management page](https://www.garmin.com/account/datamanagement/exportdata.html).

```python
from fit2gpx_lightning import GarminConverter

# Initialize converter with your Garmin export ZIP
converter = GarminConverter('garmin_export.zip')

# Step 1: Convert to GPX with automatic metadata matching
# Matches FIT files to activities by timestamp (±10 second tolerance)
stats = converter.garmin_fit_to_gpx('./gpx_output/')
print(f"✓ Converted: {stats['converted']}")
print(f"✗ Failed: {stats['failed']}")

# Optional: Update metadata for already-converted files
converter.add_metadata_to_gpx('./gpx_output/')
```

## Performance

Benchmark converting my Strava export of 1354 activities.

| Library | Time | Throughput | Speedup |
|---------|------|------------|---------|
| fit2gpx (Python) | 9 m 25.9s| 2.7 files/s | 1x (baseline) |
| **fit2gpx-lightning** | **1.39s** | **971.8 files/s** | **406.1x** |

## Development

### Building from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/bobby060/fit2gpx-lightning
cd fit2gpx-lightning/fit2gpx-lightning

# Install maturin (Python build tool for Rust extensions)
pip install maturin

# Build and install in development mode
maturin develop --release
```

### Running Tests
Integration tests won't pass unless you download your data and move some .fit files to the `test_data/` folder. Strava export zip should be named `strava.zip` and Garmin export zip should be named `garmin.zip`.

```bash
# Rust unit and integration tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_strava

# Python tests (after maturin develop)
python -m pytest tests/
```


## Contributing

Contributions are welcome! 

For feature suggestions or errors, open an issue.

## Changelog

### v0.1.2 (2025-12-27)
- Update documentation for clarity
- Minor code refactoring for readability
- Add support for aarch64 Linux wheels in CI/CD pipeline

### v0.1.1 (2025-12-24)
- Use `abi3` feature of PyO3 to build wheels compatible with multiple Python 3.x versions
- Updated CI/CD pipeline to build `abi3` wheels for Python 3.8+

### v0.1.0 (2025-12-24)

**Initial Release**

-  Core FIT to GPX conversion functions
-  Strava export support (.fit.gz, activities.csv)
-  Garmin export support (nested ZIPs, summarizedActivities.json)
-  Parallel processing for bulk operations
-  Metadata injection from CSV and JSON
-  Python bindings via PyO3
-  CI/CD pipeline for multi-platform wheels
-  Comprehensive documentation and examples
-   Unit testing in rust and integration testing in Python using full exports downloaded in December 2025.

## Support

- **Documentation**: This README + inline code documentation
- **Issues**: [GitHub Issues](https://github.com/bobby060/fit2gpx-lightning/issues)



