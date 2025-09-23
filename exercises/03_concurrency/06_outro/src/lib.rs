mod thread_crawler;
mod utils;

use anyhow::Result;
use pyo3::{prelude::*, types::PySet};
use std::collections::HashSet;
use tracing::{debug, error, info};
use tracing_subscriber;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[pyfunction]
/// Given a starting URL (`start_from`), discover all the URLs *on the same domain*
/// that can be reached by following links from the starting URL.
///
/// The discovered URLs should be inserted into the `site_map` set provided as an argument.
///
/// # Constraints
///
/// ## GIL
///
/// You should, as much as possible, avoid holding the GIL.
/// Try to scope the GIL to the smallest possible block of code—e.g. when touching `site_map`.
///
/// ## Threads
///
/// The program should use as many threads as there are available cores on the machine.
///
/// ## Invalid URLs
///
/// If a URL is invalid (e.g. it's malformed or it returns a 404 status code), ignore it.
///
/// ## External URLs
///
/// Do not follow links to external websites. Restrict your search to the domain of the
/// starting URL.
///
/// ## Anchors and Query Parameters
///
/// Ignore anchors and query parameters when comparing URLs.
/// E.g. `http://example.com` and `http://example.com#section` should be considered the same URL,
/// and normalizing them to `http://example.com` is the expected approach.
///
/// # Tooling
///
/// We recommend using the following crates to help you with this exercise:
///
/// - `ureq` for making HTTP requests (https://crates.io/crates/ureq)
/// - `scraper` for parsing HTML and extracting links (https://crates.io/crates/scraper)
/// - `url` for parsing URLs (https://crates.io/crates/url)
/// - `std`'s `sync` and `thread` modules for synchronization primitives.
///
/// Feel free to pull in any other crates you think might be useful.
/// If your approach is channel-based, you might want to use the `crossbeam` crate too.
pub fn site_map<'py>(
    python: Python<'py>,
    start_from: String,
    site_map: Bound<'py, PySet>,
) -> PyResult<()> {
    // Initialize tracing with file/line info and thread details
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(
            fmt::layer()
                .with_file(true) // Show file names
                .with_line_number(true) // Show line numbers
                .with_thread_ids(true) // Show thread IDs
                .with_thread_names(true) // Show thread names
                .with_target(true), // Show module target
        )
        .init();

    // Build site map without GIL
    let result = python.allow_threads(|| -> Result<HashSet<String>> {
        let max_links = 200;
        let max_wait_time_s = 30;
        let max_concurrency = 8;

        thread_crawler::ThreadCrawler::build_site_map(
            &start_from,
            max_links,
            max_wait_time_s,
            max_concurrency,
        )
    });

    // Convert the Result to PyResult
    result
        .map(|rs_site_map| {
            // Enrich the python site map from rust site map
            for link in rs_site_map {
                println!("{}", link);
                let _ = site_map.add(link).unwrap();
            }

            ()
        })
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

#[pymodule]
fn outro3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(site_map, m)?)?;
    Ok(())
}
