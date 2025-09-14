use pyo3::{prelude::*, types::PyString};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Use `std::thread::scope` to spawn `n_threads` threads to count words in parallel.
///
/// Rely on:
/// - `word_count_chunk` to count words in each chunk
/// - `split_into_chunks` to split the text into `n_threads` chunks
///
/// If you've never used `std::thread::scope` before, you can find more information here:
/// https://rust-exercises.com/100-exercises/07_threads/04_scoped_threads.html
#[pyfunction]
fn word_count(text: Bound<'_, PyString>, n_threads: usize) -> PyResult<usize> {
    if n_threads == 0 {
        panic!("Number of threads 'n_threads' must be greater than 0");
    }

    // Get a Rust view (&str) over the Python string
    // This may fail if the string contains invalid UTF-8
    // We go down this route, rather than asking for a `&str`
    // directly as an argument, to avoid an extra copy of the string
    let text = text.to_str()?;

    // We use std::thread::scope so that child threads never outlive their parent
    let chunks = split_into_chunks(text, n_threads);
    // We don't need `mut` here, `Arc` should suffice.
    // It's recommended to use AtomicUsize instead of usize, because usize is immutable.
    let result: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    std::thread::scope(|scope| {
        for chunk in chunks {
            // Question why we can get away without `move` here?
            scope.spawn(/* move */ || {
                // We must clone Arc reference for each scoped thread.
                // It will not create new result, only reference count to it.
                // This is actually cheaper than using mutex.
                let result_clone = Arc::clone(&result);
                let local_result = word_count_chunk(chunk);
                result_clone.fetch_add(local_result, Ordering::Relaxed);
            });
        }
    });

    let final_result = result.load(Ordering::Relaxed);

    Ok(final_result)
}

/// Count words in a single chunk of text.
fn word_count_chunk(chunk: &str) -> usize {
    chunk.split_whitespace().count()
}

/// Splits a string into `n` chunks, ensuring splits occur at whitespace.
fn split_into_chunks(text: &str, n: usize) -> Vec<&str> {
    if n == 0 {
        panic!("Number of chunks 'n' must be greater than 0");
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    let avg_length = text.len() / n;

    for _ in 0..n {
        if start >= text.len() {
            break; // No more content to split
        }

        // Tentative end index
        let mut end = (start + avg_length).min(text.len());

        // Adjust end to nearest whitespace
        while end < text.len() && !text[end..].starts_with(char::is_whitespace) {
            end += 1;
        }

        // If we hit the end of the string, take the rest
        if end >= text.len() {
            chunks.push(&text[start..]);
            break;
        }

        // Add the chunk and move the start index forward
        chunks.push(text[start..end].trim());
        start = end + 1; // Move past the whitespace
    }

    chunks
}

#[pymodule]
fn gil2(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(word_count, m)?)?;
    Ok(())
}
