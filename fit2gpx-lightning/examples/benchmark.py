#!/usr/bin/env python3
"""
Benchmark: Compare fit2gpx vs fit2gpx-lightning

This script compares the performance of the original Python fit2gpx library
against fit2gpx-lightning (Rust-based) for processing Strava exports.

Requirements:
    pip install fit2gpx fit2gpx-lightning

Usage:
    python examples/benchmark.py [path_to_strava.zip]

If no path is provided, uses the test data at ../strava.zip
"""

import sys
import os
import time
import tempfile
import shutil
from pathlib import Path


def benchmark_fit2gpx_lightning(strava_zip, output_dir):
    """Benchmark fit2gpx-lightning"""
    from fit2gpx_lightning import StravaConverter

    start = time.time()

    converter = StravaConverter(strava_zip)
    stats = converter.strava_fit_to_gpx(output_dir)
    converter.add_metadata_to_gpx(output_dir)

    duration = time.time() - start

    return {
        'duration': duration,
        'total': stats['total'],
        'converted': stats['converted'],
        'failed': stats['failed']
    }


def benchmark_original_fit2gpx(strava_zip, output_dir):
    """Benchmark original Python fit2gpx"""
    try:
        from fit2gpx import StravaConverter
    except ImportError:
        return None

    start = time.time()

    # Extract to temp directory first (original fit2gpx needs extracted files)
    temp_extract = tempfile.mkdtemp()
    try:
        import zipfile
        with zipfile.ZipFile(strava_zip, 'r') as zip_ref:
            zip_ref.extractall(temp_extract)

        converter = StravaConverter(dir_in=temp_extract)
        converter.unzip_activities()
        converter.add_metadata_to_gpx()

        converter.strava_fit_to_gpx()

        # Count converted files
        gpx_dir = os.path.join(temp_extract, 'activities_gpx')
        if os.path.exists(gpx_dir):
            converted = len([f for f in os.listdir(gpx_dir) if f.endswith('.gpx')])
            # Copy to output dir for comparison
            if not os.path.exists(output_dir):
                os.makedirs(output_dir)
            for f in os.listdir(gpx_dir):
                shutil.copy(os.path.join(gpx_dir, f), output_dir)
        else:
            converted = 0

        duration = time.time() - start

        return {
            'duration': duration,
            'total': converted,
            'converted': converted,
            'failed': 0
        }

    finally:
        # Cleanup temp directory
        shutil.rmtree(temp_extract, ignore_errors=True)


def format_duration(seconds):
    """Format duration in human-readable format"""
    if seconds < 1:
        return f"{seconds * 1000:.0f}ms"
    elif seconds < 60:
        return f"{seconds:.2f}s"
    else:
        mins = int(seconds / 60)
        secs = seconds % 60
        return f"{mins}m {secs:.1f}s"


def run_benchmark(strava_zip):
    """Run complete benchmark comparison"""
    print("=" * 70)
    print("FIT to GPX Conversion Benchmark")
    print("=" * 70)
    print()

    # Verify input file
    if not os.path.exists(strava_zip):
        print(f"❌ Error: File not found: {strava_zip}")
        sys.exit(1)

    print(f"📦 Test data: {strava_zip}")
    file_size = os.path.getsize(strava_zip) / (1024 * 1024)
    print(f"📊 Archive size: {file_size:.1f} MB")
    print()

    # Check which libraries are available
    try:
        import fit2gpx_lightning
        has_lightning = True
    except ImportError:
        has_lightning = False
        print("⚠️  fit2gpx-lightning not installed")

    try:
        import fit2gpx
        has_original = True
    except ImportError:
        has_original = False
        print("⚠️  fit2gpx not installed (pip install fit2gpx)")

    if not has_lightning and not has_original:
        print("\n❌ Error: No libraries available to benchmark")
        print("Install at least one: pip install fit2gpx-lightning fit2gpx")
        sys.exit(1)

    print()
    results = {}

    # Benchmark fit2gpx-lightning
    if has_lightning:
        print("⚡ Benchmarking fit2gpx-lightning (Rust)...")
        output_lightning = tempfile.mkdtemp(prefix='lightning_')
        try:
            result = benchmark_fit2gpx_lightning(strava_zip, output_lightning)
            results['lightning'] = result

            print(f"   Duration: {format_duration(result['duration'])}")
            print(f"   Converted: {result['converted']} files")
            print(f"   Throughput: {result['converted'] / result['duration']:.1f} files/sec")
            print()

        except Exception as e:
            print(f"   ❌ Error: {e}")
            print()
        finally:
            shutil.rmtree(output_lightning, ignore_errors=True)

    # Benchmark original fit2gpx
    if has_original:
        print("🐍 Benchmarking fit2gpx (Python)...")
        output_original = tempfile.mkdtemp(prefix='original_')
        try:
            result = benchmark_original_fit2gpx(strava_zip, output_original)

            if result:
                results['original'] = result

                print(f"   Duration: {format_duration(result['duration'])}")
                print(f"   Converted: {result['converted']} files")
                print(f"   Throughput: {result['converted'] / result['duration']:.1f} files/sec")
                print()
            else:
                print("   ❌ Benchmark failed")
                print()

        except Exception as e:
            print(f"   ❌ Error: {e}")
            print()
        finally:
            shutil.rmtree(output_original, ignore_errors=True)

    # Comparison
    if len(results) == 2:
        print("=" * 70)
        print("📊 Comparison Results")
        print("=" * 70)
        print()

        lightning = results['lightning']
        original = results['original']

        speedup = original['duration'] / lightning['duration']

        print(f"{'Library':<25} {'Duration':<15} {'Throughput':<20} {'Speedup':<10}")
        print("-" * 70)

        # Original
        throughput_original = original['converted'] / original['duration']
        print(f"{'fit2gpx (Python)':<25} "
              f"{format_duration(original['duration']):<15} "
              f"{throughput_original:.1f} files/sec{'':<9} "
              f"1.0x")

        # Lightning
        throughput_lightning = lightning['converted'] / lightning['duration']
        print(f"{'fit2gpx-lightning (Rust)':<25} "
              f"{format_duration(lightning['duration']):<15} "
              f"{throughput_lightning:.1f} files/sec{'':<9} "
              f"{speedup:.1f}x ⚡")

        print()
        print("=" * 70)
        print("🏆 Summary")
        print("=" * 70)
        print()
        print(f"   fit2gpx-lightning is {speedup:.1f}x faster than fit2gpx")
        print(f"   Time saved: {format_duration(original['duration'] - lightning['duration'])}")
        print()

        if speedup >= 10:
            print("   ⚡⚡⚡ BLAZINGLY FAST! ⚡⚡⚡")
        elif speedup >= 5:
            print("   ⚡⚡ Very Fast! ⚡⚡")
        elif speedup >= 2:
            print("   ⚡ Faster! ⚡")

        print()

    elif 'lightning' in results:
        print("=" * 70)
        print("ℹ️  Only fit2gpx-lightning was benchmarked")
        print("=" * 70)
        print()
        print("Install fit2gpx to compare: pip install fit2gpx")
        print()

    elif 'original' in results:
        print("=" * 70)
        print("ℹ️  Only fit2gpx was benchmarked")
        print("=" * 70)
        print()
        print("Install fit2gpx-lightning to compare: pip install fit2gpx-lightning")
        print()


def main():
    # Get input file from command line or use default test data
    if len(sys.argv) > 1:
        strava_zip = sys.argv[1]
    else:
        # Default to test data
        script_dir = Path(__file__).parent
        strava_zip = str(script_dir.parent.parent / 'strava.zip')

        if not os.path.exists(strava_zip):
            print("Usage: python benchmark.py <path_to_strava.zip>")
            print()
            print("No test data found. Please provide a Strava export ZIP file.")
            sys.exit(1)

        print(f"ℹ️  Using test data: {strava_zip}")
        print()

    run_benchmark(strava_zip)


if __name__ == "__main__":
    main()
