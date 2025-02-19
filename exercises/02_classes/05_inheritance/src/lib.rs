// TODO: Define a base class named `Person`, with `first_name` and `last_name` attributes, set
//  by the constructor. It should be possible to access the `first_name` and `last_name` attributes
//  of a `Person`.
//  `Person` should also have a method named `full_name` that returns the full name of the person.
//  Then define a subclass named `Employee` that inherits from `Person` and adds an
//  unsigned integer `id` attribute and a constructor that sets the `id` attribute.
//  It should be possible to access the `first_name`, `last_name` and `id`
//  attributes of an `Employee`.
use pyo3::prelude::*;

/**
 * https://rust-exercises.com/rust-python-interop/02_classes/05_inheritance
 *
 * Limitations

pyo3 supports two kinds of superclasses:

    A Python class defined in Rust, via #[pyclass]
    A Python built-in class, like PyDict or PyList

It currently doesn't support using a custom Python class as the parent class for a class defined in Rust.

#[pyclass(subclass)]
struct Parent {
    name: String,
}

#[pymethods]
impl Parent {
    #[new]
    fn new(name: String) -> Self {
        Parent { name }
    }

    fn greet(&self) {
        println!("Hello, {}!", self.name);
    }
}

#[pyclass(extends=Parent)]
struct Child {
    age: u8,
}

#[pymethods]
impl Child {
    #[new]
    fn new(name: String, age: u8) -> PyClassInitializer<Self> {
        let parent = Parent::new(name);
        let child = Self { age };
        PyClassInitializer::from(parent).add_subclass(child)
    }
}

Nested inheritance

PyClassInitializer can be used to build arbitrarily deep inheritance hierarchies.
For example, if Child had its own subclass, you could call add_subclass again to
add yet another subclass to the chain.

#[pyclass(extends=Child)]
struct Grandchild {
    hobby: String,
}

#[pymethods]
impl Grandchild {
    #[new]
    fn new(name: String, age: u8, hobby: String) -> PyClassInitializer<Self> {
        let child = Child::new(name, age);
        let grandchild = Self { hobby };
        PyClassInitializer::from(child).add_subclass(grandchild)
    }
}
*/

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
        vec![self.first_name.clone(), self.last_name.clone()].join(" ")
    }
}

#[pyclass(extends=Person)]
struct Employee {
    #[pyo3(get)]
    id: u64,
}

#[pymethods]
impl Employee {
    #[new]
    fn new(first_name: String, last_name: String, id: u64) -> PyClassInitializer<Self> {
        let person = Person::new(first_name, last_name);
        let employee = Employee { id };
        PyClassInitializer::from(person).add_subclass(employee)
    }
}

#[pymodule]
fn inheritance(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Person>()?;
    m.add_class::<Employee>()?;
    Ok(())
}
