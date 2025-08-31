use pyo3::prelude::*;

#[pyfunction]
// TODO: Implement a function that returns a list containing the first `n` numbers in Fibonacci's sequence.
fn fibonacci(n: u8) -> Vec<u64> {
    if n == 0 {
        return vec![];
    }

    if n == 1 {
        return vec![0];
    }

    let mut result: Vec<u64> = vec![0, 1];
    let mut runner: u8 = 2;
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

#[pymodule]
fn output(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    Ok(())
}
