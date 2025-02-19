use pyo3::prelude::*;
use pyo3::types::PyList;

/**
 * https://rust-exercises.com/rust-python-interop/01_intro/05_gil
 *
 * There is overhead in converting a Python object into a Rust-native type.
 * That overhead might dominate the cost of invoking your Rust function
 * if the function itself isn't doing much computational work.
 * In those cases, it can be desirable to work directly using Python's
 * in-memory representation of the object. That's where the Py* types
 * come in: they give you direct access to Python objects, with minimal overhead.
 *
 * Out of the entire family of Py* types, PyAny deserves a special mention.
 * It's the most general Python-native type in pyo3: it stands for an
 * arbitrary Python object. You can use it whenever you don't know the
 * exact type of the object you're working with, or you don't care about it.
 *
 * Python<'py> is the cornerstone of the entire system: it's a token type
 * that guarantees that you're holding the GIL. All APIs that require you to
 * hold the GIL will, either directly or indirectly,
 * require you to provide a Python<'py> token as proof.
 *
 * 'py, the lifetime parameter of Python<'py>, is used to represent
 * how long the GIL is going to be held.

 use pyo3::prelude::*;
// There is no runtime difference between invoking the two functions
// below from Python.
// The first one is just more explicit about the fact that it requires
// the caller to acquire the GIL ahead of time.

#[pyfunction]
fn print_number_list(_py: Python<'_>, list: Vec<u64>) {
    todo!()
}

#[pyfunction]
fn print_number_list2(list: Vec<u64>) {
    todo!()
}

You won't be interacting with Python<'py> directly most of the time.
Instead, you'll use the Bound<'py, T> type, a smart pointer
that encapsulates a reference to a Python object, ensuring that you're
holding the GIL when you're interacting with it.

Using Bound<'py, T> we can finally start using the Py* types as function arguments:

use pyo3::prelude::*;

#[pyfunction]
fn print_number_list(list: Bound<'_, PyList>) {
    todo!()
}

Bound ensures that we're holding the GIL when interacting with the
list instance that has been passed to us as function argument.
*/

#[pyfunction]
// TODO: Use `PyList` instead of `Vec<u64>` as the input type. Panic on errors, for now.
// You might find this useful: https://pyo3.rs/v0.22.0/conversions/traits#extract-and-the-frompyobject-trait
fn print_number_list(xs: Bound<'_, PyList>) {
    if xs.len() < 1 {
        return;
    }

    for x in xs {
        println!("{}", x)
    }
}

#[pymodule]
fn gil(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(print_number_list, m)?)?;
    Ok(())
}
