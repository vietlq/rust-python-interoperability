/*
* https://rust-exercises.com/rust-python-interop/03_concurrency/04_minimize_gil_locking.html
*
* Minimize GIL locking

All our examples so far fall into two categories:

    The Rust function holds the GIL for the entire duration of its execution.
    The Rust function doesn't hold the GIL at all, going straight into
Python::allow_threads mode.

Real-world applications are often more nuanced, though.
You'll need to hold the GIL for some operations
(e.g. passing data back to Python), but you're able to release it for others
(e.g. long-running computations).

The goal is to minimize the time spent holding the GIL to the bare minimum,
thus maximizing the potential parallelism of your application.
Strategy 1: isolate the GIL-free section

Let's look at an example: we're given a list of numbers and we need to modify
it in place, replacing each number with the result of an expensive computation
that uses no Python objects.

To minimize GIL locking, we create Rust vector from the Python list, release
the GIL, and perform the computation and then re-acquire the GIL to update the
Python list in place:

#[pyfunction]
fn update_in_place<'py>(
    python: Python<'py>,
    numbers: Bound<'py, PyList>
) -> PyResult<()> {
    // Holding the GIL
    let v: Vec<i32> = numbers.extract()?;
    let updated_v: Vec<_> = python.allow_threads(|| {
        v.iter().map(|&n| expensive_computation(n)).collect()
    });
    // Back to holding the GIL
    for (i, &n) in updated_v.iter().enumerate() {
        numbers.set_item(i, n)?;
    }
    Ok(()
}

fn expensive_computation(n: i32) -> i32 {
    // Some heavy number crunching
    // [...]
}

Strategy 2: manually re-acquire the GIL inside the closure

In the example above, we've created a whole new vector to decouple the GIL-free
section from the GIL-holding one. If the input data is large, this can be
a significant overhead.

Let's explore a different approach: we won't create a new pure-Rust vector.
Instead, we will re-acquire the GIL inside the closure—we'll hold it to access
each list element and, after the computation is done, update it in place.
Nothing more.

We need to re-bind it using a Python<'py> token, thus getting a
Bound<'py, PyList> back. How do we get a Python<'py> token inside the closure,
though? Using Python::with_gil: it's the opposite of Python::allow_threads,
it makes sure to acquire the GIL before executing the closure and release it
afterwards. The closure is given a Python token as argument, which we can use
to re-bind the PyList object:

#[pyfunction]
fn update_in_place<'py>(
    python: Python<'py>,
    numbers: Bound<'py, PyList>
) -> PyResult<()> {
    let n_numbers = numbers.len();
    let numbers_ref = numbers.unbind();
    // Release the GIL
    python.allow_threads(|| -> PyResult<()> {
        for i in 0..n_numbers {
            // Acquire the GIL again, to access the
            // i-th element of the list
            let n = Python::with_gil(|inner_py| {
                numbers_ref
                    .bind(inner_py)
                    .get_item(i)?
                    .extract::<i64>()
            })?;
            // Run the computation without holding the GIL
            let result = expensive_computation(n);
            // Re-acquire the GIL to update the list in place
            Python::with_gil(|inner_py| {
                numbers_ref.bind(inner_py).set_item(i, result)
            })?;
        }
        Ok(())
    })
}

Be mindful of concurrency

The GIL is there for a reason: to protect Python objects from concurrent access.
Whenever you release the GIL, you're allowing other threads to run and
potentially modify the Python objects you're working with.

In the examples above, another Python thread could modify the numbers list
while we're computing the result. E.g. it could remove an element, causing
the index i to be out of bounds.

This is a common issue in multi-threaded programming, and it's up to you
to handle it.
Consider using synchronization primitives like Lock to serialize access to
the Python objects you're working with. In other words, move towards
fine-grained locking rather than the lock-the-world approach you
get with the GIL.
* */
use primes::factors_uniq;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use rayon::prelude::*;

#[pyfunction]
// You're given a Python list of non-negative numbers.
// You need to return a Python dictionary where the keys are the numbers in the list and the values
// are the unique prime factors of each number, sorted in ascending order.
//
// # Resources
//
// You can use `factors_uniq` from the `primes` crate to compute the prime factors of a number.
//
// # Constraints
//
// Don't hold the GIL while computing the prime factors
//
// # Fun additional challenge
//
// Can you use multiple threads to parallelize the computation?
// Consider using `rayon` to make it easier.
fn compute_prime_factors<'python>(
    python: Python<'python>,
    numbers: Bound<'python, PyList>,
) -> PyResult<Bound<'python, PyDict>> {
    // Step 1. Create empty containers
    let rs_numbers = numbers.extract::<Vec<u64>>()?;
    let out_dict = PyDict::new(python);
    let mut result: Vec<(u64, Vec<u64>)> = Vec::new();

    // Step 2. Release GIL and do expensive computations
    python.allow_threads(|| {
        // let chrono::
        result = rs_numbers
            .par_iter()
            .map(|number| {
                let number = number.clone();
                let vec_unique_factors = factors_uniq(number);
                (number.clone(), vec_unique_factors)
            })
            .collect();
    });

    // Step 3. Populate the result and return
    for (num, factors) in result {
        out_dict.set_item(num, factors)?;
    }

    Ok(out_dict)
}

#[pymodule]
fn minimize(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_prime_factors, m)?)?;
    Ok(())
}
