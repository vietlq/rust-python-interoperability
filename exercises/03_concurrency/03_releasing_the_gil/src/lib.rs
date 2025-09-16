use pyo3::prelude::*;

/*
* https://rust-exercises.com/rust-python-interop/03_concurrency/03_releasing_the_gil.html
* Releasing the GIL

What happens to our Python code when it calls a Rust function?
It waits for the Rust function to return:

 Time -->

          +------------+--------------------+------------+--------------------+
 Python:  |  Execute   | Call Rust Function |    Idle    |  Resume Execution  |
          +------------+--------------------+------------+--------------------+
                                 │                                ▲
                                 ▼                                │
          +------------+--------------------+------------+--------------------+
 Rust:    |    Idle    |       Idle         |  Execute   |  Return to Python  |
          +------------+--------------------+------------+--------------------+

The schema doesn't change even if the Rust function is multithreaded:

 Time -->

          +------------+--------------------+-------------------+--------------------+
 Python:  |  Execute   | Call Rust Function |       Idle        |  Resume Execution  |
          +------------+--------------------+-------------------+--------------------+
                                 │                                        ▲
                                 ▼                                        │
          +------------+--------------------+-------------------+--------------------+
 Rust:    |    Idle    |       Idle         | Execute Thread 1  |  Return to Python  |
          |            |                    | Execute Thread 2  |                    |
          +------------+--------------------+-------------------+--------------------+

It begs the question: can we have Python and Rust code running concurrently?
Yes!

The focus point, once again, is the GIL.
Python access must be serialized

The GIL's job is to serialize all interactions with Python objects.
On the pyo3 side, this is modeled by the Python<'py> token:
you can only get an instance of Python<'py> if you're holding the GIL.
Going further, you can only interact with Python objects via smart pointers
like Borrowed<'py, T> or Owned<'py, T>, which internally hold a Python<'py> instance.
There's no way around it: any interaction with Python objects must be
serialized.

But, here's the kicker: not all Rust code needs to interact
with Python objects!

Using Python::allow_threads

Python::allow_threads

For example, consider a Rust function that calculates the nth Fibonacci number:

#[pyfunction]
fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let tmp = a;
        a = b;
        b = tmp + b;
    }
    a
}

There's no Python object in sight! We're just offloading a computation to Rust.
In principle, we could spawn a thread to run this function while the main thread continues executing Python code:

from threading import Thread

def other_work():
    print("I'm doing other work!")

t = Thread(target=fibonacci, args=(10,))
t.start()
other_work()
t.join()

As it stands, other_work and fibonacci will not be run in parallel:
our fibonacci routine is still holding the GIL, even though it doesn't need it.
We can fix it by explicitly releasing the GIL:

#[pyfunction]
fn fibonacci(py: Python<'_>, n: u64) -> u64 {
    py.allow_threads(|| {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..n {
            let tmp = a;
            a = b;
            b = tmp + b;
        }
        a
    })
}

Python::allow_threads releases the GIL while executing the closure passed to it.
This frees up the Python interpreter to run other Python code, such as the other_work function in our example, while the Rust thread is busy calculating the nth Fibonacci number.

Using the same line diagram as before, we have the following:

 Time -->

          +------------+--------------------+-------------------+--------------------+
 Python:  |  Execute   | Call Rust Function |    other_work()   |      t.join()      |
          +------------+--------------------+-------------------+--------------------+
                                 │                                        ▲
                                 ▼                                        │
          +------------+--------------------+-------------------+--------------------+
 Rust:    |    Idle    |       Idle         |    fibonacci(n)   |  Return to Python  |
          +------------+--------------------+-------------------+--------------------+
                                                     ▲
                                                     │
                                            Python and Rust code
                                          running concurrently here

Ungil

Python::allow_threads is only sound if the closure doesn't interact with Python objects.
If that's not the case, we end up with undefined behavior: Rust code touching
Python objects while the Python interpreter is running other Python code,
assuming nothing else is happening to those objects thanks to the GIL.
A recipe for disaster!

It'd be ideal to rely on the type system to enforce this constraint for us at
compile-time, in true Rust fashion—"if it compiles, it's safe."
pyo3 tries to follow this principle with the Ungil marker trait: only types
that are safe to access without the GIL can implement Ungil. The trait is then
used to constrain the arguments of Python::allow_threads:

pub fn allow_threads<T, F>(self, f: F) -> T
where
    F: Ungil + FnOnce() -> T,
    T: Ungil,
{
    // ...
}

Unfortunately, Ungil is not perfect. On stable Rust, it leans on the Send trait,
but that allows for some unsafe interactions with Python objects.
The tracking is more precise on nightly Rust1, but it doesn't catch every
possible misuse of Python::allow_threads.

My recommendation: if you're using Python::allow_threads, trigger an
additional run of your CI pipeline using the nightly Rust compiler to
catch more issues. On top of that, review your code carefully.
* */

#[pyfunction]
// Modify this function to release the GIL while computing the nth prime number.
fn nth_prime(py: Python<'_>, n: u64) -> u64 {
    // We must release GIL explicitly by passing `py: Python<'_>` as an argument
    // and then using `py.allow_threads(|| {...})`.
    py.allow_threads(|| {
        // This code runs in a native thread in parallel and does not need GIL.
        // Python can switch to other threads and when this is done,
        // GIL will be acquired and the result passed back to Python.
        let mut count = 0;
        let mut num = 2; // Start checking primes from 2
        while count < n {
            if is_prime(num) {
                count += 1;
            }
            num += 1;
        }
        num - 1 // Subtract 1 because we increment after finding the nth prime
    })
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..=(n as f64).sqrt() as u64 {
        if n % i == 0 {
            return false;
        }
    }
    true
}

#[pymodule]
fn release(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(nth_prime, m)?)?;
    Ok(())
}
