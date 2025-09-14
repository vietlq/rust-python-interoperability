// https://rust-exercises.com/rust-python-interop/02_classes/06_parent.html
//
// TODO: Define a base class named `Account`, with a floating point `balance` property.
//  Then define a subclass named `AccountWithHistory`.
//  `AccountWithHistory` adds a `history` attribute: every time the `balance` is modified,
//  the old balance is stored in the `history` list. `history` can be accessed but not modified
//  directly. The `history` list should be initialized as an empty list.
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyList;

#[pyclass(subclass)]
struct Account {
    #[pyo3(get, set)]
    balance: f64,
}

#[pymethods]
impl Account {
    #[new]
    fn new(balance: f64) -> PyResult<Self> {
        if balance < 0.0 {
            return Err(PyValueError::new_err(
                "Balance must not be negative on creation",
            ));
        }

        Ok(Self { balance })
    }
}

#[pyclass(extends=Account)]
struct AccountWithHistory {
    #[pyo3(get)]
    history: Py<PyList>,
}

#[pymethods]
impl AccountWithHistory {
    #[new]
    fn new(py: Python, balance: f64) -> PyResult<PyClassInitializer<Self>> {
        let parent = Account::new(balance)?;
        // Use can use PyList::empty(py).into(), but using unbind() is explicit
        // and faster. Note that into() involves traits and still uses unbind()
        // behind the scene.
        let history: Py<PyList> = PyList::empty(py).unbind();
        let child = AccountWithHistory { history };
        Ok(PyClassInitializer::from(parent).add_subclass(child))
    }

    // We must use PyRef/PyRefMut to access Python `self`.
    // Then we must use as_super() to get the instance of the parent class.
    // We must use non-colliding function names and refer back to `balance`.
    #[setter(balance)]
    fn set_balance(mut self_: PyRefMut<'_, Self>, py: Python, balance: f64) {
        let parent = self_.as_super();

        let curr_balance = parent.balance;
        parent.balance = balance;
        let _ = self_.history.bind(py).append(curr_balance);
    }

    // We must use non-colliding function names and refer back to `balance`.
    // When overriding setter from the parent class, we must override the getter too.
    #[getter(balance)]
    fn get_balance(self_: PyRef<'_, Self>) -> f64 {
        self_.as_super().balance
    }
}

#[pymodule]
fn parent(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Account>()?;
    m.add_class::<AccountWithHistory>()?;
    Ok(())
}
