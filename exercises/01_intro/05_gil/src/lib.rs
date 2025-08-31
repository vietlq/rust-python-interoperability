use pyo3::{prelude::*, types::PyList};

#[pyfunction]
// TODO: Use `PyList` instead of `Vec<u64>` as the input type. Panic on errors, for now.
// You might find this useful: https://pyo3.rs/v0.22.0/conversions/traits#extract-and-the-frompyobject-trait
fn print_number_list(pxs: &Bound<'_, PyList>) {
    let xs: Vec<u64> = pxs.extract().unwrap();

    for x in xs {
        println!("{}", x)
    }
}

#[pymodule]
fn gil(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(print_number_list, m)?)?;
    Ok(())
}
