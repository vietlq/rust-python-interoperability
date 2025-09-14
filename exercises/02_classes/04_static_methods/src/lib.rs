// https://rust-exercises.com/rust-python-interop/02_classes/04_static_methods.html
// TODO: Add an additional method to the `Discount` class to return a default 10% discount.
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pyclass]
struct Discount {
    #[pyo3(get)]
    percentage: f64,
}

#[pymethods]
impl Discount {
    #[new]
    fn new(percentage: f64) -> Self {
        Discount { percentage }
    }

    #[staticmethod]
    fn default() -> Self {
        Discount::new(0.1)
    }

    // Notice the cls argument
    // Since we can refer to Self type in #[staticmethod] in Rust,
    // we don't usually need classmethod like in Python.
    #[classmethod]
    fn default_class_method(_cls: &Bound<'_, PyType>) -> Self {
        Discount::new(0.1)
    }
}

#[pymodule]
fn static_methods(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Discount>()?;
    Ok(())
}
