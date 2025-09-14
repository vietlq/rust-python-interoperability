// https://rust-exercises.com/rust-python-interop/02_classes/05_inheritance.html
// Whenever you initialize a subclass, you need to make sure that the parent class is initialized first.
// We start by calling Parent::new to create an instance of the parent class.
// We then initialize Child, via Self { age }.
// We then use PyClassInitializer to return both the parent and child instances together.
// Limitations
// pyo3 supports two kinds of superclasses:
// A Python class defined in Rust, via #[pyclass]
// A Python built-in class, like PyDict or PyList
// It currently doesn't support using a custom Python class as the parent class for a class defined in Rust.
//
// TODO: Define a base class named `Person`, with `first_name` and `last_name` attributes, set
//  by the constructor. It should be possible to access the `first_name` and `last_name` attributes
//  of a `Person`.
//  `Person` should also have a method named `full_name` that returns the full name of the person.
//  Then define a subclass named `Employee` that inherits from `Person` and adds an
//  unsigned integer `id` attribute and a constructor that sets the `id` attribute.
//  It should be possible to access the `first_name`, `last_name` and `id`
//  attributes of an `Employee`.
use pyo3::prelude::*;

// We must use pyclass(subclass) to allow subclassing,
// otherwise compiler will complain.
#[pyclass(subclass)]
struct Person {
    #[pyo3(get)]
    first_name: String,

    #[pyo3(get)]
    last_name: String,
}

#[pymethods]
impl Person {
    #[new]
    fn new(first_name: String, last_name: String) -> Self {
        Person {
            first_name,
            last_name,
        }
    }

    fn full_name(&self) -> String {
        format!("{} {}", self.first_name.clone(), self.last_name.clone())
    }
}

// We need to use pyclass(extends=ParentClassName)
#[pyclass(subclass, extends=Person)]
struct Employee {
    #[pyo3(get)]
    id: u64,
}

#[pymethods]
impl Employee {
    // Note that we must return PyClassInitializer<Self> for child classes
    #[new]
    fn new(first_name: String, last_name: String, id: u64) -> PyClassInitializer<Self> {
        // We need to create the parent first
        let parent = Person::new(first_name, last_name);
        // Then we create the child
        let child = Self { id };
        // Finally we return using PyClassInitializer to join them into a single Python object
        PyClassInitializer::from(parent).add_subclass(child)
    }
}

#[pyclass(extends=Employee)]
struct Manager {
    #[pyo3(get, set)]
    title: String,
}

#[pymethods]
impl Manager {
    #[new]
    fn new(
        first_name: String,
        last_name: String,
        id: u64,
        title: String,
    ) -> PyClassInitializer<Self> {
        let parent = Employee::new(first_name, last_name, id);
        let child = Manager { title };
        PyClassInitializer::from(parent).add_subclass(child)
    }
}

#[pymodule]
fn inheritance(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Person>()?;
    m.add_class::<Employee>()?;
    m.add_class::<Manager>()?;
    Ok(())
}
