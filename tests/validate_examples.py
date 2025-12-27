#!/usr/bin/env python3
"""
Validation Script for fit2gpx-lightning Examples

This script validates all example scripts using your test data.
It runs each example and reports success/failure.

Usage:
    python validate_examples.py

Requirements:
    - Test data files in testdata/:
      - At least one .fit file
      - strava.zip (optional, for Strava examples)
      - garmin.zip (optional, for Garmin examples)
"""

import sys
import os
import subprocess
import tempfile
import shutil
from pathlib import Path


class Colors:
    """ANSI color codes for terminal output"""
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    RESET = '\033[0m'
    BOLD = '\033[1m'


def print_header(text):
    """Print a formatted header"""
    print()
    print("=" * 70)
    print(f"{Colors.BOLD}{text}{Colors.RESET}")
    print("=" * 70)


def print_success(text):
    """Print success message"""
    print(f"{Colors.GREEN}✓ {text}{Colors.RESET}")


def print_error(text):
    """Print error message"""
    print(f"{Colors.RED}✗ {text}{Colors.RESET}")


def print_warning(text):
    """Print warning message"""
    print(f"{Colors.YELLOW}⚠ {text}{Colors.RESET}")


def print_info(text):
    """Print info message"""
    print(f"{Colors.BLUE}ℹ {text}{Colors.RESET}")


def check_test_data():
    """Check for required test data"""
    print_header("Checking Test Data")

    project_root = Path(__file__).parent.parent
    testdata_dir = project_root / 'testdata'

    if not testdata_dir.exists():
        print_error(f"Test data directory not found: {testdata_dir}")
        return None

    print_info(f"Test data directory: {testdata_dir}")

    # Check for FIT files
    fit_files = list(testdata_dir.glob('*.fit'))
    if fit_files:
        print_success(f"Found {len(fit_files)} .fit file(s)")
    else:
        print_error("No .fit files found in testdata/")
        return None

    # Check for Strava export
    strava_zip = project_root / 'testdata/strava.zip'
    if strava_zip.exists():
        print_success(f"Found Strava export: {strava_zip}")
    else:
        print_warning(f"Strava export not found: {strava_zip} (Strava examples will be skipped)")

    # Check for Garmin export
    garmin_zip = project_root / 'testdata/garmin.zip'
    if garmin_zip.exists():
        print_success(f"Found Garmin export: {garmin_zip}")
    else:
        print_warning(f"Garmin export not found: {garmin_zip} (Garmin examples will be skipped)")

    return {
        'testdata_dir': testdata_dir,
        'fit_files': fit_files,
        'strava_zip': strava_zip if strava_zip.exists() else None,
        'garmin_zip': garmin_zip if garmin_zip.exists() else None,
    }


def run_example(script_path, args=None, description=""):
    """Run an example script and return success status"""
    print()
    print(f"{Colors.BOLD}Testing: {script_path.name}{Colors.RESET}")
    if description:
        print(f"  {description}")

    cmd = [sys.executable, str(script_path)]
    if args:
        cmd.extend(args)

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=300  # 5 minute timeout
        )

        if result.returncode == 0:
            print_success(f"{script_path.name} completed successfully")
            return True
        else:
            print_error(f"{script_path.name} failed with exit code {result.returncode}")
            if result.stderr:
                print(f"  Error output: {result.stderr[:500]}")
            return False

    except subprocess.TimeoutExpired:
        print_error(f"{script_path.name} timed out after 5 minutes")
        return False
    except Exception as e:
        print_error(f"{script_path.name} failed: {e}")
        return False


def validate_all_examples(test_data):
    """Validate all example scripts"""
    print_header("Validating Example Scripts")

    project_root = Path(__file__).parent.parent
    examples_dir = project_root / 'fit2gpx-lightning' / 'examples'

    if not examples_dir.exists():
        print_error(f"Examples directory not found: {examples_dir}")
        return False

    results = {}
    temp_output_dir = Path(tempfile.mkdtemp(prefix='validate_examples_'))

    try:
        # Test 1: single_convert.py
        first_fit = test_data['fit_files'][0]
        output_gpx = temp_output_dir / 'test_single.gpx'
        results['single_convert'] = run_example(
            examples_dir / 'single_convert.py',
            args=[str(first_fit), str(output_gpx)],
            description="Convert a single FIT file"
        )

        # Test 2: bulk_convert.py
        bulk_output = temp_output_dir / 'bulk_output'
        bulk_output.mkdir(exist_ok=True)
        results['bulk_convert'] = run_example(
            examples_dir / 'bulk_convert.py',
            args=[str(test_data['testdata_dir']), str(bulk_output)],
            description="Bulk convert directory of FIT files"
        )

        # Test 3: process_strava.py (if test data available)
        if test_data['strava_zip']:
            strava_output = temp_output_dir / 'strava_output'
            strava_output.mkdir(exist_ok=True)
            results['process_strava'] = run_example(
                examples_dir / 'process_strava.py',
                args=[str(test_data['strava_zip'])],
                description="Process Strava bulk export"
            )
        else:
            print_warning("Skipping process_strava.py (no strava.zip found)")
            results['process_strava'] = None

        # Test 4: process_garmin.py (if test data available)
        if test_data['garmin_zip']:
            garmin_output = temp_output_dir / 'garmin_output'
            garmin_output.mkdir(exist_ok=True)
            results['process_garmin'] = run_example(
                examples_dir / 'process_garmin.py',
                args=[str(test_data['garmin_zip'])],
                description="Process Garmin bulk export"
            )
        else:
            print_warning("Skipping process_garmin.py (no garmin.zip found)")
            results['process_garmin'] = None

        # Test 5: quick_test.py
        results['quick_test'] = run_example(
            examples_dir / 'quick_test.py',
            description="Quick installation test"
        )

        # Test 6: benchmark.py (if Strava data available)
        if test_data['strava_zip']:
            results['benchmark'] = run_example(
                examples_dir / 'benchmark.py',
                args=[str(test_data['strava_zip'])],
                description="Performance benchmark"
            )
        else:
            print_warning("Skipping benchmark.py (no strava.zip found)")
            results['benchmark'] = None

    finally:
        # Cleanup temp directory
        shutil.rmtree(temp_output_dir, ignore_errors=True)

    return results


def print_summary(results):
    """Print validation summary"""
    print_header("Validation Summary")

    passed = sum(1 for v in results.values() if v is True)
    failed = sum(1 for v in results.values() if v is False)
    skipped = sum(1 for v in results.values() if v is None)
    total = len(results)

    print()
    for name, result in results.items():
        if result is True:
            print_success(f"{name}.py - PASSED")
        elif result is False:
            print_error(f"{name}.py - FAILED")
        else:
            print_warning(f"{name}.py - SKIPPED")

    print()
    print(f"Total: {total} examples")
    print(f"  Passed:  {passed}")
    print(f"  Failed:  {failed}")
    print(f"  Skipped: {skipped}")
    print()

    if failed == 0:
        print_header("🎉 All Examples Validated Successfully!")
        return True
    else:
        print_header("❌ Some Examples Failed")
        return False


def main():
    """Main validation workflow"""
    print_header("fit2gpx-lightning Example Validation")
    print()
    print("This script validates all example scripts with your test data.")
    print()

    # Check test data
    test_data = check_test_data()
    if not test_data:
        print()
        print_error("Cannot proceed without test data")
        print()
        print("Please ensure you have:")
        print("  1. At least one .fit file in testdata/")
        print("  2. Optionally: strava.zip in testdata/")
        print("  3. Optionally: garmin.zip in testdata/")
        print()
        sys.exit(1)

    # Run validation
    results = validate_all_examples(test_data)

    # Print summary
    success = print_summary(results)

    # Exit with appropriate code
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
