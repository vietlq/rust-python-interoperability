use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

fn fibonacci_impl(n: u64) -> Vec<u64> {
    if n == 0 {
        return vec![];
    }

    if n == 1 {
        return vec![0];
    }

    let mut result: Vec<u64> = vec![0, 1];
    let mut runner: u64 = 2;
    let mut prev: u64 = 1;
    let mut curr: u64 = 1;

    // Avoiding recursion for better speed and memory utilisation
    while runner < n {
        result.push(curr.into());
        runner += 1;

        let new_prev = curr.clone();
        let new_curr = prev + curr;
        prev = new_prev;
        curr = new_curr;
    }

    result
}

#[pyfunction]
// TODO: Implement a function that returns a list containing the first `n` numbers in Fibonacci's sequence.
//  It must raise a `TypeError` if `n` is not an integer or if it is less than 0.
fn fibonacci(n: &Bound<'_, PyAny>) -> PyResult<Vec<u64>> {
    match n.extract::<u64>() {
        Ok(int_val) => Ok(fibonacci_impl(int_val)),
        Err(_) => Err(PyTypeError::new_err(
            "Invalid type/value. Expected unsigned integer values only.",
        )),
    }
}

#[pymodule]
fn exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    Ok(())
}
