use anyhow::{anyhow, Result};
use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use dashmap::DashSet;
use reqwest;
use scraper;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::log_error;
use crate::utils::*;

pub struct ThreadCrawler {}

impl ThreadCrawler {
    pub fn build_site_map(
        start_from: &String,
        max_links: usize,
        max_wait_time_s: u64,
        max_concurrency: u64,
    ) -> Result<HashSet<String>> {
        build_site_map(start_from, max_links, max_wait_time_s, max_concurrency)
    }
}

fn build_site_map(
    start_from: &String,
    max_links: usize,
    max_wait_time_s: u64,
    max_concurrency: u64,
) -> Result<HashSet<String>> {
    info!("start_from = {}", start_from);

    let orig_domain =
        get_orig_domain(start_from).ok_or_else(|| log_error!("Invalid URL {}", start_from))?;
    info!("orig_domain = {}", &orig_domain);

    let mut thread_handles = Vec::new();

    let rs_site_map: Arc<DashSet<String>> = Arc::new(DashSet::new());
    let visited: Arc<DashSet<String>> = Arc::new(DashSet::new());
    let (sender, receiver) = unbounded();

    // Send initial URL
    sender.send(start_from.clone())?;

    // Keep a sender reference to prevent channel from closing
    let _sender_keeper = sender.clone();

    for tid in 0..max_concurrency {
        let orig_domain = orig_domain.clone();
        let sender = sender.clone();
        let receiver = receiver.clone();
        let rs_site_map = rs_site_map.clone();
        let visited = visited.clone();

        let handle = thread::Builder::new()
            .name(format!("crawler-{}", tid))
            .spawn(move || {
                let _ = extract_links_from(
                    tid,
                    &orig_domain,
                    sender,
                    receiver,
                    rs_site_map,
                    visited,
                    max_links,
                    max_wait_time_s,
                );
            })?;
        thread_handles.push(handle);
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }

    // Drop the keeper after a reasonable time to allow threads to eventually stop
    drop(_sender_keeper);

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
    max_wait_time_s: u64,
) -> Result<()> {
    let short_timeout = Duration::from_millis(500); // Short timeout for checking
    let max_idle_time = Duration::from_secs(max_wait_time_s);
    let mut last_work_time = std::time::Instant::now();

    loop {
        if rs_site_map.len() >= max_links {
            info!("[Thread {}] Reached max_links {}, exiting", tid, max_links);
            return Ok(());
        }

        // The key is to do short timeouts, then keep accumulating the idle time.
        // Reset last_work_time whenever there's a new message.
        // If we use recv_timeout(max_idle_time), likely the thread will exit
        // without doing anything useful.
        match receiver.recv_timeout(short_timeout) {
            Ok(curr_link) => {
                last_work_time = std::time::Instant::now(); // Reset idle timer

                if rs_site_map.len() >= max_links {
                    return Ok(());
                }

                if visited.contains(&curr_link) {
                    info!("[Thread {}] Already visited {}, skipping", tid, curr_link);
                    continue;
                }

                info!("[Thread {}] <<< processing the link {}", tid, curr_link);

                match reqwest::blocking::get(&curr_link) {
                    Ok(response) => {
                        match response.text() {
                            Ok(html_text) => {
                                let html_doc = scraper::Html::parse_document(&html_text);
                                let mut sub_site_map: HashSet<String> = HashSet::new();

                                for (element, attr) in vec![("a", "href"), ("iframe", "src")] {
                                    info!(
                                        "[Thread {}] ^^^ the size of sub_site_map BEFORE extraction from element {}: {}",
                                        tid, element, sub_site_map.len()
                                    );

                                    let _ = extract_links_from_element(
                                        tid,
                                        &html_doc,
                                        &curr_link,
                                        orig_domain,
                                        &mut sub_site_map,
                                        element,
                                        attr,
                                    );

                                    info!(
                                        "[Thread {}] $$$ the size of sub_site_map AFTER extraction from element {}: {}",
                                        tid,
                                        element,
                                        sub_site_map.len(),
                                    );
                                }

                                // Process discovered links
                                for new_link in sub_site_map {
                                    if !visited.contains(&new_link)
                                        && !rs_site_map.contains(&new_link)
                                    {
                                        info!(
                                            "[Thread {}] >>> queuing the link {}",
                                            tid, &new_link
                                        );
                                        let _ = sender.send(new_link.clone());
                                    }
                                    rs_site_map.insert(new_link);
                                }

                                visited.insert(curr_link.to_string());
                                info!(
                                    "[Thread {}] **processed** the link {} (total discovered: {})",
                                    tid,
                                    curr_link,
                                    rs_site_map.len()
                                );
                            }
                            Err(e) => {
                                error!(
                                    "[Thread {}] Failed to read response text from {}: {}",
                                    tid, curr_link, e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!("[Thread {}] Failed to fetch {}: {}", tid, curr_link, e);
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                // Check if we should continue waiting or exit
                if rs_site_map.len() >= max_links {
                    info!("[Thread {}] Reached max_links on timeout, exiting", tid);
                    return Ok(());
                }

                // Check if we've been idle too long
                if last_work_time.elapsed() > max_idle_time {
                    info!("[Thread {}] Idle timeout reached, exiting", tid);
                    return Ok(());
                }

                // Continue waiting for work
                debug!(
                    "[Thread {}] Waiting for work... (idle for {:?})",
                    tid,
                    last_work_time.elapsed()
                );
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => {
                info!("[Thread {}] Channel disconnected, exiting", tid);
                return Ok(());
            }
        }
    }
}
