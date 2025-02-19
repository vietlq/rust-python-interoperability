//! Modify this extension to get the tests in `test_sample.py` to pass.
use pyo3::prelude::*;

#[pyfunction]
fn it_works() -> bool {
    true
}

/// A Python module implemented in Rust.
/// The fix: We added `name="modules"` here to customise the name of the Python module
#[pymodule(name="modules")]
fn learn_modules(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(it_works, m)?)?;
    Ok(())
}
