# Deployment Guide

This guide explains how to deploy `fit2gpx-lightning` to PyPI using GitHub Actions CI/CD.

## Overview

The `.github/workflows/ci.yml` workflow automates the entire build, test, and deployment process. It builds Python wheels for multiple platforms (Linux, macOS Intel/ARM, Windows) and automatically publishes them to PyPI when you create a GitHub release.

## What ci.yml Does

The workflow consists of three main jobs:

### 1. Test Job

**Purpose**: Validate the code works on all platforms before building wheels.

**Runs on**:
- Ubuntu (latest)
- macOS (latest)
- Windows (latest)

**Steps**:
1. Checks out the repository
2. Installs Rust toolchain
3. Runs `cargo test --all-features` to execute all tests
4. Runs `cargo clippy -- -D warnings` to catch code quality issues

**Triggers**: Every push to `main` branch and on release creation

### 2. Build Wheels Job

**Purpose**: Build platform-specific Python wheels for distribution.

**Runs on**:
- `ubuntu-latest` (x86_64) - Builds manylinux wheels (compatible with most Linux distros)
- `ubuntu-latest` (aarch64) - Builds manylinux wheels for Linux ARM64 (Raspberry Pi, AWS Graviton, etc.)
- `macos-13` - Builds wheels for macOS Intel (x86_64)
- `macos-14` - Builds wheels for macOS Apple Silicon (arm64)
- `windows-latest` - Builds wheels for Windows

**Steps**:
1. Checks out the repository
2. Sets up Python
3. Installs Rust toolchain
4. Uses `PyO3/maturin-action@v1` to build wheels
   - `--release` flag for optimized builds
   - `--out dist` to output wheels to dist/ directory
   - `--features python` to enable PyO3 bindings
   - `manylinux: auto` for Linux compatibility
5. Uploads wheels as GitHub artifacts

**Triggers**: Only on release creation (not on regular pushes)

**Output**: Platform-specific `.whl` files:
- `fit2gpx_lightning-0.1.0-cp38-abi3-manylinux_2_17_x86_64.whl` (Linux x86_64)
- `fit2gpx_lightning-0.1.0-cp38-abi3-manylinux_2_17_aarch64.whl` (Linux ARM64)
- `fit2gpx_lightning-0.1.0-cp38-abi3-macosx_10_12_x86_64.whl` (macOS Intel)
- `fit2gpx_lightning-0.1.0-cp38-abi3-macosx_11_0_arm64.whl` (macOS ARM)
- `fit2gpx_lightning-0.1.0-cp38-abi3-win_amd64.whl` (Windows)

### 3. Publish Job

**Purpose**: Upload built wheels to PyPI.

**Runs on**: Ubuntu (latest)

**Steps**:
1. Downloads all wheel artifacts from the build-wheels job
2. Merges wheels from all platforms into single `dist/` directory
3. Uses `PyO3/maturin-action@v1` with `command: upload` to publish to PyPI
4. Uses `--skip-existing` to avoid errors if some wheels already exist

**Triggers**: Only on release creation, after build-wheels job succeeds

**Requirements**: `MATURIN_PYPI_TOKEN` secret must be configured

## Prerequisites

Before deploying, you need to:

### 1. Create a PyPI Account

1. Go to https://pypi.org/account/register/
2. Create an account and verify your email
3. Enable two-factor authentication (recommended)

### 2. Create a PyPI API Token

1. Log in to PyPI
2. Go to Account Settings → API tokens
3. Click "Add API token"
4. Set token name: `fit2gpx-lightning-github-ci`
5. Set scope: "Entire account" (or limit to specific project after first upload)
6. Click "Create token"
7. **IMPORTANT**: Copy the token immediately (starts with `pypi-`)
   - Format: `pypi-AgEIcHlwaS5vcmc...`
   - You won't be able to see it again!

### 3. Add PyPI Token to GitHub Secrets

1. Go to your GitHub repository
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Name: `PYPI_API_TOKEN`
5. Value: Paste your PyPI token (the full `pypi-...` string)
6. Click **Add secret**

**Note**: The workflow expects the secret to be named `PYPI_API_TOKEN`. The workflow configuration maps this to `MATURIN_PYPI_TOKEN` for the maturin action.

### 4. Verify Repository Settings

Ensure GitHub Actions is enabled:
1. Go to **Settings** → **Actions** → **General**
2. Under "Actions permissions", select "Allow all actions and reusable workflows"
3. Under "Workflow permissions", select "Read and write permissions"

## Deployment Steps

### Step 1: Prepare for Release

1. **Update version number** in `Cargo.toml`:
   ```toml
   [package]
   version = "0.1.0"  # Change to your new version
   ```

2. **Update version** in `pyproject.toml`:
   ```toml
   [project]
   version = "0.1.0"  # Must match Cargo.toml
   ```

3. **Update CHANGELOG** (if you have one):
   ```markdown
   ## v0.1.0 (2024-12-15)
   - Initial release
   - Core FIT to GPX conversion
   - Strava and Garmin export support
   ```

4. **Commit changes**:
   ```bash
   git add Cargo.toml pyproject.toml
   git commit -m "Bump version to 0.1.0"
   git push origin main
   ```

### Step 2: Test Locally (Optional but Recommended)

Before creating a release, test the build locally:

```bash
# Run tests
cargo test --all-features

# Build Python wheel locally
maturin build --release --features python

# Install and test the wheel
pip install target/wheels/fit2gpx_lightning-*.whl
python examples/quick_test.py
```

### Step 3: Create a GitHub Release

#### Option A: Via GitHub Web Interface

1. Go to your repository on GitHub
2. Click **Releases** (right sidebar)
3. Click **Draft a new release**
4. Click **Choose a tag** → Type `v0.1.0` → Click "Create new tag: v0.1.0 on publish"
5. Set **Release title**: `v0.1.0` or `fit2gpx-lightning v0.1.0`
6. Write **Release notes**:
   ```markdown
   # fit2gpx-lightning v0.1.0

   Initial release of fit2gpx-lightning - a blazingly fast FIT to GPX converter.

   ## Features
   - 🚀 10-50x faster than Python fit2gpx
   - 📊 Full Strava export support
   - 🏃 Complete Garmin export support
   - ⚡ Parallel processing
   - 🐍 Simple Python API

   ## Installation
   ```bash
   pip install fit2gpx-lightning
   ```

   ## Quick Start
   See [README.md](README.md) for usage examples.
   ```
7. Click **Publish release**

#### Option B: Via Command Line

```bash
# Create and push tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# Create release via GitHub CLI (if installed)
gh release create v0.1.0 \
  --title "fit2gpx-lightning v0.1.0" \
  --notes "Initial release. See README for details."
```

### Step 4: Monitor the Deployment

1. **Watch the workflow**:
   - Go to **Actions** tab in your GitHub repository
   - Click on the workflow run triggered by your release
   - Monitor the three jobs: `test`, `build-wheels`, `publish`

2. **Expected timeline**:
   - Test job: ~2-5 minutes
   - Build wheels job: ~5-10 minutes per platform (runs in parallel)
   - Publish job: ~1-2 minutes

3. **Check for errors**:
   - If any job fails, click on it to see detailed logs
   - Common issues are listed in the Troubleshooting section below

### Step 5: Verify Deployment

1. **Check PyPI**:
   - Go to https://pypi.org/project/fit2gpx-lightning/
   - Verify your package appears
   - Check that all platform wheels are available

2. **Test installation**:
   ```bash
   # Create a fresh virtual environment
   python -m venv test_env
   source test_env/bin/activate  # On Windows: test_env\Scripts\activate

   # Install from PyPI
   pip install fit2gpx-lightning

   # Test it works
   python -c "from fit2gpx_lightning import fit_to_gpx; print('Success!')"
   ```

3. **Test on different platforms** (if possible):
   - Linux: `pip install fit2gpx-lightning`
   - macOS: `pip install fit2gpx-lightning`
   - Windows: `pip install fit2gpx-lightning`

## Workflow Diagram

```
Push to main          Create Release (tag)
     ↓                       ↓
┌────────────────────────────────────────────┐
│  Test Job (Linux, macOS, Windows)         │
│  - cargo test                              │
│  - cargo clippy                            │
└────────────────────────────────────────────┘
                               ↓
              ┌────────────────────────────────┐
              │  Build Wheels Job (parallel)   │
              ├────────────────────────────────┤
              │  • Linux (manylinux)           │
              │  • macOS Intel (x86_64)        │
              │  • macOS ARM (arm64)           │
              │  • Windows (amd64)             │
              └────────────────────────────────┘
                               ↓
              ┌────────────────────────────────┐
              │  Publish Job                   │
              │  - Collect all wheels          │
              │  - Upload to PyPI              │
              └────────────────────────────────┘
                               ↓
                      PyPI Package Live! 🎉
```

## Updating an Existing Package

To release a new version:

1. Make your code changes
2. Update version in `Cargo.toml` and `pyproject.toml`
3. Commit and push changes
4. Create a new release with a new tag (e.g., `v0.2.0`)
5. GitHub Actions will automatically build and deploy

## Troubleshooting

### Issue: "Authentication failed" during publish

**Cause**: PyPI token is missing, incorrect, or expired.

**Solution**:
1. Verify secret exists: GitHub repo → Settings → Secrets → Actions
2. Verify secret name is exactly `PYPI_API_TOKEN`
3. Generate a new PyPI token and update the secret
4. Re-run the failed workflow

### Issue: "File already exists" error during publish

**Cause**: Trying to upload a version that already exists on PyPI.

**Solution**:
- You cannot overwrite existing PyPI releases
- Increment the version number in `Cargo.toml` and `pyproject.toml`
- Create a new release with the new version

### Issue: Build wheels job fails with "cargo build failed"

**Cause**: Compilation error in Rust code.

**Solution**:
1. Test locally first: `cargo build --release --features python`
2. Fix any compilation errors
3. Push fixes and create a new release

### Issue: Test job fails

**Cause**: Tests are failing on one or more platforms.

**Solution**:
1. Run tests locally: `cargo test --all-features`
2. Fix failing tests
3. Push fixes - CI will automatically re-run tests on the next push

### Issue: Wheels missing for some platforms

**Cause**: Build job failed for specific platform.

**Solution**:
1. Check the build-wheels job logs for that platform
2. Common causes:
   - Platform-specific compilation issues
   - Missing system dependencies
   - Timeout (builds take too long)
3. Fix issues and create a new release

### Issue: "Project name already in use" on first upload

**Cause**: Someone else registered that name on PyPI.

**Solution**:
- Choose a different package name
- Update name in `Cargo.toml` and `pyproject.toml`
- Update the module name in `.github/workflows/ci.yml` if needed

## Advanced Configuration

### Building for Specific Python Versions

The current configuration uses `abi3` for broad compatibility (Python 3.8+). To target specific versions:

```toml
# pyproject.toml
[tool.maturin]
features = ["python", "abi3"]  # Remove abi3 for version-specific builds
```

### Adding More Platforms

To build for additional platforms, add them to the matrix in `ci.yml`.

**Current platforms** (as of this version):
- Linux x86_64
- Linux ARM64 (aarch64) ✨
- macOS Intel (x86_64)
- macOS Apple Silicon (arm64)
- Windows x64

**Example - Adding Linux ARMv7** (for older Raspberry Pi):

```yaml
strategy:
  matrix:
    platform:
      - os: ubuntu-latest
        target: x86_64
      - os: ubuntu-latest
        target: aarch64
      - os: ubuntu-latest
        target: armv7  # NEW: For Raspberry Pi 2/3
      # ... other platforms
```

### Using TestPyPI for Testing

Before publishing to the real PyPI, test with TestPyPI:

1. Create account at https://test.pypi.org/
2. Create API token
3. Add as secret: `TEST_PYPI_API_TOKEN`
4. Modify the publish job in `ci.yml`:

```yaml
- uses: PyO3/maturin-action@v1
  env:
    MATURIN_PYPI_TOKEN: ${{ secrets.TEST_PYPI_API_TOKEN }}
  with:
    command: upload
    args: --skip-existing dist/* --repository testpypi
```

5. Install from TestPyPI:
   ```bash
   pip install --index-url https://test.pypi.org/simple/ fit2gpx-lightning
   ```

## Security Best Practices

1. **Never commit API tokens** to your repository
2. **Use scoped tokens** - limit PyPI tokens to specific projects when possible
3. **Rotate tokens regularly** - especially if you suspect compromise
4. **Enable 2FA** on your PyPI account
5. **Review workflow permissions** - use minimum necessary permissions
6. **Monitor releases** - watch for unexpected package uploads

## Additional Resources

- [Maturin Documentation](https://www.maturin.rs/)
- [PyO3 Guide](https://pyo3.rs/)
- [PyPI Help](https://pypi.org/help/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Packaging Python Projects](https://packaging.python.org/tutorials/packaging-projects/)

## Support

If you encounter issues not covered in this guide:

1. Check the [GitHub Actions logs](https://github.com/yourusername/fit2gpx-lightning/actions)
2. Review [PyO3 maturin-action issues](https://github.com/PyO3/maturin-action/issues)
3. Open an issue in the repository

---

**Ready to deploy?** Follow the steps above and your package will be live on PyPI in ~15 minutes! 🚀
