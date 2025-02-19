use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::Python;

/**
 * https://rust-exercises.com/rust-python-interop/01_intro/06_output
 *
 * https://pyo3.rs/main/doc/pyo3/types/struct.pylist
 *
 * https://pyo3.rs/v0.22.0/conversions/traits#the-topyobject-trait
 *
 * We need to pass `py: Python<'_>` so that we can use `PyList::new`
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
fn fibonacci(n: u64, py: Python<'_>) -> Bound<'_, PyList> {
    PyList::new(py, fibonacci_impl(n)).unwrap()
}

#[pymodule]
fn output(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    Ok(())
}
