#!/usr/bin/env python3
"""
Example: Process Garmin export archive

This example demonstrates how to process a complete Garmin bulk export,
extracting FIT files from nested ZIPs, converting to GPX, and matching
with metadata from summarizedActivities.json.

Usage:
    python examples/process_garmin.py [path_to_garmin.zip]

If no path is provided, it will use the test data at ../garmin.zip
"""

from fit2gpx_lightning import GarminConverter
import sys
import os
from pathlib import Path


def process_garmin_export(export_zip, output_dir='./output_garmin_gpx'):
    """
    Complete Garmin export processing workflow

    Args:
        export_zip: Path to Garmin bulk export ZIP file
        output_dir: Directory for GPX output (default: ./output_garmin_gpx)

    Returns:
        dict: Conversion statistics
    """
    print("=" * 60)
    print("Garmin Export Processor")
    print("=" * 60)
    print()

    # Verify input file exists
    if not os.path.exists(export_zip):
        print(f"❌ Error: File not found: {export_zip}")
        sys.exit(1)

    # Create output directory
    os.makedirs(output_dir, exist_ok=True)
    print(f"📁 Input:  {export_zip}")
    print(f"📁 Output: {output_dir}")
    print()

    # Initialize converter
    print("🔧 Initializing converter...")
    converter = GarminConverter(export_zip)
    print()

    # Step 1: Extract FIT files (optional - mainly for information)
    print("📦 Step 1: Scanning archive...")
    extract_stats = converter.extract_fit_files()
    print(f"   Found {extract_stats['total']} FIT files in nested archives")
    print()

    # Step 2: Convert to GPX with metadata matching
    print("⚡ Step 2: Converting FIT to GPX with metadata matching...")
    print("   (Matching FIT files to activities by timestamp ±10s)")
    stats = converter.garmin_fit_to_gpx(output_dir)
    print(f"   ✓ Converted: {stats['converted']}")
    print(f"   ✓ Matched with metadata: {stats['matched']}")
    print(f"   ⚠ Unmatched (no metadata): {stats['unmatched']}")
    print(f"   ✗ Failed: {stats['failed']}")
    print()

    # Summary
    print("=" * 60)
    print("✅ Processing Complete!")
    print("=" * 60)
    print(f"📊 Summary:")
    print(f"   Total FIT files: {stats['total']}")
    print(f"   Successfully converted: {stats['converted']}")
    print(f"   Matched with metadata: {stats['matched']}")
    print(f"   Unmatched: {stats['unmatched']}")
    print(f"   Failed: {stats['failed']}")
    if stats['total'] > 0:
        print(f"   Success rate: {stats['converted'] / stats['total'] * 100:.1f}%")
        print(f"   Metadata match rate: {stats['matched'] / stats['total'] * 100:.1f}%")
    print()
    print(f"📁 GPX files saved to: {os.path.abspath(output_dir)}")
    print()
    print("ℹ️  Note: Unmatched files indicate FIT files without corresponding")
    print("   metadata in summarizedActivities.json (timestamp mismatch)")
    print()

    return stats


def main():
    # Get input file from command line or use default test data
    if len(sys.argv) > 1:
        export_zip = sys.argv[1]
    else:
        # Default to test data
        script_dir = Path(__file__).parent
        export_zip = str(script_dir.parent.parent / 'garmin.zip')
        print(f"ℹ️  No input file specified, using test data: {export_zip}")
        print()

    # Process the export
    stats = process_garmin_export(export_zip)

    # Exit with appropriate code
    sys.exit(0 if stats['failed'] == 0 else 1)


if __name__ == "__main__":
    main()
