use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyInt;

// https://rust-exercises.com/rust-python-interop/02_classes/01_constructors.html

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

/**
 * Signature for `new`

Everything we learned about arguments in the context of #[pyfunction]s
applies to constructors as well.
In terms of output type, you can return Self if the
constructor is infallible, or PyResult<Self> if it can fail.
 */

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
            (Ok(price), Ok(quantity)) => {
                if price < 1 || quantity < 1 {
                    Err(PyValueError::new_err(
                        "price and quantity must be positive integers",
                    ))
                } else {
                    Ok(ShoppingOrder {
                        name,
                        price,
                        quantity,
                    })
                }
            }
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
