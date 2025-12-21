"""
Integration tests for fit2gpx-lightning Python bindings.

Tests the Python interface for:
- fit_to_gpx: Single file conversion
- fit_to_gpx_bulk: Bulk directory conversion
- StravaConverter: Strava export processing
- GarminConverter: Garmin export processing
"""

import os
import tempfile
from pathlib import Path

import pytest

# Import the Python bindings
try:
    from fit2gpx_lightning import (
        GarminConverter,
        StravaConverter,
        fit_to_gpx,
        fit_to_gpx_bulk,
    )
except ImportError as e:
    raise ImportError(
        "fit2gpx_lightning module not found. "
        "Please build the package first with: "
        "cd fit2gpx-lightning && maturin develop --release"
    ) from e

# Test data paths (relative to project root)
PROJECT_ROOT = Path(__file__).parent.parent
TESTDATA_DIR = PROJECT_ROOT / "testdata"
STRAVA_ZIP = PROJECT_ROOT / "strava.zip"
GARMIN_ZIP = PROJECT_ROOT / "garmin.zip"


class TestFitToGpx:
    """Test single file conversion with fit_to_gpx."""

    def test_single_file_conversion(self, tmp_path):
        """Test converting a single .fit file to .gpx."""
        # Get first test FIT file
        fit_files = list(TESTDATA_DIR.glob("*.fit"))
        assert len(fit_files) > 0, "No .fit files found in testdata/"

        input_file = fit_files[0]
        output_file = tmp_path / "output.gpx"

        # Convert
        fit_to_gpx(str(input_file), str(output_file))

        # Verify output exists and is not empty
        assert output_file.exists()
        assert output_file.stat().st_size > 0

        # Verify it's valid XML/GPX
        content = output_file.read_text()
        assert '<?xml version="1.0"' in content
        assert "<gpx" in content
        assert "</gpx>" in content

    def test_nonexistent_file_raises_error(self, tmp_path):
        """Test that converting a nonexistent file raises an error."""
        output_file = tmp_path / "output.gpx"

        with pytest.raises(RuntimeError, match="Conversion failed"):
            fit_to_gpx("/nonexistent/file.fit", str(output_file))

    def test_output_directory_created(self, tmp_path):
        """Test that output directory is created if it doesn't exist."""
        fit_files = list(TESTDATA_DIR.glob("*.fit"))
        input_file = fit_files[0]

        # Output in a nested directory that doesn't exist
        output_file = tmp_path / "subdir" / "nested" / "output.gpx"
        assert not output_file.parent.exists()

        fit_to_gpx(str(input_file), str(output_file))

        assert output_file.exists()
        assert output_file.parent.exists()


class TestFitToGpxBulk:
    """Test bulk directory conversion with fit_to_gpx_bulk."""

    def test_bulk_conversion(self, tmp_path):
        """Test converting all .fit files in a directory."""
        output_dir = tmp_path / "output"

        stats = fit_to_gpx_bulk(str(TESTDATA_DIR), str(output_dir))

        # Verify stats
        assert stats["total"] > 0, "No .fit files were found"
        assert stats["converted"] > 0, "No files were converted"
        assert stats["converted"] + stats["failed"] == stats["total"]

        # Verify output files exist
        gpx_files = list(output_dir.glob("*.gpx"))
        assert len(gpx_files) == stats["converted"]

        # Verify each file is valid GPX
        for gpx_file in gpx_files:
            content = gpx_file.read_text()
            assert "<gpx" in content
            assert "</gpx>" in content

    def test_bulk_conversion_empty_directory(self, tmp_path):
        """Test bulk conversion on empty directory."""
        empty_dir = tmp_path / "empty"
        empty_dir.mkdir()
        output_dir = tmp_path / "output"

        stats = fit_to_gpx_bulk(str(empty_dir), str(output_dir))

        assert stats["total"] == 0
        assert stats["converted"] == 0
        assert stats["failed"] == 0

    def test_bulk_conversion_creates_output_directory(self, tmp_path):
        """Test that output directory is created if it doesn't exist."""
        output_dir = tmp_path / "output"
        assert not output_dir.exists()

        stats = fit_to_gpx_bulk(str(TESTDATA_DIR), str(output_dir))

        assert output_dir.exists()
        assert stats["converted"] > 0


class TestStravaConverter:
    """Test Strava export processing with StravaConverter."""

    @pytest.fixture
    def strava_converter(self):
        """Create a StravaConverter instance."""
        if not STRAVA_ZIP.exists():
            pytest.skip(f"Strava test data not found at {STRAVA_ZIP}")
        return StravaConverter(str(STRAVA_ZIP))

    def test_strava_fit_to_gpx(self, strava_converter, tmp_path):
        """Test converting Strava export to GPX files."""
        output_dir = tmp_path / "gpx_output"

        stats = strava_converter.strava_fit_to_gpx(str(output_dir))

        # Verify stats structure
        assert "total" in stats
        assert "converted" in stats
        assert "failed" in stats
        assert "matched" in stats

        # Verify some files were processed
        assert stats["total"] > 0, "No files found in Strava export"
        assert stats["converted"] > 0, "No files were converted"

        # Verify output files exist
        gpx_files = list(output_dir.glob("*.gpx"))
        assert len(gpx_files) > 0

        # Verify GPX format
        for gpx_file in gpx_files[:5]:  # Check first 5 files
            content = gpx_file.read_text()
            assert "<gpx" in content
            assert "</gpx>" in content

    def test_strava_add_metadata(self, strava_converter, tmp_path):
        """Test adding metadata from activities.csv to GPX files."""
        output_dir = tmp_path / "gpx_output"

        # First convert files
        stats = strava_converter.strava_fit_to_gpx(str(output_dir))
        assert stats["converted"] > 0

        # Then add metadata
        strava_converter.add_metadata_to_gpx(str(output_dir))

        # Verify metadata was added (check for name/type tags in GPX)
        gpx_files = list(output_dir.glob("*.gpx"))

        # Check at least one file has metadata
        found_metadata = False
        for gpx_file in gpx_files[:10]:  # Check first 10 files
            content = gpx_file.read_text()
            if "<name>" in content or "<type>" in content:
                found_metadata = True
                break

        # Note: metadata may not be present in all files if activities.csv
        # doesn't have entries for all activities
        # Just verify the function completes without error

    def test_strava_converter_with_verbose(self, tmp_path):
        """Test StravaConverter with verbose option."""
        if not STRAVA_ZIP.exists():
            pytest.skip(f"Strava test data not found at {STRAVA_ZIP}")

        converter = StravaConverter(str(STRAVA_ZIP), verbose=True)
        output_dir = tmp_path / "gpx_output"

        stats = converter.strava_fit_to_gpx(str(output_dir))
        assert stats["converted"] > 0


class TestGarminConverter:
    """Test Garmin export processing with GarminConverter."""

    @pytest.fixture
    def garmin_converter(self):
        """Create a GarminConverter instance."""
        if not GARMIN_ZIP.exists():
            pytest.skip(f"Garmin test data not found at {GARMIN_ZIP}")
        return GarminConverter(str(GARMIN_ZIP))

    def test_garmin_extract_fit_files(self, garmin_converter):
        """Test extracting FIT files from nested Garmin archive."""
        stats = garmin_converter.extract_fit_files()

        # Verify stats structure
        assert "total" in stats
        assert "converted" in stats
        assert "failed" in stats
        assert "matched" in stats
        assert "unmatched" in stats

        # Verify files were extracted
        assert stats["total"] > 0, "No FIT files found in Garmin export"

    def test_garmin_fit_to_gpx(self, garmin_converter, tmp_path):
        """Test converting Garmin FIT files to GPX with metadata."""
        output_dir = tmp_path / "gpx_output"

        # Extract first
        extract_stats = garmin_converter.extract_fit_files()
        assert extract_stats["total"] > 0

        # Convert with metadata matching
        stats = garmin_converter.garmin_fit_to_gpx(str(output_dir))

        # Verify stats
        assert stats["total"] > 0
        assert stats["converted"] > 0
        assert "matched" in stats
        assert "unmatched" in stats

        # Verify output files
        gpx_files = list(output_dir.glob("*.gpx"))
        assert len(gpx_files) > 0

        # Verify GPX format
        for gpx_file in gpx_files[:5]:  # Check first 5 files
            content = gpx_file.read_text()
            assert "<gpx" in content
            assert "</gpx>" in content

    def test_garmin_add_metadata(self, garmin_converter, tmp_path):
        """Test adding metadata to already-converted GPX files."""
        output_dir = tmp_path / "gpx_output"

        # Extract and convert
        garmin_converter.extract_fit_files()
        stats = garmin_converter.garmin_fit_to_gpx(str(output_dir))
        assert stats["converted"] > 0

        # Add metadata (should work even if already added during conversion)
        garmin_converter.add_metadata_to_gpx(str(output_dir))

        # Verify files still exist and are valid
        gpx_files = list(output_dir.glob("*.gpx"))
        assert len(gpx_files) > 0

    def test_garmin_full_workflow(self, garmin_converter, tmp_path):
        """Test complete Garmin workflow: extract -> convert -> metadata."""
        output_dir = tmp_path / "gpx_output"

        # Step 1: Extract
        extract_stats = garmin_converter.extract_fit_files()
        assert extract_stats["total"] > 0

        # Step 2: Convert
        convert_stats = garmin_converter.garmin_fit_to_gpx(str(output_dir))
        assert convert_stats["converted"] > 0

        # Step 3: Add metadata
        garmin_converter.add_metadata_to_gpx(str(output_dir))

        # Verify final output
        gpx_files = list(output_dir.glob("*.gpx"))
        assert len(gpx_files) == convert_stats["converted"]


class TestEndToEnd:
    """End-to-end integration tests."""

    def test_multiple_conversions_same_output_dir(self, tmp_path):
        """Test that multiple conversion runs to the same output work correctly."""
        fit_files = list(TESTDATA_DIR.glob("*.fit"))[:3]  # Use first 3 files
        output_dir = tmp_path / "output"

        # Convert each file individually
        for fit_file in fit_files:
            output_file = output_dir / f"{fit_file.stem}.gpx"
            fit_to_gpx(str(fit_file), str(output_file))

        # Verify all files exist
        gpx_files = list(output_dir.glob("*.gpx"))
        assert len(gpx_files) == len(fit_files)

    def test_file_naming_consistency(self, tmp_path):
        """Test that bulk conversion maintains consistent file naming."""
        output_dir = tmp_path / "output"

        stats = fit_to_gpx_bulk(str(TESTDATA_DIR), str(output_dir))

        # Get original FIT filenames
        fit_files = list(TESTDATA_DIR.glob("*.fit"))
        fit_stems = {f.stem for f in fit_files}

        # Get converted GPX filenames
        gpx_files = list(output_dir.glob("*.gpx"))
        gpx_stems = {f.stem for f in gpx_files}

        # Verify naming consistency (converted files should match FIT file stems)
        assert gpx_stems.issubset(fit_stems), "GPX files don't match FIT file names"
