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
        // NOTE: Use bind(py) here to use Python API (list.append) and for GIL to allocate memory
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

/*
Q: Do I use GIL only for bind(py) or for the whole method?
Does passing  py to the method means holding GIL for
the whole duration of that method?

A: Great question! This touches on PyO3's GIL management. Let me clarify:

## GIL Token vs GIL Lock

The `py: Python` parameter is a **GIL token**, not the GIL lock itself.
It's proof that the current thread holds the GIL, but
passing it around doesn't extend the GIL duration.

## GIL Duration

The GIL is held for the **entire duration** of:
1. Python calling your Rust function
2. Any `Python::with_gil(|py| { ... })` block

## Your specific case

```rust
#[pymethods]
impl AccountWithHistory {
    // The GIL is held for the ENTIRE duration of this method
    fn add_transaction(&self, py: Python, amount: f64) -> PyResult<()> {
        // GIL is already held when this method starts

        // This just "borrows" the Py<PyList> with the existing GIL token
        let bound_history = self.history.bind(py);  // No new GIL acquisition

        // This operation uses the existing GIL
        bound_history.append(amount)?;

        // GIL is released when method returns to Python
        Ok(())
    }
}
```

## Different patterns:

### Pattern 1: Python calls your method (GIL held throughout)
```rust
#[pymethods]
impl AccountWithHistory {
    fn method1(&self, py: Python) -> PyResult<()> {
        // GIL held for entire method duration
        self.history.bind(py).append(1.0)?;

        // Even if you call other operations:
        let list2 = PyList::empty(py);
        list2.append(2.0)?;

        // GIL still held until method returns
        Ok(())
    }
}
```

### Pattern 2: You acquire GIL yourself
```rust
impl AccountWithHistory {
    fn method_from_rust(&self, amount: f64) -> PyResult<()> {
        Python::with_gil(|py| {
            // GIL acquired here and held for this block
            self.history.bind(py).append(amount)?;
            // Multiple operations all use the same GIL acquisition
            self.history.bind(py).append(amount * 2.0)?;
            Ok(())
        }) // GIL released here
    }
}
```

### Pattern 3: Multiple GIL acquisitions (inefficient)
```rust
impl AccountWithHistory {
    fn inefficient_method(&self, amount: f64) -> PyResult<()> {
        // BAD: Acquiring GIL multiple times
        Python::with_gil(|py| {
            self.history.bind(py).append(amount)
        })?;

        Python::with_gil(|py| {
            self.history.bind(py).append(amount * 2.0)
        })?;

        Ok(())
    }
}
```

## Key insights:

1. **`py: Python` parameter**: The GIL is held for the entire method when Python calls it
2. **`.bind(py)`**: Just creates a temporary view, doesn't acquire/release GIL
3. **Performance**: All operations within one method call share the same GIL acquisition
4. **Multiple operations**: You can do many `.bind(py)` calls efficiently within one method

## Best practices:

```rust
#[pymethods]
impl AccountWithHistory {
    fn efficient_method(&self, py: Python, amounts: Vec<f64>) -> PyResult<()> {
        // Bind once, use many times (more efficient)
        let history = self.history.bind(py);

        for amount in amounts {
            history.append(amount)?;
        }

        // All operations used the same GIL held for this method
        Ok(())
    }

    // vs less efficient:
    fn less_efficient_method(&self, py: Python, amounts: Vec<f64>) -> PyResult<()> {
        for amount in amounts {
            // Binding repeatedly (still works, but creates more temporary objects)
            self.history.bind(py).append(amount)?;
        }
        Ok(())
    }
}
```

**Answer**: When Python calls your method with `py: Python`,
the GIL is held for the **entire method duration**.
The `.bind(py)` operation just creates a
temporary `Bound` reference and doesn't involve GIL acquisition/release.
* */
