//! Holds algorithm(s) used to traverse across a website

use async_recursion::async_recursion;
use log::{error, info, warn};
use petgraph::graph::NodeIndex;
use reqwest::{Client, Response};
use scraper::{Element, Html};
use std::sync::Mutex;
use url::Url;

use crate::error::{SpiderError, SpiderErrorType};
use crate::url_helpers::{check_host, get_url_from_element};
use crate::{Link, Page, PageGraph, PageMap, SpiderOptions};

/// Attempts to retrieve the HTTP ContentType from a Response and check if it is some form of HTML document.
/// Returns `(true, Some(content_type: String))` if the ContentType is some form of HTML document.
/// Returns `(false, Some(content_type: String))` if the ContentType is not HTML.
/// Returns `(false, None)` if failed to get the ContentType
fn check_content_type(response: &Response) -> (bool, Option<String>) {
    if response.headers().contains_key("Content-Type") {
        let content_type = response.headers().get("Content-Type").unwrap().to_str();
        if let Ok(content_type) = content_type {
            let mut content_type = content_type.to_string().to_lowercase();
            let split_index: Option<usize> = content_type.find(';');

            if let Some(split_index) = split_index {
                let (split_content_type, _) = content_type.split_at(split_index);
                content_type = split_content_type.to_string();
            }

            match content_type.as_str() {
                "text/html" | "html" => return (true, Some(content_type)),
                _ => return (false, Some(content_type)),
            }
        }
    }

    // Response does not contain a Content-Type header, or Failed to get the content-type header
    //  do not attempt to check the page
    // TODO: Warn the user about the missing Content-Type header?
    (false, None)
}

/// Recursive function that visits the URL of the node given by `node_index` in the graph locked by the `graph_mutex`.
/// Keeps track of pages that were already visited by inserting URLs into the HashMap locked behind the `page_map_mutex`.
/// Behavior can be controlled via the `options` parameter.
/// Current distance from the root node is given by the `current_depth` parameter.
/// Will recursive call itself until one of the following occurs:
/// * `current_depth` reaches `options.max_depth`
/// * Domain name of the newly discovered URL does not match the `options.domain_name`
/// * ContentType of the visited URL is not `HTML`
/// * Failed to get the ContentType of the visited URL
/// * HTTP GET request to the URL results in a non-2XX HTTP status code
/// * Newly discovered URL has already been visited
#[async_recursion]
pub async fn visit_page(
    node_index: NodeIndex,
    url: Url,
    client: &Client,
    options: &SpiderOptions,
    graph_mutex: &Mutex<&mut PageGraph>,
    page_map_mutex: &Mutex<&mut PageMap>,
    current_depth: i32,
) -> bool {
    let mut new_nodes = Vec::<(NodeIndex, Url)>::new();
    let mut found_problem: bool = false;
    // Reserve some space for our new node indices.
    new_nodes.reserve(64);

    {
        // Send an HTTP(S) GET request for the desired URL
        let response_result = client
            .request(reqwest::Method::GET, url.clone())
            .send()
            .await;
        let response: Response;

        {
            // Acquire a lock on the graph so that we can update it with our findings for this page
            let mut graph = graph_mutex.lock().unwrap();
            let page = graph.node_weight_mut(node_index).unwrap();

            page.visited = true;
            if response_result.is_err() {
                page.status_code = response_result.err().unwrap().status();
                page.good = Some(false);

                if options.is_rule_enabled(SpiderErrorType::UnableToRetrieve, &url) {
                    page.errors.push(SpiderError {
                        target_page: Some(url.to_string()),
                        error_type: SpiderErrorType::UnableToRetrieve,
                        ..Default::default()
                    });
                    return false;
                }
                return true;
            }

            response = response_result.unwrap();

            // Record the HTTP status code
            page.status_code = Some(response.status());
            if !response.status().is_success() {
                page.good = Some(false);
                if options.is_rule_enabled(SpiderErrorType::HTTPError, &url) {
                    page.errors.push(SpiderError {
                        target_page: Some(url.to_string()),
                        http_error_code: Some(response.status().as_u16()),
                        error_type: SpiderErrorType::HTTPError,
                        ..Default::default()
                    });
                    return false;
                }
                return true;
            }

            // Attempt to get the Content-Type of the page
            let (parse_html, content_type) = check_content_type(&response);
            page.content_type = content_type.clone();

            // If Content-Type is not HTML, then don't try to parse the HTML
            if !parse_html {
                warn!(
                    "Not parsing HTML for: {}, Content-Type is {:?}",
                    url, content_type
                );
                return true;
            }

            // Check to see if the domain is inside the starting domain.
            let parse_html = check_host(&options.hosts, &url);

            if !parse_html {
                info!("Not parsing HTML for: {}, outside of domain", url);
                return true;
            }
        }

        // Get the Contents of the page
        let contents = response.text().await;

        // Acquire a lock on the graph so that we can update it with our findings for this page
        let mut graph = graph_mutex.lock().unwrap();
        {
            let page = graph.node_weight_mut(node_index).unwrap();
            if contents.is_err() {
                page.good = Some(false);

                if options.is_rule_enabled(SpiderErrorType::UnableToRetrieve, &url) {
                    error!("Failed to get contents of page! {}", url);
                    page.errors.push(SpiderError {
                        target_page: Some(url.to_string()),
                        error_type: SpiderErrorType::UnableToRetrieve,
                        ..Default::default()
                    });
                    return false;
                }
                return true;
            }
        }
        let contents = contents.unwrap();
        let html = Html::parse_document(contents.as_str());
        {
            let page = graph.node_weight_mut(node_index).unwrap();
            page.good = Some(true);

            let mut title_element = html.select(options.title_selector.as_ref());
            let title_element = title_element.next();
            if title_element.is_some() {
                page.title = Some(title_element.unwrap().inner_html())
            } else if options.is_rule_enabled(SpiderErrorType::MissingTitle, &url) {
                page.errors.push(SpiderError {
                    error_type: SpiderErrorType::MissingTitle,
                    source_page: Some(url.to_string()),
                    ..SpiderError::default()
                });
                warn!("Page at {} does not have a title!", url.as_str());
            }
        }

        info!("Visited page {}", url.as_str());

        let elements = html.select(options.element_selector.as_ref());

        let mut page_map = page_map_mutex.lock().unwrap();

        for l in elements {
            if l.has_class(&options.skip_class, scraper::CaseSensitivity::CaseSensitive) {
                // Link is marked with the spider-crab-skip class, so skip it
                continue;
            }

            // Parse out a URL from the link
            let next_url = get_url_from_element(l, &url);
            if next_url.is_err() {
                let err = next_url.unwrap_err();
                if options.is_rule_enabled(err.error_type.clone(), &url) {
                    error!("Failed to get URL from element: {}", l.html());

                    found_problem = true;

                    let page = graph.node_weight_mut(node_index).unwrap();
                    page.errors.push(err);
                }
                continue;
            }

            let next_url = next_url.unwrap();
            if next_url.is_none() {
                // Element did not contain a URL, but it was not required, so make sure it's innerHTML contains content
                // This case only happens for <script> elements
                if l.inner_html().trim().is_empty()
                    && options.is_rule_enabled(SpiderErrorType::EmptyScript, &url)
                {
                    error!(
                        "Script element at page {} is missing content!",
                        url.as_str()
                    );

                    found_problem = true;

                    let page = graph.node_weight_mut(node_index).unwrap();
                    page.errors.push(SpiderError {
                        error_type: SpiderErrorType::EmptyScript,
                        source_page: Some(url.to_string()),
                        ..SpiderError::default()
                    });
                }
                continue;
            }
            let next_url = next_url.unwrap();

            // Check to see if the target URL has already been visited
            let existing_page = page_map.get(&next_url);
            if existing_page.is_some() {
                // Target URL has already been visited
                graph.add_edge(node_index, *existing_page.unwrap(), Link { html: l.html() });
                continue;
            }

            // Target URL has not been visited yet, add a node to the graph
            let new_node = graph.add_node(Page::new(&next_url));

            // Add an edge to the graph connecting current page to the target page
            graph.add_edge(node_index, new_node, Link { html: l.html() });

            // Add an entry to the page HashMap to mark that we're going to visit the page
            page_map.insert(next_url.clone(), new_node);

            if current_depth == options.max_depth {
                // If we have reached max depth, then do not add the new node to the
                // new_nodes list. This prevents us from visiting those nodes after
                // this loop finishes
                continue;
            }

            new_nodes.push((new_node, next_url));
        }
    }

    let mut futures_vec = Vec::new();
    futures_vec.reserve_exact(new_nodes.len());

    // Create a future for each node we discovered
    for (node, next_url) in new_nodes {
        futures_vec.push(visit_page(
            node,
            next_url,
            client,
            options,
            graph_mutex,
            page_map_mutex,
            current_depth + 1,
        ));
    }

    // Wait for all the tasks to complete
    let result = futures::future::join_all(futures_vec).await;

    // Return true if page is OK and all referenced pages also return true
    !found_problem && !result.contains(&false)
}

/// Visits the page pointed to by the `url` and then recursively calls `visit_page()` on all links contained in that page.
/// Entry point to the page traversal algorithm.
pub async fn visit_root_page(
    url: &Url,
    client: &Client,
    options: &SpiderOptions,
    graph: &Mutex<&mut PageGraph>,
    page_map: &Mutex<&mut PageMap>,
) -> bool {
    let root_index: NodeIndex;
    {
        // Insert the root page as a node into the graph
        root_index = graph.lock().unwrap().add_node(Page::new(url));

        // Mark the root node as visited because visit_page assumes
        //  that the target page is already marked as visited
        page_map.lock().unwrap().insert(url.clone(), root_index);
    }

    visit_page(root_index, url.clone(), client, options, graph, page_map, 0).await
}
