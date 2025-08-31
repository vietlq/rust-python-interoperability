use pyo3::prelude::*;

#[pyfunction]
// TODO: Define a function that takes as input a vector of unsigned integers
//  and prints each number in the list.
fn print_number_list(xs: Vec<u64>) {
    // This works but we do not want to keep the variables
    /*
    _ = xs.iter().map(|x| println!("{}", x)).collect::<Vec<_>>();
    ()
    */

    // This solution is simpler and uses less memory
    for x in xs {
        println!("{}", x)
    }
}

#[pymodule]
fn arguments(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(print_number_list, m)?)?;
    Ok(())
}
