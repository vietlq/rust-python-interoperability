/*
* Usage:
{
    use tokio::runtime::Runtime;

    let rt = Runtime::new()?;

    let _ = rt.block_on(build_site_map_proper_timeout(
        &start_from,
        max_links,
        max_wait_time_s,
        max_concurrency,
    ));
}
*/

mod thread_crawler;
mod utils;

use anyhow::Result;
use pyo3::{prelude::*, types::PySet};
use std::collections::HashSet;
use tracing::{debug, error, info};
use tracing_subscriber;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::utils::get_orig_domain;

async fn build_site_map_proper_timeout(
    start_from: &str,
    max_links: usize,
    max_wait_time_s: u64,
    max_concurrency: u64,
) -> Result<HashSet<String>> {
    use crate::utils::get_orig_domain;
    use dashmap::DashSet;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::{mpsc, Semaphore};
    use tokio::task::JoinSet;

    info!("Inside async function!");

    let (work_tx, mut work_rx) = mpsc::unbounded_channel::<String>();
    let discovered = Arc::new(DashSet::new());
    let active_tasks = Arc::new(AtomicUsize::new(0));
    let semaphore = Arc::new(Semaphore::new(max_concurrency as usize));

    work_tx.send(start_from.to_string())?;

    let idle_timeout = tokio::time::Duration::from_secs(max_wait_time_s);
    let mut last_activity = tokio::time::Instant::now();

    loop {
        // Check if we've reached our target
        if discovered.len() >= max_links {
            tracing::info!("Reached max_links: {}", max_links);
            break;
        }

        // Try to receive work with a short timeout for checking conditions
        match tokio::time::timeout(tokio::time::Duration::from_millis(500), work_rx.recv()).await {
            Ok(Some(url)) => {
                if discovered.insert(url.clone()) {
                    last_activity = tokio::time::Instant::now(); // Reset idle timer

                    let permit = semaphore.clone().acquire_owned().await?;
                    let work_tx = work_tx.clone();
                    let discovered = discovered.clone();
                    let active_tasks = active_tasks.clone();
                    let orig_domain = get_orig_domain(start_from)
                        .ok_or_else(|| anyhow::anyhow!("Invalid domain"))?;

                    active_tasks.fetch_add(1, Ordering::Relaxed);

                    tokio::spawn(async move {
                        let _permit = permit;

                        match process_url_async(&reqwest::Client::new(), &url, &orig_domain).await {
                            Ok(new_urls) => {
                                for new_url in new_urls {
                                    let _ = work_tx.send(new_url);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to process {}: {}", url, e);
                            }
                        }

                        active_tasks.fetch_sub(1, Ordering::Relaxed);
                    });
                }
            }
            Ok(None) => {
                // Channel closed, but wait for active tasks
                break;
            }
            Err(_) => {
                // Timeout on recv - check if we should stop due to inactivity
                let current_active = active_tasks.load(Ordering::Relaxed);

                if current_active == 0 {
                    // No active tasks, check idle time
                    if last_activity.elapsed() > idle_timeout {
                        tracing::info!(
                            "No activity for {} seconds, stopping (found {} URLs)",
                            max_wait_time_s,
                            discovered.len()
                        );
                        break;
                    }

                    // Also check if work queue is truly empty
                    if work_rx.is_empty() {
                        tracing::info!("No active tasks and work queue empty, stopping");
                        break;
                    }
                }

                // Continue - there are still active tasks or recent activity
                tracing::debug!(
                    "Waiting... Active tasks: {}, URLs found: {}, Idle time: {:?}",
                    current_active,
                    discovered.len(),
                    last_activity.elapsed()
                );
            }
        }
    }

    // Wait for any remaining tasks to complete
    let final_wait_start = tokio::time::Instant::now();
    while active_tasks.load(Ordering::Relaxed) > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Safety timeout for final cleanup - don't wait forever for tasks
        if final_wait_start.elapsed() > tokio::time::Duration::from_secs(30) {
            tracing::warn!("Timeout waiting for tasks to complete, stopping anyway");
            break;
        }
    }

    let mut result = HashSet::new();
    discovered.iter().for_each(|url| {
        result.insert(url.clone());
    });

    tracing::info!("Crawling completed: {} URLs discovered", result.len());
    Ok(result)
}

async fn process_url_async(
    client: &reqwest::Client,
    url: &str,
    orig_domain: &str,
) -> Result<Vec<String>> {
    use crate::utils::{get_orig_domain, resolve_link};

    // Fetch the URL
    let response = client.get(url).send().await?;

    // Check if response is successful
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP error: {} for URL: {}",
            response.status(),
            url
        ));
    }

    // Get content type to ensure we're processing HTML
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("text/html") {
        tracing::debug!("Skipping non-HTML content: {} ({})", url, content_type);
        return Ok(Vec::new());
    }

    // Get the HTML content
    let html_text = response.text().await?;

    // Parse HTML
    let document = scraper::Html::parse_document(&html_text);

    let mut discovered_urls = Vec::new();

    // Extract links from different elements
    for (element, attribute) in [("a", "href"), ("iframe", "src")] {
        if let Ok(selector) = scraper::Selector::parse(element) {
            for element_node in document.select(&selector) {
                if let Some(link_value) = element_node.value().attr(attribute) {
                    // Skip empty, javascript, mailto, tel links
                    if link_value.is_empty()
                        || link_value.starts_with("javascript:")
                        || link_value.starts_with("mailto:")
                        || link_value.starts_with("tel:")
                        || link_value.starts_with("#")
                    {
                        continue;
                    }

                    // Resolve relative URLs to absolute URLs
                    if let Some(resolved_url) = resolve_link(url, link_value) {
                        // Check if the URL is on the same domain
                        if let Some(link_domain) = get_orig_domain(&resolved_url) {
                            if link_domain == orig_domain {
                                // Normalize URL (remove fragments and query params)
                                if let Some(normalized_url) = normalize_url(&resolved_url) {
                                    discovered_urls.push(normalized_url);
                                }
                            } else {
                                tracing::debug!(
                                    "Skipping external domain: {} ({})",
                                    resolved_url,
                                    link_domain
                                );
                            }
                        } else {
                            tracing::debug!("Could not extract domain from: {}", resolved_url);
                        }
                    } else {
                        tracing::debug!(
                            "Could not resolve link: {} from base: {}",
                            link_value,
                            url
                        );
                    }
                }
            }
        } else {
            tracing::warn!("Invalid CSS selector: {}", element);
        }
    }

    tracing::debug!("Extracted {} URLs from {}", discovered_urls.len(), url);
    Ok(discovered_urls)
}

// Helper function to normalize URLs (remove fragments and query params as specified)
fn normalize_url(url: &str) -> Option<String> {
    if let Ok(mut parsed) = url::Url::parse(url) {
        // Remove fragment (anchor)
        parsed.set_fragment(None);

        // Remove query parameters
        parsed.set_query(None);

        Some(parsed.to_string())
    } else {
        None
    }
}
