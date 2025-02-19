use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyInt;

// TODO: Add a `__new__` constructor to the `ShoppingOrder` class that takes the following arguments:
//  - `name` (non-empty string)
//  - `price` (non-zero integer)
//  - `quantity` (non-zero integer)
//  The constructor should raise a `ValueError` if any of the arguments are invalid.

#[pyclass]
struct ShoppingOrder {
    #[pyo3(get)]
    name: String,

    #[pyo3(get)]
    price: u64,

    #[pyo3(get, set)]
    quantity: u64,
}

#[pymethods]
impl ShoppingOrder {
    #[new]
    fn new<'py>(
        name: String,
        price: Bound<'py, PyInt>,
        quantity: Bound<'py, PyInt>,
        py: Python<'py>,
    ) -> PyResult<ShoppingOrder> {
        let res_price = price.extract::<u64>();
        let res_quantity = quantity.extract::<u64>();

        match (res_price, res_quantity) {
            (Ok(price), Ok(quantity)) => Ok(ShoppingOrder {
                name,
                price,
                quantity,
            }),
            _ => Err(PyValueError::new_err(
                "price and quantity must be of the type u64",
            )),
        }
    }
}

#[pymodule]
fn constructors(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ShoppingOrder>()?;
    Ok(())
}
