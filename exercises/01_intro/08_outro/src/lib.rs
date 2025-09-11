// TODO: Expose a function named `max_k` that takes a list of unsigned integers and return as output
//   a list containing the `k` largest numbers in the list, in descending order.
//
// Hint: you can use the `num_bigint` crate if you think it'd be useful.
use num_bigint::BigUint;
use pyo3::prelude::*;
use pyo3::types::{PyInt, PyList};
use std::str::FromStr;

fn pyint_to_biguint(py: Python, pyobj: &PyObject) -> PyResult<BigUint> {
    let pyint = pyobj.downcast_bound::<PyInt>(py)?;

    // Fast path for small integers
    if let Ok(small_val) = pyint.extract::<u64>() {
        return Ok(BigUint::from(small_val));
    }

    // For large integers, convert to decimal string
    let int_str: String = pyint.call_method0("__str__")?.extract()?;
    BigUint::from_str(&int_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

fn pyint_to_biguint_hex(py: Python, pyobj: &PyObject) -> PyResult<BigUint> {
    let pyint = pyobj.downcast_bound::<PyInt>(py)?;

    // Fast path for small integers
    if let Ok(small_val) = pyint.extract::<u64>() {
        return Ok(BigUint::from(small_val));
    }

    // Use Python's hex() for large integers
    let hex_builtin = py.import("builtins")?.getattr("hex")?;
    let hex_result: String = hex_builtin.call1((pyint,))?.extract()?;

    // Remove "0x" prefix
    let hex_str = hex_result.strip_prefix("0x").unwrap_or(&hex_result);

    BigUint::from_str_radix(hex_str, 16)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

fn pylist_to_biguints(py: Python, pylist: &PyList) -> PyResult<Vec<BigUint>> {
    let items: Vec<PyObject> = pylist.extract()?;
    items
        .into_iter()
        .map(|item| pyint_to_biguint(py, &item))
        .collect()
}

#[pyfunction]
fn max_k(py: Python, int_list: &PyList, k: usize) -> &PyList {
    pylist_to_biguints(py, int_list)
}

#[pymodule]
fn outro1(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(max_k, m)?)?;
    Ok(())
}
