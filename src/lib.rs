use error::SpiderError;
use petgraph::graph::{DiGraph, NodeIndex};
use reqwest::StatusCode;
use scraper::{selector::CssLocalName, Selector};
use std::collections::HashMap;
use std::sync::Mutex;
use url::{Host, Url};

pub mod algo;
pub mod error;
pub mod url_helpers;

#[cfg(test)]
pub mod tests;

pub struct Link {
    pub html: String,
}

/// Representation of a document/page
pub struct Page {
    /// Title of the page
    pub title: Option<String>,
    /// Content-Type that was given when this page was visited
    pub content_type: Option<String>,
    /// True if the page was visited and a 2XX HTTP status code was returned, false otherwise
    pub good: Option<bool>,
    /// True if this page was visited, false otherwise
    pub checked: bool,
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
            checked: false,
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
    /// Scraper CSS Selector for link elements
    pub link_selector: Box<Selector>,
    /// Scraper CSS Selector for title elements
    pub title_selector: Box<Selector>,
    /// Flag to enable quiet mode. True if quiet mode enabled.
    pub quiet: bool,
    /// Flag to enable verbose mode. True if verbose mode enabled.
    pub verbose: bool,
    /// Name of the CSS class that marks elements to not check URLs for
    pub skip_class: CssLocalName,
    /// Vector of hosts (domain names and IP addresses) that Spider Crab will traverse
    pub hosts: Vec<Host<String>>,
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
}

impl Default for SpiderOptions {
    fn default() -> Self {
        Self {
            max_depth: -1,
            link_selector: Box::new(Selector::parse("a").expect("Invalid title selector!")),
            title_selector: Box::new(Selector::parse("title").expect("Invalid title selector!")),
            quiet: false,
            #[cfg(test)]
            verbose: true,
            #[cfg(not(test))]
            verbose: false,
            skip_class: CssLocalName::from("scrab-skip"),
            hosts: vec![],
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
    pub fn new(domain_names: &[&str]) -> Self {
        Self {
            options: SpiderOptions::new(domain_names),
            ..Default::default()
        }
    }

    pub async fn visit_website(&mut self, url: &str) -> bool {
        let url = Url::parse(url).unwrap();
        let map_mutex = Mutex::<&mut PageMap>::new(&mut self.map);
        let graph_mutex = Mutex::<&mut PageGraph>::new(&mut self.graph);
        algo::visit_root_page(&url, &self.client, &self.options, &graph_mutex, &map_mutex).await
    }
}
