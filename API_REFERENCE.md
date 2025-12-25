# API Reference

## Core Conversion Functions

### `fit_to_gpx(input_path, output_path)`

Convert a single FIT file to GPX format.

**Parameters:**
- `input_path` (str): Path to the input `.fit` file
- `output_path` (str): Path where the `.gpx` file will be written

**Returns:** None

**Raises:**
- `FileNotFoundError`: If the input file doesn't exist
- `RuntimeError`: If conversion fails

**Example:**
```python
from fit2gpx_lightning import fit_to_gpx

fit_to_gpx('morning_run.fit', 'morning_run.gpx')
```

---

### `fit_to_gpx_bulk(input_dir, output_dir)`

Convert all FIT files in a directory to GPX format using parallel processing.

**Parameters:**
- `input_dir` (str): Directory containing `.fit` files (immediate children only, not recursive)
- `output_dir` (str): Directory where `.gpx` files will be written (created if doesn't exist)

**Returns:** dict with keys:
- `total` (int): Total number of FIT files found
- `converted` (int): Number successfully converted
- `failed` (int): Number that failed to convert

**Raises:**
- `FileNotFoundError`: If input directory doesn't exist
- `RuntimeError`: If output directory cannot be created

**Example:**
```python
from fit2gpx_lightning import fit_to_gpx_bulk

stats = fit_to_gpx_bulk('./activities/', './gpx_output/')
print(f"Converted {stats['converted']}/{stats['total']} files")
if stats['failed'] > 0:
    print(f"Failed: {stats['failed']}")
```

---

## Strava Export Processing

### `StravaConverter(zip_path)`

Process Strava bulk export archives.

**Constructor Parameters:**
- `zip_path` (str): Path to Strava export ZIP file

**Raises:**
- `FileNotFoundError`: If ZIP file doesn't exist
- `RuntimeError`: If ZIP file is invalid

**Example:**
```python
from fit2gpx_lightning import StravaConverter

converter = StravaConverter('strava_export.zip')
```

### `StravaConverter.strava_fit_to_gpx(output_dir)`

Extract and convert activity files from Strava export to GPX format.

Handles multiple file types:
- `.fit` files → converted to GPX
- `.fit.gz` files → decompressed and converted to GPX
- `.gpx.gz` files → decompressed to GPX
- `.gpx` files → copied directly

**Parameters:**
- `output_dir` (str): Directory where GPX files will be written (created if doesn't exist)

**Returns:** dict with keys:
- `total` (int): Total number of activity files found
- `converted` (int): Number successfully processed
- `failed` (int): Number that failed


**Raises:**
- `RuntimeError`: If extraction or conversion fails

**Example:**
```python
stats = converter.strava_fit_to_gpx('./strava_gpx/')
print(f"Extracted {stats['converted']} activities")
```

### `StravaConverter.add_metadata_to_gpx(gpx_dir)`

Inject activity names and types from `activities.csv` into GPX files.

Matches GPX files to CSV entries by Strava activity ID (extracted from filename). Modifies GPX files in-place by adding:
- Activity name to `<name>` tag
- Activity type to `<type>` tag

**Parameters:**
- `gpx_dir` (str): Directory containing GPX files to update

**Raises:**
- `FileNotFoundError`: If directory doesn't exist or `activities.csv` not found in ZIP
- `RuntimeError`: If CSV parsing fails

**Example:**
```python
stats = converter.add_metadata_to_gpx('./strava_gpx/')
print(f"Added metadata to {stats['matched']} files")
if stats['unmatched'] > 0:
    print(f"No metadata for {stats['unmatched']} files")
```

---

## Garmin Export Processing

### `GarminConverter(zip_path)`

Process Garmin bulk export archives.

**Constructor Parameters:**
- `zip_path` (str): Path to Garmin export ZIP file

**Raises:**
- `FileNotFoundError`: If ZIP file doesn't exist
- `RuntimeError`: If ZIP file is invalid

**Example:**
```python
from fit2gpx_lightning import GarminConverter

converter = GarminConverter('garmin_export.zip')
```

### `GarminConverter.garmin_fit_to_gpx(output_dir)`

Extract FIT files from nested Garmin ZIP structure, convert to GPX, and automatically inject metadata.

Garmin exports contain nested ZIP files. This method:
1. Extracts all FIT files from nested structure
2. Converts them to GPX format
3. Automatically matches and injects metadata from `summarizedActivities.json` by timestamp (±10 second tolerance)

**Parameters:**
- `output_dir` (str): Directory where GPX files will be written (created if doesn't exist)

**Returns:** dict with keys:
- `total` (int): Total FIT files found
- `converted` (int): Files successfully converted
- `failed` (int): Files that failed conversion

**Raises:**
- `RuntimeError`: If extraction or conversion fails

**Example:**
```python
stats = converter.garmin_fit_to_gpx('./garmin_gpx/')
print(f"Converted: {stats['converted']}")
```

### `GarminConverter.add_metadata_to_gpx(gpx_dir)`

Update existing GPX files with metadata from `summarizedActivities.json`.

Matches GPX files to activities by comparing FIT start time to activity start time (±10 second tolerance). Modifies GPX files in-place by adding:
- Activity name to `<name>` tag
- Distance to `<desc>` tag

**Parameters:**
- `gpx_dir` (str): Directory containing GPX files to update


**Raises:**
- `FileNotFoundError`: If directory doesn't exist or `summarizedActivities.json` not found in ZIP
- `RuntimeError`: If JSON parsing fails

**Example:**
```python
stats = converter.add_metadata_to_gpx('./garmin_gpx/')
print(f"Updated metadata for {stats['matched']}/{stats['total']} files")
```
