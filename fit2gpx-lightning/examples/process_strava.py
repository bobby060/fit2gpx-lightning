#!/usr/bin/env python3
"""
Example: Process Strava export archive

This example demonstrates how to process a complete Strava bulk export,
converting .fit.gz files to GPX format and adding metadata from activities.csv.

Usage:
    python examples/process_strava.py [path_to_strava.zip]

If no path is provided, it will use the test data at ../strava.zip
"""

from fit2gpx_lightning import StravaConverter
import sys
import os
from pathlib import Path


def process_strava_export(export_zip, output_dir='./output_strava_gpx'):
    """
    Complete Strava export processing workflow

    Args:
        export_zip: Path to Strava bulk export ZIP file
        output_dir: Directory for GPX output (default: ./output_strava_gpx)

    Returns:
        dict: Conversion statistics
    """
    print("=" * 60)
    print("Strava Export Processor")
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
    converter = StravaConverter(export_zip, verbose=True)
    print()

    # Step 2: Convert to GPX
    print("⚡ Step 2: Converting FIT files to GPX...")
    stats = converter.strava_fit_to_gpx(output_dir)
    print(f"   ✓ Converted: {stats['converted']}")
    print(f"   ✗ Failed:    {stats['failed']}")
    print()

    # Step 3: Add metadata
    print("📝 Step 3: Adding metadata from activities.csv...")
    converter.add_metadata_to_gpx(output_dir)
    print(f"   ✓ Metadata injected into GPX files")
    print()

    # Summary
    print("=" * 60)
    print("✅ Processing Complete!")
    print("=" * 60)
    print(f"📊 Summary:")
    print(f"   Total activities: {stats['total']}")
    print(f"   Successfully converted: {stats['converted']}")
    print(f"   Failed: {stats['failed']}")
    print(f"   Success rate: {stats['converted'] / stats['total'] * 100:.1f}%")
    print()
    print(f"📁 GPX files saved to: {os.path.abspath(output_dir)}")
    print()

    return stats


def main():
    # Get input file from command line or use default test data
    if len(sys.argv) > 1:
        export_zip = sys.argv[1]
    else:
        # Default to test data
        script_dir = Path(__file__).parent
        export_zip = str(script_dir.parent.parent / 'strava.zip')
        print(f"ℹ️  No input file specified, using test data: {export_zip}")
        print()

    # Process the export
    stats = process_strava_export(export_zip)

    # Exit with appropriate code
    sys.exit(0 if stats['failed'] == 0 else 1)


if __name__ == "__main__":
    main()
