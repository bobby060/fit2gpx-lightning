#!/usr/bin/env python3
"""
Example: Bulk convert directory of FIT files

This example shows how to convert all FIT files in a directory to GPX format
using parallel processing for maximum performance.

Usage:
    python examples/bulk_convert.py <input_dir> [output_dir]

Example:
    python examples/bulk_convert.py ./my_fit_files/ ./my_gpx_files/
"""

from fit2gpx_lightning import fit_to_gpx_bulk
import sys
import os
from pathlib import Path


def convert_directory(input_dir, output_dir='./output_gpx'):
    """
    Convert all FIT files in a directory to GPX

    Args:
        input_dir: Directory containing .fit files
        output_dir: Directory for .gpx output

    Returns:
        dict: Conversion statistics
    """
    print("=" * 60)
    print("Bulk FIT to GPX Converter")
    print("=" * 60)
    print()

    # Verify input directory exists
    if not os.path.exists(input_dir):
        print(f"❌ Error: Directory not found: {input_dir}")
        sys.exit(1)

    if not os.path.isdir(input_dir):
        print(f"❌ Error: Not a directory: {input_dir}")
        sys.exit(1)

    # Create output directory
    os.makedirs(output_dir, exist_ok=True)

    print(f"📁 Input directory:  {os.path.abspath(input_dir)}")
    print(f"📁 Output directory: {os.path.abspath(output_dir)}")
    print()

    # Count FIT files
    fit_count = len([f for f in os.listdir(input_dir)
                     if f.lower().endswith('.fit')])

    if fit_count == 0:
        print(f"⚠️  No .fit files found in {input_dir}")
        print()
        return {'total': 0, 'converted': 0, 'failed': 0}

    print(f"🔍 Found {fit_count} FIT files")
    print()

    # Convert with progress
    print("⚡ Converting files (using all CPU cores)...")
    stats = fit_to_gpx_bulk(input_dir, output_dir)

    # Display results
    print()
    print("=" * 60)
    print("✅ Conversion Complete!")
    print("=" * 60)
    print(f"📊 Results:")
    print(f"   Total files:     {stats['total']}")
    print(f"   ✓ Converted:     {stats['converted']}")
    print(f"   ✗ Failed:        {stats['failed']}")

    if stats['total'] > 0:
        success_rate = stats['converted'] / stats['total'] * 100
        print(f"   Success rate:    {success_rate:.1f}%")

    print()
    print(f"📁 GPX files saved to: {os.path.abspath(output_dir)}")
    print()

    return stats


def main():
    # Parse command line arguments
    if len(sys.argv) < 2:
        print("Usage: python bulk_convert.py <input_dir> [output_dir]")
        print()
        print("Example:")
        print("  python bulk_convert.py ./my_activities/ ./my_gpx_files/")
        print()
        sys.exit(1)

    input_dir = sys.argv[1]
    output_dir = sys.argv[2] if len(sys.argv) > 2 else './output_gpx'

    # Run conversion
    stats = convert_directory(input_dir, output_dir)

    # Exit with appropriate code
    sys.exit(0 if stats['failed'] == 0 else 1)


if __name__ == "__main__":
    main()
