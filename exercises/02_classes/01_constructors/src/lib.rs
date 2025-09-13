use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyInt, PyString};

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

/*
fn extract_or_value_error<'py, In: FromPyObject<'py>, Out>(in_val: In) -> PyResult<Out>
where
    In: FromPyObject<'py>,
{
    in_val
        .extract_bound::<Out>(in_val)
        .map_err(|_| PyValueError::new_err(""))?
}
*/

#[pymethods]
impl ShoppingOrder {
    #[new]
    fn new(
        name: &Bound<'_, PyString>,
        price: &Bound<'_, PyInt>,
        quantity: &Bound<'_, PyInt>,
    ) -> PyResult<Self> {
        let price = price
            .extract::<u64>()
            .map_err(|_| PyValueError::new_err("price cannot be negative"))?;
        if price == 0 {
            return Err(PyValueError::new_err("price must be positive"));
        }

        let quantity = quantity
            .extract::<u64>()
            .map_err(|_| PyValueError::new_err("quantity cannot be negative"))?;
        if quantity == 0 {
            return Err(PyValueError::new_err("quantity must be positive"));
        }

        let name = name.to_string();
        if name.len() == 0 {
            return Err(PyValueError::new_err("name cannot be empty"));
        }
        if name.trim().len() == 0 {
            return Err(PyValueError::new_err(
                "name must contain non-space characters",
            ));
        }

        Ok(ShoppingOrder {
            name: name,
            price: price,
            quantity: quantity,
        })
    }
}

#[pymodule]
fn constructors(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ShoppingOrder>()?;
    Ok(())
}
