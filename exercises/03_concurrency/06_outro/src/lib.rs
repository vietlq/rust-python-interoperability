use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver, Sender};
use dashmap::DashSet;
use pyo3::{prelude::*, types::PySet};
use reqwest;
use scraper;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
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
    // Build site map without GIL
    let result = python.allow_threads(|| -> Result<HashSet<String>> {
        let max_links = 200;
        let max_wait_time_s = 30;
        let max_concurrency = 8;
        build_site_map(&start_from, max_links, max_wait_time_s, max_concurrency)
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

fn build_site_map(
    start_from: &String,
    max_links: usize,
    max_wait_time_s: u64,
    max_concurrency: u64,
) -> Result<HashSet<String>> {
    println!("start_from = {}", start_from);

    let orig_domain =
        get_orig_domain(start_from).ok_or_else(|| log_error!("Invalid URL {}", start_from))?;
    println!("orig_domain = {}", &orig_domain);

    let mut thread_handles = Vec::new();

    let rs_site_map: Arc<DashSet<String>> = Arc::new(DashSet::new());
    let visited: Arc<DashSet<String>> = Arc::new(DashSet::new());
    let start_from = start_from.clone();
    let duration = Duration::from_secs(max_wait_time_s);
    let (sender, receiver) = unbounded();
    sender.send(start_from)?;

    for tid in 0..max_concurrency {
        let orig_domain = orig_domain.clone();
        let sender = sender.clone();
        let receiver = receiver.clone();
        let rs_site_map = rs_site_map.clone();
        let visited = visited.clone();
        let max_links = max_links.clone();
        let duration = duration.clone();

        let handle = thread::spawn(move || {
            let _ = extract_links_from(
                tid,
                &orig_domain,
                sender,
                receiver,
                rs_site_map,
                visited,
                max_links,
                duration,
            );
        });
        thread_handles.push(handle);
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }

    let mut result: HashSet<String> = HashSet::new();
    rs_site_map.iter().for_each(|x| {
        result.insert(x.to_string());
    });
    Ok(result)
}

fn extract_links_from(
    tid: u64,
    orig_domain: &str,
    sender: Sender<String>,
    receiver: Receiver<String>,
    rs_site_map: Arc<DashSet<String>>,
    visited: Arc<DashSet<String>>,
    max_links: usize,
    max_wait_time: Duration,
) -> Result<()> {
    loop {
        if rs_site_map.len() >= max_links {
            return Ok(());
        }

        if let Ok(curr_link) = receiver.recv_timeout(max_wait_time) {
            if rs_site_map.len() >= max_links {
                return Ok(());
            }

            if visited.contains(&curr_link) {
                continue;
            }

            log_info!("[Thread {}] processing the link {}", tid, curr_link);

            let mut sub_site_map: HashSet<String> = HashSet::new();
            let response = reqwest::blocking::get(&curr_link)
                .map_err(|e| log_error!("Could not get the URL {}. Error: {:?}", &curr_link, e))?;
            let html_text = response.text()?;
            let html_doc = scraper::Html::parse_document(&html_text);

            // Extract src from <iframe>
            extract_links_from_element(
                tid,
                &html_doc,
                &curr_link,
                orig_domain,
                &mut sub_site_map,
                "iframe",
                "src",
            )?;
            // Extract href from <a>
            extract_links_from_element(
                tid,
                &html_doc,
                &curr_link,
                orig_domain,
                &mut sub_site_map,
                "a",
                "href",
            )?;

            for new_link in sub_site_map {
                if !visited.contains(&new_link) {
                    log_info!("[Thread {}] queuing the link {}", tid, &new_link);
                    sender.send(new_link.clone())?;
                }

                rs_site_map.insert(new_link);
            }

            visited.insert(curr_link.to_string());
            log_info!("[Thread {}] **processed** the link {}", tid, curr_link);
        } else {
            return Ok(());
        }
    }
}

fn extract_links_from_element(
    tid: u64,
    html_doc: &scraper::Html,
    curr_link: &str,
    orig_domain: &str,
    sub_site_map: &mut HashSet<String>,
    element: &str,
    attr: &str,
) -> Result<()> {
    log_info!(
        "[Thread {}] extracting links from <{} {}='...'>",
        tid,
        element,
        attr
    );

    let selector =
        scraper::Selector::parse(element).map_err(|e| log_error!("Bad selector: {:?}", e))?;

    for link_obj in html_doc.select(&selector) {
        let link = link_obj
            .value()
            .attr(attr)
            .ok_or_else(|| log_error!("Invalid attribute {}: {:?}", attr, link_obj))?;

        let resolved_link = resolve_link(&curr_link, link)
            .ok_or_else(|| log_error!("Could not resolve link {}", link))?;

        let link_domain =
            get_orig_domain(&resolved_link).ok_or_else(|| log_error!("Bad link: {}", link))?;
        log_info!("[Thread {}] resolved_link = {}", tid, resolved_link);

        if link_domain == *orig_domain {
            sub_site_map.insert(resolved_link.to_string());
        }

        log_info!(
            "[Thread {}] finished extracting links from <a href='...'>",
            tid
        );
    }

    Ok(())
}

fn get_orig_domain(url_str: &str) -> Option<String> {
    Url::parse(url_str).ok()?.host_str().map(String::from)
}

fn resolve_link(base_url: &str, href: &str) -> Option<String> {
    // Skip non-HTTP links
    if href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("javascript:")
        || href.starts_with("#")
    {
        return None;
    }

    // If it's already a full URL, return as-is if it's HTTP(S)
    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(href.to_string());
    }

    // Try to resolve relative URL
    if let Ok(base) = Url::parse(base_url) {
        if let Ok(full_url) = base.join(href) {
            // Only return HTTP(S) URLs
            if full_url.scheme() == "http" || full_url.scheme() == "https" {
                return Some(full_url.to_string());
            }
        }
    }

    None
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
