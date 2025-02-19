use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList};
use pyo3::Python;

/**
 * https://rust-exercises.com/rust-python-interop/01_intro/07_exceptions.html
 *
 * Use `PyAny` and define `item: Bound<'_, PyAny>`. Then use `item.extract::<u64>()`.
 *
 * Example:

#[pyfunction]
fn print_if_number(item: Bound<'_, PyAny>) -> PyResult<()> {
    let number = item.extract::<u64>()?;
    println!("{}", number);
    Ok(())
}

In the example above, extract::<u64>()? returns a PyResult<u64>.

If the object is not an unsigned integer, extract will return an error,
which will be propagated up to the caller via the ? operator.
On the Python side, this error will be raised as a Python exception by pyo3.

All built-in Python exceptions are available in pyo3::exceptions
E.g. pyo3::exceptions::PyValueError for a ValueError.
You can use their new_err method to create an instance.
*/

fn fibonacci_impl(n: u64) -> Vec<u64> {
    match n {
        0 => vec![],
        1 => vec![0],
        2 => vec![0, 1],
        3 => vec![0, 1, 1],
        _ => {
            let mut prev_vec = fibonacci_impl(n - 1);
            let new_val = prev_vec[prev_vec.len() - 1] + prev_vec[prev_vec.len() - 2];
            prev_vec.push(new_val);
            prev_vec
        }
    }
}

#[pyfunction]
// TODO: Implement a function that returns a list containing the first `n` numbers in Fibonacci's sequence.
//  It must raise a `TypeError` if `n` is not an integer or if it is less than 0.
// Remember to always return `Bound<'py, PyList>`, even inside `PyResult`.
// Never return naked `PyList`.
// NOTE: Here we put the common lifetime with name `'py` to `fibonacci` and all its arguments.
// We raise exception in Python by defining return type `PyResult`
// and returning `Err(PyTypeError::new_err(..))`
fn fibonacci<'py>(item: Bound<'py, PyAny>, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
    match item.extract::<u64>() {
        Ok(num) => Ok(PyList::new(py, fibonacci_impl(num))?),
        _ => Err(PyTypeError::new_err(
            "Invalid type/value. Expected unsigned integer values only.",
        )),
    }
}

#[pymodule]
fn exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    Ok(())
}
