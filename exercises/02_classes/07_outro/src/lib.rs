// TODO: Define a base class named `Discount`, with a `percentage` attribute.
//  It should be possible to access the `percentage` attribute of a `Discount`.
//  It should also be possible to modify the `percentage` attribute of a `Discount`.
//  It must be enforced that the `percentage` attribute is a float between 0. and 1.
//  Then define two subclasses:
//  - `SeasonalDiscount` that inherits from `Discount` with two additional attributes, `to` and `from_`.
//    `from_` is a datetime object that represents the start of the discount period.
//    `to` is a datetime object that represents the end of the discount period.
//     Both `from_` and `to` should be accessible and modifiable.
//     The class should enforce that `from` is before `to`.
//  - `CappedDiscount` that inherits from `Discount` with an additional attribute `cap`.
//    `cap` is a float that represents the maximum discount (in absolute value) that can be applied.
//    It should be possible to access and modify the `cap` attribute.
//    The class should enforce that `cap` is a non-zero positive float.
//
// All classes should have a method named `apply` that takes a price (float) as input and
// returns the discounted price.
// `SeasonalDiscount` should raise an `ExpiredDiscount` exception if `apply` is called but
// the current date is outside the discount period.
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;

#[pyclass(subclass)]
struct Discount {
    #[pyo3(get)]
    percentage: f64,
}

#[pymethods]
impl Discount {
    #[new]
    fn new(percentage: f64) -> PyResult<Self> {
        if percentage < 0.0 || percentage > 1.0 {
            Err(PyValueError::new_err("Percentage must be between 0 and 1"))
        } else {
            Ok(Discount {
                percentage: percentage,
            })
        }
    }

    #[setter]
    fn percentage(&mut self, percentage: f64) -> PyResult<()> {
        if percentage < 0.0 || percentage > 1.0 {
            Err(PyValueError::new_err("Percentage must be between 0 and 1"))
        } else {
            Ok(())
        }
    }

    fn apply(&self, price: f64) -> f64 {
        price * (1.0 - self.percentage)
    }
}

#[pyclass(extends=PyException)]
struct ExpiredDiscount {}

#[pyclass(extends=Discount)]
struct SeasonalDiscount {
    //from_: ,
    //to: ,
}

#[pyclass(extends=Discount)]
struct CappedDiscount {}

#[pymodule]
fn outro2(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Discount>()?;
    m.add_class::<ExpiredDiscount>()?;
    m.add_class::<SeasonalDiscount>()?;
    m.add_class::<CappedDiscount>()?;
    Ok(())
}
