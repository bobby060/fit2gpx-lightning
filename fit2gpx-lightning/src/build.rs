// Build script for pyo3 extension module, needed for macOS compatibility
fn main() {
    pyo3_build_config::add_extension_module_link_args();
}
