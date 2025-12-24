#!/usr/bin/env python3
"""
Quick Test: Verify fit2gpx-lightning installation

This script performs a quick test to verify that fit2gpx-lightning is
properly installed and working with the test data.

Usage:
    python examples/quick_test.py
"""

import sys
import os
import tempfile
import shutil
from pathlib import Path


def test_installation():
    """Test if fit2gpx-lightning is installed"""
    print("Testing fit2gpx-lightning installation...")
    print()

    try:
        import fit2gpx_lightning
        print("✓ fit2gpx-lightning is installed")
        return True
    except ImportError:
        print("✗ fit2gpx-lightning is NOT installed")
        print()
        print("Install with: pip install fit2gpx-lightning")
        print("Or build from source: maturin develop --release")
        return False


def test_basic_api():
    """Test basic API functions are available"""
    print()
    print("Testing API availability...")

    try:
        from fit2gpx_lightning import (
            fit_to_gpx,
            fit_to_gpx_bulk,
            StravaConverter,
            GarminConverter
        )
        print("✓ fit_to_gpx")
        print("✓ fit_to_gpx_bulk")
        print("✓ StravaConverter")
        print("✓ GarminConverter")
        return True
    except ImportError as e:
        print(f"✗ API import failed: {e}")
        return False


def test_strava_conversion():
    """Test Strava conversion with test data"""
    print()
    print("Testing Strava conversion...")

    # Find test data
    script_dir = Path(__file__).parent
    strava_zip = script_dir.parent.parent / 'strava.zip'

    if not strava_zip.exists():
        print("⚠  Test data not found, skipping conversion test")
        print(f"   Expected: {strava_zip}")
        return None

    try:
        from fit2gpx_lightning import StravaConverter

        # Create temp output directory
        output_dir = tempfile.mkdtemp(prefix='test_strava_')

        try:
            converter = StravaConverter(str(strava_zip))
            stats = converter.strava_fit_to_gpx(output_dir)
            converter.add_metadata_to_gpx(output_dir)

            print(f"✓ Converted {stats['converted']} activities")
            print(f"  Total: {stats['total']}")
            print(f"  Failed: {stats['failed']}")

            # Check output files
            gpx_files = [f for f in os.listdir(output_dir) if f.endswith('.gpx')]
            print(f"  GPX files created: {len(gpx_files)}")

            return True

        finally:
            # Cleanup
            shutil.rmtree(output_dir, ignore_errors=True)

    except Exception as e:
        print(f"✗ Conversion failed: {e}")
        import traceback
        traceback.print_exc()
        return False


def test_garmin_conversion():
    """Test Garmin conversion with test data"""
    print()
    print("Testing Garmin conversion...")

    # Find test data
    script_dir = Path(__file__).parent
    garmin_zip = script_dir.parent.parent / 'garmin.zip'

    if not garmin_zip.exists():
        print("⚠  Test data not found, skipping conversion test")
        print(f"   Expected: {garmin_zip}")
        return None

    try:
        from fit2gpx_lightning import GarminConverter

        # Create temp output directory
        output_dir = tempfile.mkdtemp(prefix='test_garmin_')

        try:
            converter = GarminConverter(str(garmin_zip))
            stats = converter.garmin_fit_to_gpx(output_dir)

            print(f"✓ Converted {stats['converted']} activities")
            print(f"  Total: {stats['total']}")
            print(f"  Matched: {stats['matched']}")
            print(f"  Unmatched: {stats['unmatched']}")
            print(f"  Failed: {stats['failed']}")

            # Check output files
            gpx_files = [f for f in os.listdir(output_dir) if f.endswith('.gpx')]
            print(f"  GPX files created: {len(gpx_files)}")

            return True

        finally:
            # Cleanup
            shutil.rmtree(output_dir, ignore_errors=True)

    except Exception as e:
        print(f"✗ Conversion failed: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    print("=" * 60)
    print("fit2gpx-lightning Quick Test")
    print("=" * 60)
    print()

    results = []

    # Test installation
    results.append(("Installation", test_installation()))

    if not results[-1][1]:
        print()
        print("=" * 60)
        print("Cannot proceed without installation")
        print("=" * 60)
        sys.exit(1)

    # Test API
    results.append(("API", test_basic_api()))

    if not results[-1][1]:
        print()
        print("=" * 60)
        print("API test failed")
        print("=" * 60)
        sys.exit(1)

    # Test conversions
    strava_result = test_strava_conversion()
    if strava_result is not None:
        results.append(("Strava Conversion", strava_result))

    garmin_result = test_garmin_conversion()
    if garmin_result is not None:
        results.append(("Garmin Conversion", garmin_result))

    # Summary
    print()
    print("=" * 60)
    print("Test Summary")
    print("=" * 60)
    print()

    all_passed = True
    for name, result in results:
        if result:
            print(f"✓ {name}")
        elif result is None:
            print(f"⊘ {name} (skipped)")
        else:
            print(f"✗ {name}")
            all_passed = False

    print()

    if all_passed:
        print("=" * 60)
        print("🎉 All tests passed!")
        print("=" * 60)
        print()
        print("fit2gpx-lightning is ready to use!")
        print()
        print("Next steps:")
        print("  • Process your own Strava export: python examples/process_strava.py export.zip")
        print("  • Process your own Garmin export: python examples/process_garmin.py export.zip")
        print("  • Run benchmark: python examples/benchmark.py")
        print()
        sys.exit(0)
    else:
        print("=" * 60)
        print("Some tests failed")
        print("=" * 60)
        sys.exit(1)


if __name__ == "__main__":
    main()
