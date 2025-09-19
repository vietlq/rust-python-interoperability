use anyhow::{Context, Result};
use pyo3::{prelude::*, types::PySet};
use reqwest;
use scraper;
use std::collections::HashSet;
use url::Url;

macro_rules! log_with_location {
    ($($arg:tt)*) => {
        println!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
    };
}

macro_rules! error_with_location {
    ($($arg:tt)*) => {
        anyhow::anyhow!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
    };
}

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
    let rs_site_map: HashSet<String> = site_map.extract::<HashSet<String>>()?;

    let result = python.allow_threads(|| -> Result<()> {
        println!("start_from = {}", &start_from);

        let host_url =
            get_host_url(&start_from).ok_or_else(|| error_with_location!("Invalid URL"))?;
        println!("host_url = {}", &host_url);

        let response = reqwest::blocking::get(&start_from).map_err(|e| {
            error_with_location!("Could not get the URL {}. Error: {:?}", &start_from, e)
        })?;
        let html_text = response.text()?;
        let html_doc = scraper::Html::parse_document(&html_text);
        let selector = scraper::Selector::parse("a")
            .map_err(|e| error_with_location!("Bad selector: {:?}", e))?;

        for link in html_doc.select(&selector) {
            println!(
                "{}",
                link.value()
                    .attr("href")
                    .context("Invalid attribute href")?
            )
        }

        for link in &rs_site_map {
            println!("{}", &link);
        }

        Ok(())
    });

    // Convert the Result to PyResult
    result.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

#[pymodule]
fn outro3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(site_map, m)?)?;
    Ok(())
}

fn get_host_url(url_str: &str) -> Option<String> {
    let parsed_url = Url::parse(&url_str).unwrap();
    Some(parsed_url.host_str().unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_host_url() {
        assert_eq!(
            get_host_url("http://a.b.c.com"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_host_url("http://a.b.c.com/"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_host_url("http://a.b.c.com/d"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_host_url("https://a.b.c.com/d"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_host_url("https://a.b.c.com/d/"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_host_url("https://a.b.c.com/d/?"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_host_url("https://a.b.c.com/d/?e"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_host_url("https://a.b.c.com/d/?e="),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_host_url("https://a.b.c.com/d/?e=f"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_host_url("https://a.b.c.com/d/?e=f#g=h"),
            Some("a.b.c.com".to_string())
        );
    }
}
