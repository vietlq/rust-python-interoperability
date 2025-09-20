use anyhow::{Context, Result};
use pyo3::{prelude::*, types::PySet};
use reqwest;
use scraper;
use std::collections::HashSet;
use url::Url;

macro_rules! log_info {
    ($($arg:tt)*) => {
        println!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
    };
}

macro_rules! log_error {
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
    // Get the seed site map so we don't fetch the links in the site_map again
    let seed_site_map: HashSet<String> = site_map.extract::<HashSet<String>>()?;

    // Build site map without GIL
    let result = python.allow_threads(|| -> Result<HashSet<String>> {
        build_site_map(&start_from, &seed_site_map)
    });

    // Convert the Result to PyResult
    result
        .map(|rs_site_map| {
            // Enrich the python site map from rust site map
            for link in rs_site_map {
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

fn build_site_map(start_from: &String, seed_site_map: &HashSet<String>) -> Result<HashSet<String>> {
    let mut rs_site_map = seed_site_map.clone();
    println!("start_from = {}", start_from);

    let orig_domain =
        get_orig_domain(start_from).ok_or_else(|| log_error!("Invalid URL {}", start_from))?;
    println!("orig_domain = {}", &orig_domain);

    let response = reqwest::blocking::get(start_from)
        .map_err(|e| log_error!("Could not get the URL {}. Error: {:?}", start_from, e))?;
    let html_text = response.text()?;
    let html_doc = scraper::Html::parse_document(&html_text);
    let selector =
        scraper::Selector::parse("a").map_err(|e| log_error!("Bad selector: {:?}", e))?;

    for link_obj in html_doc.select(&selector) {
        let link = link_obj
            .value()
            .attr("href")
            .ok_or_else(|| log_error!("Invalid attribute href: {:?}", link_obj))?;
        println!("{}", link);

        rs_site_map.insert(link.to_string());
    }

    for link in &rs_site_map {
        println!("{}", &link);
    }

    Ok(rs_site_map)
}

fn get_orig_domain(url_str: &str) -> Option<String> {
    let parsed_url = Url::parse(&url_str).unwrap();
    Some(parsed_url.host_str().unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_orig_domain() {
        assert_eq!(
            get_orig_domain("http://a.b.c.com"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_orig_domain("http://a.b.c.com/"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_orig_domain("http://a.b.c.com/d"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_orig_domain("https://a.b.c.com/d"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_orig_domain("https://a.b.c.com/d/"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_orig_domain("https://a.b.c.com/d/?"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_orig_domain("https://a.b.c.com/d/?e"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_orig_domain("https://a.b.c.com/d/?e="),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_orig_domain("https://a.b.c.com/d/?e=f"),
            Some("a.b.c.com".to_string())
        );
        assert_eq!(
            get_orig_domain("https://a.b.c.com/d/?e=f#g=h"),
            Some("a.b.c.com".to_string())
        );
    }
}
