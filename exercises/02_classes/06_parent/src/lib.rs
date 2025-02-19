// TODO: Define a base class named `Account`, with a floating point `balance` property.
//  Then define a subclass named `AccountWithHistory`.
//  `AccountWithHistory` adds a `history` attribute: every time the `balance` is modified,
//  the old balance is stored in the `history` list. `history` can be accessed but not modified
//  directly. The `history` list should be initialized as an empty list.
use pyo3::prelude::*;

/**
 * https://rust-exercises.com/rust-python-interop/02_classes/06_parent
 *
 * PyRef and PyRefMut

PyRef is for immutable references, but what if we need to modify the parent class?
In that case, we can use PyRefMut, which is a mutable reference.

as_super to the rescue

We need a way, in Rust, to access the fields and methods of the parent class from the child class.
This can be done using another one of pyo3's smart pointers: PyRef.

#[pymethods]
impl Child {
    // [...]

    fn greet(self_: PyRef<'_, Self>) {
        todo!()
    }
}

PyRef represents an immutable reference to the Python object.
It allows us, in particular, to call the as_super method, which returns a reference to the parent class.

#[pymethods]
impl Child {
    // [...]

    fn greet(self_: PyRef<'_, Self>) {
        // This is now a reference to a `Parent` instance!
        let parent = self_.as_super();
        println!("Hi, I'm {} and I'm {} years old!", parent.name, self_.age);
    }
}

Now we can access the name field from the parent class, and the age field from the child class.
*/

#[pyclass(subclass)]
struct Account {
    #[pyo3(get)]
    balance: i64,
}

#[pymethods]
impl Account {
    #[new]
    fn new(balance: i64) -> Self {
        Account { balance }
    }

    #[setter]
    fn set_balance(&mut self, new_balance: i64) {
        self.balance = new_balance;
    }
}

#[pyclass(extends=Account)]
struct AccountWithHistory {
    #[pyo3(get)]
    history: Vec<i64>,
}

#[pymethods]
impl AccountWithHistory {
    #[new]
    fn new(balance: i64) -> PyClassInitializer<Self> {
        let account = Account::new(balance);
        let history: Vec<i64> = vec![];
        let account_with_history = AccountWithHistory { history };
        PyClassInitializer::from(account).add_subclass(account_with_history)
    }

    // NOTE: When overriding set_balance, we have to repeat the getter for balance
    #[getter]
    fn balance(self_: PyRef<'_, Self>) -> i64 {
        self_.as_super().balance
    }

    #[setter]
    fn set_balance(mut self_: PyRefMut<'_, Self>, new_balance: i64) {
        let parent = self_.as_super();
        let old_balance = parent.balance;
        // Stop referencing `parent` ASAP
        parent.balance = new_balance;
        // Never put self_ and parent in the same statement after assignment
        self_.history.push(old_balance);
    }
}

#[pymodule]
fn parent(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Account>()?;
    m.add_class::<AccountWithHistory>()?;
    Ok(())
}
