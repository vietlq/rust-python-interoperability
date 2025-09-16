/*
* https://rust-exercises.com/rust-python-interop/03_concurrency/05_immutable_types.html
*
* Let's see how we can define a similar immutable type in Rust.

use pyo3::prelude::*;

#[pyclass(frozen)]
struct Point {
    x: i32,
    y: i32,
}

The above is not enough to get all the niceties of Python's dataclasses,
but it's sufficient to make the class immutable.
If a pyclass is marked as frozen, pyo3 will allow us to access its fields
without holding the GIL—i.e. via Py<T> instead of Bound<'py, T>

#[pyfunction]
fn print_point<'py>(python: Python<'py>, point: Bound<'py, Point>) {
    let point: Py<Point> = point.unbind();
    python.allow_threads(|| {
        // We can now access the fields of the Point struct
        // even though we are not holding the GIL
        let point: &Point = point.get();
        println!("({}, {})", point.x, point.y);
    });
}

This wouldn't compile if Point wasn't marked as frozen,
thanks to Py<T>::get's signature:

impl<T> Py<T>
where
    T: PyClass,
{
    pub fn get(&self) -> &T
    where
        // `Frozen = True` is where the magic happens!
        T: PyClass<Frozen = True> + Sync,
    { /* ... */ }
}

Summary

Immutable types significantly simplify GIL jugglery in pyo3. If it fits the
constraints of the problem you're solving, consider using them to make
your code easier to reason about (and potentially faster!).
* */
use pyo3::prelude::*;

#[pyclass(frozen)]
struct Rectangle {
    width: u32,
    length: u32,
}

#[pymethods]
impl Rectangle {
    #[new]
    fn new(width: u32, length: u32) -> Self {
        Self { width, length }
    }

    fn area(&self) -> u32 {
        self.width * self.length
    }
}

#[pyfunction]
/// Compute the area of a rectangle while allowing Python to run other threads.
/// Fill in the body of the function.
/// Modify `Rectangle`'s definition if necessary.
///
/// # Constraints
///
/// Do NOT remove the `allow_threads` call. The computation must be done inside
/// the closure passed to `allow_threads`.
fn compute_area<'py>(python: Python<'py>, shape: Bound<'py, Rectangle>) -> u32 {
    // We must unbind before entering Python::allow_threads()
    let shape = shape.unbind();

    python.allow_threads(|| {
        // Thanks to ``#[pyclass(frozen)]`, we can use `shape.get()` here.
        let area: u32 = shape.get().area();
        area
    })
}

#[pymodule]
fn immutable(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_area, m)?)?;
    m.add_class::<Rectangle>()?;
    Ok(())
}
