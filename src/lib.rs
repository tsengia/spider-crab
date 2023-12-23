use error::{SpiderError, SpiderErrorType};

use log::info;
use petgraph::graph::{DiGraph, NodeIndex};
use reqwest::StatusCode;
use scraper::{selector::CssLocalName, Selector};
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::Mutex;
use std::{collections::HashMap, fs::File};
use url::{Host, Url};

pub mod algo;
pub mod dot;
pub mod error;
pub mod url_helpers;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod positive_tests;

#[cfg(test)]
mod negative_tests;

#[derive(Debug)]
pub struct Link {
    pub html: String,
}

/// Representation of a document/page
#[derive(Debug)]
pub struct Page {
    /// Title of the page
    pub title: Option<String>,
    /// Content-Type that was given when this page was visited
    pub content_type: Option<String>,
    /// True if the page was visited and a 2XX HTTP status code was returned, false otherwise
    pub good: Option<bool>,
    /// True if this page was visited, false otherwise
    pub visited: bool,
    /// URL that this page is represented by. Does not include URL parameters or fragments
    pub url: Url,
    /// HTTP status code returned when this page was visited
    pub status_code: Option<StatusCode>,
    /// Vector of errors encountered while scraping this page
    pub errors: Vec<SpiderError>,
}

impl Page {
    pub fn new(url: &Url) -> Self {
        Self {
            title: None,
            content_type: None,
            good: None,
            visited: false,
            url: url.clone(),
            status_code: None,
            errors: Vec::<SpiderError>::new(),
        }
    }
}

/// Helper type for the HashMap that maps Urls to Nodes in the graph
pub type PageMap = HashMap<Url, NodeIndex>;

/// Helper type that tracks all visited pages and the links between them
pub type PageGraph = DiGraph<Page, Link>;

/// Options to pass to the traversal algorithm
pub struct SpiderOptions {
    /// Maximum depth to traverse from root node.
    /// If set to `-1`, traverses infinitely
    /// If set to `0`, then only visits the root node.
    /// Any positive value visits noes that are a distance `max_depth` away from the root node
    pub max_depth: i32,
    /// Scraper CSS Selector used for getting all elements we want to check
    pub element_selector: Box<Selector>,
    /// Scraper CSS Selector used for getting the <title> of a page
    pub title_selector: Box<Selector>,
    /// Name of the CSS class that marks elements to not check URLs for
    pub skip_class: CssLocalName,
    /// Vector of hosts (domain names and IP addresses) that Spider Crab will traverse
    pub hosts: Vec<Host<String>>,
    /// List of patterns that the user has specified to ignore
    pub ignore_patterns: HashMap<SpiderErrorType, Vec<String>>,
}

impl SpiderOptions {
    /// Convenience constructor that allows for setting a list of URLs to traverse across
    pub fn new(target_urls: &[&str]) -> Self {
        Self {
            hosts: target_urls
                .iter()
                .map(|s| Url::parse(s).unwrap().host().unwrap().to_owned())
                .collect(),
            ..Default::default()
        }
    }

    /// Add the host referenced by `url` to the `hosts` vector. This allows the spider crab algorithm to traverse the newly added host
    pub fn add_host(&mut self, url: &str) {
        self.hosts
            .push(Url::parse(url).unwrap().host().unwrap().to_owned())
    }

    pub fn is_rule_enabled(&self, rule: SpiderErrorType, url: &Url) -> bool {
        let patterns = self.ignore_patterns.get(&rule);
        if patterns.is_none() {
            return true;
        }
        let patterns = patterns.unwrap();
        for p in patterns {
            if p == url.as_str() {
                return false;
            }
        }
        true
    }

    pub fn read_ignore_list_from_file(&mut self, filepath: &str) {
        let ignore_file = File::open(filepath).unwrap();
        let reader = BufReader::new(ignore_file);
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            if let Ok(line) = line {
                let line = line.trim();
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.split_whitespace();
                let rule = parts.next().unwrap_or_else(|| {
                    panic!(
                        "Could not read ignore rule from line {} in the ignore file!",
                        line_num
                    )
                });
                let url = parts.next().unwrap_or_else(|| {
                    panic!(
                        "Could not read URL from line {} in the ignore file!",
                        line_num
                    )
                });
                let error_type = SpiderErrorType::from_str(rule).unwrap_or_else(|_| {
                    panic!("Invalid ignore rule on line {} of ignore file!", line_num)
                });

                if !self.ignore_patterns.contains_key(&error_type) {
                    self.ignore_patterns.insert(error_type.clone(), Vec::new());
                }

                // The get_mut().unwrap() _should_ never panic because we check to make sure that the map contains the key right before this
                self.ignore_patterns
                    .get_mut(&error_type)
                    .unwrap()
                    .push(url.to_string());
                count += 1;
            }
        }
        info!("Parsed {} ignore rules from {}", count, filepath);
    }
}

impl Default for SpiderOptions {
    fn default() -> Self {
        Self {
            max_depth: -1,
            element_selector: Box::new(
                Selector::parse("a,link,img,script").expect("Invalid selector!"),
            ),
            title_selector: Box::new(Selector::parse("title").expect("Invalid <title> selector!")),
            skip_class: CssLocalName::from("scrab-skip"),
            hosts: vec![],
            ignore_patterns: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct SpiderCrab {
    /// Options controlling behavior of the traversal algorithm
    pub options: SpiderOptions,

    /// HTTP client that requests will be sent out with
    pub client: reqwest::Client,

    /// Graph of all pages discovered
    /// Not all discovered pages have been visited
    /// Not all discovered pages are valid (ie. if you attempt to visit a page, it may return a 404!)
    pub graph: PageGraph,

    /// HashMap of pages that have already been visited
    /// Includes pages that are visited and return an HTTP error code
    pub map: PageMap,
}

impl SpiderCrab {
    /// Create a new `SpiderCrab` struct with the list of `domain_names` as valid domains to include while traversing links
    pub fn new(domain_names: &[&str]) -> Self {
        Self {
            options: SpiderOptions::new(domain_names),
            ..Default::default()
        }
    }

    /// Begins crawling the website at `url`
    /// Returns `true` if no errors were found.
    /// Returns `false` if errors were found.
    pub async fn visit_website(&mut self, url: &str) -> bool {
        let url = Url::parse(url).unwrap();
        let map_mutex = Mutex::<&mut PageMap>::new(&mut self.map);
        let graph_mutex = Mutex::<&mut PageGraph>::new(&mut self.graph);
        algo::visit_root_page(&url, &self.client, &self.options, &graph_mutex, &map_mutex).await
    }

    /// Returns the `Page` in the page map given by `url`
    pub fn get_page(&self, url: &Url) -> &Page {
        let node_id = *self.map.get(url).unwrap();
        return self.graph.node_weight(node_id).unwrap();
    }

    /// Returns the `Page` in the page map given by `url`
    pub fn get_page_by_str(&self, url: &str) -> &Page {
        let url = Url::parse(url).unwrap();
        let node_id = *self.map.get(&url).unwrap();
        return self.graph.node_weight(node_id).unwrap();
    }

    /// Returns `true` if the page map contains the page given by `url`
    pub fn contains_page(&self, url: &Url) -> bool {
        self.map.contains_key(url)
    }

    /// Returns `true` if the page map contains the page given by `url`
    pub fn contains_page_by_str(&self, url: &str) -> bool {
        self.map.contains_key(&Url::parse(url).unwrap())
    }

    /// Returns `true` if the page given by `url` was marked good and has no errors
    pub fn is_page_good(&self, url: &Url) -> bool {
        self.get_page(url).good.unwrap_or(false) && self.get_page(url).errors.is_empty()
    }

    /// Returns `true`` if the page given by `url` was marked good and has no errors
    pub fn is_page_good_by_str(&self, url: &str) -> bool {
        let url = Url::parse(url).unwrap();
        self.get_page(&url).good.unwrap_or(false) && self.get_page(&url).errors.is_empty()
    }

    /// Returns the number of pages in the page graph
    pub fn page_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns the number of links in the page graph
    pub fn link_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Returns an iterator over all errors found in the page graph.
    pub fn errors(&self) -> impl Iterator<Item = &SpiderError> {
        self.graph
            .node_weights()
            .flat_map(|node: &Page| node.errors.iter())
    }
}
