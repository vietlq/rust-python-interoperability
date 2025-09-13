// TODO: Expose a function named `max_k` that takes a list of unsigned integers and return as output
//   a list containing the `k` largest numbers in the list, in descending order.
//
// Hint: you can use the `num_bigint` crate if you think it'd be useful.
use num_bigint::BigUint;
use pyo3::prelude::*;
use pyo3::types::{PyInt, PyList};
use std::ffi::CString;
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
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyTypeError, _>(e.to_string()))
}

fn biguint_to_pyint(py: Python, big_int: &BigUint) -> PyResult<PyObject> {
    // Convert BigUint back to Python int via string
    let int_str = big_int.to_string();

    // Use Python's int() constructor directly
    // let int_type = py.get_type::<pyo3::types::PyInt>();
    // let result = int_type.call1((int_str,));
    // Ok(result.into())
    Ok((py.eval(&CString::new(int_str)?.as_c_str(), None, None)?).into())
}

fn pylist_to_biguints(py: Python, pylist: &Bound<'_, PyList>) -> PyResult<Vec<BigUint>> {
    let items: Vec<PyObject> = pylist.extract()?;
    items
        .into_iter()
        .map(|item| pyint_to_biguint(py, &item))
        .collect()
}

#[pyfunction]
fn max_k(py: Python, int_list: &Bound<'_, PyList>, k: usize) -> PyResult<Py<PyList>> {
    if int_list.len() < k {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "k is greater than the length of the list".to_string(),
        ));
    }

    if k == 0 {
        // Both ways to return an empty list are fine
        // return Ok(PyList::new(py, Vec::<PyObject>::new())?.into());
        return Ok(PyList::empty(py).into());
    }

    let mut biguints = pylist_to_biguints(py, int_list)?;

    // Sort in descending order and take k largest
    biguints.sort_by(|a, b| b.cmp(a)); // descending order
    biguints.truncate(k);

    // Convert back to Python list
    let py_results: PyResult<Vec<_>> = biguints
        .iter()
        .map(|big_int| biguint_to_pyint(py, big_int))
        .collect();

    Ok(PyList::new(py, py_results?)?.into())
}

#[pymodule]
fn outro1(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(max_k, m)?)?;
    Ok(())
}
