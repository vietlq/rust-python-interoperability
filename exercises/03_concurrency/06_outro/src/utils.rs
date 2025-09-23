use anyhow::{anyhow, Result};
use scraper;
use std::collections::HashSet;
use tracing::info;
use url::Url;

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        anyhow!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
    };
}

pub fn extract_links_from_element(
    tid: u64,
    html_doc: &scraper::Html,
    curr_link: &str,
    orig_domain: &str,
    sub_site_map: &mut HashSet<String>,
    element: &str,
    attr: &str,
) -> Result<()> {
    info!(
        "[Thread {}] extracting links from <{} {}='...'>",
        tid, element, attr
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
        info!("[Thread {}] resolved_link = {}", tid, resolved_link);

        if link_domain == *orig_domain {
            info!(
                "[Thread {}] adding the resolved_link = {}",
                tid, resolved_link
            );
            sub_site_map.insert(resolved_link.to_string());
        }

        info!(
            "[Thread {}] finished extracting links from <a href='...'>",
            tid
        );
    }

    Ok(())
}

pub fn get_orig_domain(url_str: &str) -> Option<String> {
    Url::parse(url_str).ok()?.host_str().map(String::from)
}

pub fn resolve_link(base_url: &str, href: &str) -> Option<String> {
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
