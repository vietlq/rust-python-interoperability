// TODO: Expose a function named `max_k` that takes a list of unsigned integers and return as output
//   a list containing the `k` largest numbers in the list, in descending order.
//
// Hint: you can use the `num_bigint` crate if you think it'd be useful.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList};

#[pyfunction]
fn max_k<'py>(
    item: Bound<'py, PyAny>,
    num_to_take: usize,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyList>> {
    let the_pylist_conv = item.downcast_into::<PyList>();

    match the_pylist_conv {
        Err(_) => Err(PyTypeError::new_err(
            "Expected a list of unsigned integer up to 128 bits",
        )),
        Ok(the_pylist) => {
            let mut input_list: Vec<u128> = vec![];
            let mut valid_subitems = true;

            for subitem in the_pylist {
                match subitem.extract() {
                    Ok(num) => input_list.push(num),
                    _ => valid_subitems = false,
                }
            }

            if !valid_subitems {
                Err(PyTypeError::new_err(
                    "Some elements in the list are not unsigned integer values up to 128 bits",
                ))
            } else {
                if num_to_take == 0 {
                    let empty_vec: Vec<u128> = vec![];
                    Ok(PyList::new(py, empty_vec)?)
                } else if num_to_take > input_list.len() {
                    Err(PyValueError::new_err(
                        "The list has fewer elements than requested",
                    ))
                } else {
                    input_list.sort_by(|a, b| a.cmp(b));
                    input_list.reverse();

                    if num_to_take == input_list.len() {
                        Ok(PyList::new(py, input_list)?)
                    } else {
                        let (left, _right) = input_list.split_at(num_to_take as usize);
                        Ok(PyList::new(py, left.to_vec())?)
                    }
                }
            }
        }
    }
}

#[pymodule]
fn outro1(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(max_k, m)?)?;
    Ok(())
}
