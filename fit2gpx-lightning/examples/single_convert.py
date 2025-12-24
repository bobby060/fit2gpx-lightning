#!/usr/bin/env python3
"""
Example: Convert single FIT file to GPX

This example shows how to convert a single FIT file with proper error handling.

Usage:
    python examples/single_convert.py <input.fit> [output.gpx]

Example:
    python examples/single_convert.py morning_run.fit morning_run.gpx
"""

from fit2gpx_lightning import fit_to_gpx
import sys
import os


def convert_file(input_file, output_file=None):
    """
    Convert a single FIT file to GPX with error handling

    Args:
        input_file: Path to .fit file
        output_file: Path to .gpx output (optional)

    Returns:
        bool: True if successful, False otherwise
    """
    # Generate output filename if not provided
    if output_file is None:
        base = os.path.splitext(input_file)[0]
        output_file = f"{base}.gpx"

    print("=" * 60)
    print("FIT to GPX Converter")
    print("=" * 60)
    print()
    print(f"📄 Input:  {input_file}")
    print(f"📄 Output: {output_file}")
    print()

    # Convert with error handling
    try:
        print("⚡ Converting...")
        fit_to_gpx(input_file, output_file)

        print()
        print("=" * 60)
        print("✅ Conversion Successful!")
        print("=" * 60)
        print()
        print(f"📁 GPX file saved to: {os.path.abspath(output_file)}")
        print()

        # Show file sizes
        input_size = os.path.getsize(input_file)
        output_size = os.path.getsize(output_file)
        print(f"📊 File sizes:")
        print(f"   Input (FIT):  {input_size:,} bytes")
        print(f"   Output (GPX): {output_size:,} bytes")
        print()

        return True

    except FileNotFoundError:
        print()
        print("=" * 60)
        print("❌ Error: File Not Found")
        print("=" * 60)
        print()
        print(f"The input file does not exist: {input_file}")
        print()
        return False

    except RuntimeError as e:
        print()
        print("=" * 60)
        print("❌ Error: Conversion Failed")
        print("=" * 60)
        print()
        print(f"Details: {e}")
        print()
        print("Possible causes:")
        print("  • File is corrupted or not a valid FIT file")
        print("  • File is encrypted or protected")
        print("  • Insufficient disk space for output")
        print()
        return False

    except Exception as e:
        print()
        print("=" * 60)
        print("❌ Unexpected Error")
        print("=" * 60)
        print()
        print(f"Details: {e}")
        print()
        return False


def main():
    # Parse command line arguments
    if len(sys.argv) < 2:
        print("Usage: python single_convert.py <input.fit> [output.gpx]")
        print()
        print("Example:")
        print("  python single_convert.py morning_run.fit")
        print("  python single_convert.py morning_run.fit evening_run.gpx")
        print()
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2] if len(sys.argv) > 2 else None

    # Run conversion
    success = convert_file(input_file, output_file)

    # Exit with appropriate code
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
