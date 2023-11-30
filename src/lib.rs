use petgraph::graph::{DiGraph, NodeIndex};
use scraper::{selector::CssLocalName, Selector};
use std::collections::HashMap;
use std::sync::Mutex;
use url::Url;

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
    pub good: bool,
    /// True if this page was visited, false otherwise
    pub checked: bool,
    /// URL that this page is represented by. Does not include URL parameters or fragments
    pub url: Url,
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
    /// Domain name of the root node
    pub domain_name: String,
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
}

impl SpiderOptions {
    pub fn new(domain_name: &str) -> Self {
        Self {
            max_depth: -1,
            link_selector: Box::new(Selector::parse("a").expect("Invalid title selector!")),
            title_selector: Box::new(Selector::parse("title").expect("Invalid title selector!")),
            domain_name: domain_name.to_string(),
            quiet: false,
            verbose: false,
            skip_class: CssLocalName::from("scrab-skip"),
        }
    }
}

impl Default for SpiderOptions {
    fn default() -> Self {
        Self {
            max_depth: -1,
            link_selector: Box::new(Selector::parse("a").expect("Invalid title selector!")),
            title_selector: Box::new(Selector::parse("title").expect("Invalid title selector!")),
            domain_name: "localhost".to_string(),
            quiet: false,
            verbose: false,
            skip_class: CssLocalName::from("scrab-skip"),
        }
    }
}

pub struct SpiderCrab {
    pub options: SpiderOptions,
    pub client: reqwest::Client,
    pub graph: PageGraph,
    pub map: PageMap,
}

impl SpiderCrab {
    pub fn new(domain_name: &str) -> Self {
        Self {
            options: SpiderOptions::new(domain_name),
            client: reqwest::Client::new(),
            map: PageMap::new(),
            graph: PageGraph::new(),
        }
    }

    pub async fn visit_website(mut self, url: &str) -> bool {
        let url = Url::parse(url).unwrap();
        let map_mutex = Mutex::<&mut PageMap>::new(&mut self.map);
        let graph_mutex = Mutex::<&mut PageGraph>::new(&mut self.graph);
        algo::visit_root_page(&url, &self.client, &self.options, &graph_mutex, &map_mutex).await
    }
}

impl Default for SpiderCrab {
    fn default() -> Self {
        Self {
            options: SpiderOptions::default(),
            client: reqwest::Client::new(),
            map: PageMap::new(),
            graph: PageGraph::new(),
        }
    }
}
