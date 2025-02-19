// TODO: Expose a function named `max_k` that takes a list of unsigned integers and return as output
//   a list containing the `k` largest numbers in the list, in descending order.
//
// Hint: you can use the `num_bigint` crate if you think it'd be useful.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyList;

#[pyfunction]
fn max_k<'py>(
    input_list: Vec<u64>,
    num_to_take: u64,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyList>> {
    let list_len = (input_list.len() as u64);
    if num_to_take > list_len {
        Err(PyValueError::new_err(
            "The list has fewer elements than requested",
        ))
    } else {
        if num_to_take == 0 {
            let empty_vec: Vec<u64> = vec![];
            Ok(PyList::new(py, empty_vec)?)
        } else {
            let mut input_list = input_list.clone();
            input_list.sort_by(|a, b| a.cmp(b));
            input_list.reverse();

            if num_to_take == list_len {
                Ok(PyList::new(py, input_list)?)
            } else {
                let (left, _right) = input_list.split_at(num_to_take as usize);
                Ok(PyList::new(py, left.to_vec())?)
            }
        }
    }
}

#[pymodule]
fn outro1(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(max_k, m)?)?;
    Ok(())
}
