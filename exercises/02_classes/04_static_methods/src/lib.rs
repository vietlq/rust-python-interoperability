// TODO: Add an additional method to the `Discount` class to return a default 10% discount.
use pyo3::prelude::*;

#[pyclass]
struct Discount {
    #[pyo3(get)]
    percentage: f64,
}

const DEFAULT_DISCOUNT: f64 = 0.1;

#[pymethods]
impl Discount {
    #[new]
    fn new(percentage: f64) -> Self {
        Discount { percentage }
    }

    #[staticmethod]
    fn default() -> Self {
        Discount {
            percentage: DEFAULT_DISCOUNT,
        }
    }
}

#[pymodule]
fn static_methods(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Discount>()?;
    Ok(())
}
