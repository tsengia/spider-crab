use clap::{Arg, ArgAction, Command};
use scraper::{Html, Selector, Element, selector::CssLocalName, ElementRef};
use reqwest::{Client, Response};
use url::{Url, ParseError};
use petgraph::graph::{DiGraph, NodeIndex};
use std::{collections::HashMap, sync::Mutex};
use async_recursion::async_recursion;

#[derive(Debug)]
struct SpiderError {
    message: String
}

impl std::error::Error for SpiderError { }

impl std::fmt::Display for SpiderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpiderError: {}", self.message)
    }
}


struct SpiderOptions<'a> {
    max_depth: i32,
    domain_name: &'a str,
    link_selector: &'a Selector,
    title_selector: &'a Selector,
    client: &'a Client,
    quiet: bool,
    verbose: bool,
    skip_class: CssLocalName
}


struct Link {
    html: String
}

struct Page {
    title: Option<String>,
    content_type: Option<String>,
    good: bool,
    checked: bool,
    url: Url
}

type PageMap = HashMap<Url, NodeIndex>;
type PageGraph = DiGraph<Page, Link>;

fn parse_relative_or_absolute_url(current_url: &Url, url_str: &str) -> Option<Url> {
    // Try to parse an absolute URL from the string
    let mut parsed_url = Url::parse(url_str);

    if parsed_url.is_err() {
        // URL parse failed, is it a relative
        let err = parsed_url.err().unwrap();
        match err {
            ParseError::RelativeUrlWithoutBase => {
                // Error code tells us it is a relative URL,
                //  supply the base URL to parse the relative URL against
                parsed_url = current_url.join(url_str);
                if parsed_url.is_err() {
                    // Relative URL parse failed
                    return None
                }
            }
            // URL parse failed entirely, not a valid URL
            _ => return None
        }
    }
    
    // Remove anything with a # in the parsed URL to deduplicate 
    //  URLs pointing to same page but different sections
    let mut parsed_url = parsed_url.unwrap();
    parsed_url.set_fragment(None);

    return Some(parsed_url);
}

fn check_content_type(response: &Response) -> (bool, Option<String>) {
    if response.headers().contains_key("Content-Type") {
        let content_type = response.headers().get("Content-Type").unwrap().to_str();
        if content_type.is_ok() {
            let mut content_type = content_type.unwrap().to_string().to_lowercase();
            let split_index = content_type.find(";");
            
            if split_index.is_some() {
                let (split_content_type, _) = content_type.split_at(split_index.unwrap());
                content_type = split_content_type.to_string();
            }

            match content_type.as_str() {
                "text/html" | "html" => { return (true, Some(content_type)) },
                _ => { return (false, Some(content_type)) }
            }
        }
    }

    // Response does not contain a Content-Type header, or Failed to get the content-type header
    //  do not attempt to check the page
    // TODO: Warn the user about the missing Content-Type header?
    (false, None)
}

fn check_domain(domain_name: &str, url: &Url) -> bool {
    let url_domain = url.domain();
    if url_domain.is_none() {
        // URL doesn't have a domain associated with it
        return false;
    }
    let url_domain = url_domain.unwrap();

    // Return true if the two domains match
    domain_name == url_domain
}

fn get_url_from_element(element: ElementRef, current_url: &Url) -> Option<Url> {
    let href_attribute = element.attr("href");

    if href_attribute.is_none() {
        // Element is missing href attribute
        return None
    }

    let next_url_str = href_attribute.unwrap();

    if next_url_str.len() == 0 {
        // href attribute value is ""
        return None
    }

    let next_url = parse_relative_or_absolute_url(current_url, next_url_str);
    if next_url.is_none() {
        // Failed to parse URL in the href
        return None
    }

    next_url
}

#[async_recursion]
async fn visit_page<'a>(node_index: NodeIndex, options: &SpiderOptions<'a>, graph_mutex: &Mutex<&mut PageGraph>, page_map_mutex: &Mutex<&mut PageMap>, current_depth: i32) -> bool {
    let url: Url;
    let mut new_nodes = Vec::<NodeIndex>::new();
    let mut found_problem: bool = false;
    // Reserve some space for our new node indices. 
    new_nodes.reserve(64);

    {
        // Momentarily acquire the lock so that we can grab the URL of the page
        url = graph_mutex.lock().unwrap().node_weight(node_index).unwrap().url.clone();
    }   // End of scope, releases the lock

    {
        // Start of new scope, this is to get the document, parse links, and update the graph

        // Send an HTTP(S) GET request for the desired URL
        let response_result = options.client.request(reqwest::Method::GET, url.clone()).send().await;
        let response: Response;
        let is_good = !response_result.is_err();

        {
            // Acquire a lock on the graph so that we can update it with our findings for this page
            let mut graph = graph_mutex.lock().unwrap();
            let page = graph.node_weight_mut(node_index).unwrap();

            if !is_good {
                page.good = false;
                if !options.quiet {
                    println!("Found bad link! {}", url);
                }
                return false;
            }

            response = response_result.unwrap();

            // Attempt to get the Content-Type of the page
            let (parse_html, content_type) = check_content_type(&response);
            page.content_type = content_type.clone();

            // If Content-Type is not HTML, then don't try to parse the HTML
            if !parse_html {
                if options.verbose { 
                    println!("Not parsing HTML for: {}, Content-Type is {:?}", url, content_type);
                }
                return true;
            }

            // Check to see if the domain is inside the starting domain.
            let parse_html = check_domain(&options.domain_name, &url);

            if !parse_html {
                if options.verbose { 
                    println!("Not parsing HTML for: {}, outside of domain", url);
                }
                return true;
            }
        }

        // Get the Contents of the page
        let contents = response.text().await;
        
        // Acquire a lock on the graph so that we can update it with our findings for this page
        let mut graph = graph_mutex.lock().unwrap();
        let page = graph.node_weight_mut(node_index).unwrap();
        if contents.is_err() {
            page.good = false;
            if !options.quiet {
                println!("Failed to get contents of page! {}", url);
            }
            return false;
        }
        let contents = contents.unwrap();
        let html = Html::parse_document(contents.as_str());

        page.good = true;

        if options.verbose {
            println!("Visited page {}", url.as_str());
        }
        
        let links = html.select(options.link_selector);

        let mut page_map = page_map_mutex.lock().unwrap();

        for l in links {

            if l.has_class(&options.skip_class, scraper::CaseSensitivity::CaseSensitive) {
                // Link is marked with the spider-crab-skip class, so skip it
                continue
            }

            // Parse out a URL from the link
            let next_url = get_url_from_element(l, &url);
            if next_url.is_none() {
                println!("Failed to get URL from element: {}", l.html());
                found_problem = true;
                continue;
            }
            let next_url = next_url.unwrap();

            // Check to see if the target URL has already been visited
            let existing_page = page_map.get(&next_url);
            if existing_page.is_some() {
                // Target URL has already been visited
                graph.add_edge(node_index, 
                    *existing_page.unwrap(), 
                     Link { html: l.html() }
                    );
                continue;
            }
            
            // Target URL has not been visited yet, add a node to the graph
            let new_node = graph.add_node(
                    Page {
                    url: next_url.clone(),
                    title: None,
                    content_type: None,
                    good: false,
                    checked: false
                }
            );

            // Add an edge to the graph connecting current page to the target page
            graph.add_edge(node_index, new_node, 
                Link { 
                    html: l.html()
                }
            );

            // Add an entry to the page HashMap to mark that we're going to visit the page
            page_map.insert(next_url.clone(), new_node);

            if current_depth == options.max_depth {
                // If we have reached max depth, then do not add the new node to the
                // new_nodes list. This prevents us from visiting those nodes after
                // this loop finishes
                continue;
            }

            new_nodes.push(new_node);
        }
    }
    
    let mut futures_vec = Vec::new();
    futures_vec.reserve_exact(new_nodes.len());

    // Create a future for each node we discovered
    for node in new_nodes {
        futures_vec.push(visit_page(node, options, graph_mutex, page_map_mutex, current_depth + 1));
    }

    // Wait for all the tasks to complete
    let result = futures::future::join_all(futures_vec).await;

    // Return true if page is OK and all referenced pages also return true
    !found_problem && !result.contains(&false)
}

async fn visit_root_page<'a>(url: &Url, options: &SpiderOptions<'a>, graph: &Mutex<&mut PageGraph>, page_map: &Mutex<&mut PageMap>)
    -> bool {

    let root_index: NodeIndex;
    { 
        // Insert the root page as a node into the graph
        root_index = graph.lock().unwrap().add_node(Page {
            title: None,
            content_type: None,
            good: false,
            checked: false,
            url: url.clone()
        });

        // Mark the root node as visited because visit_page assumes 
        //  that the target page is already marked as visited
        page_map.lock().unwrap().insert(url.clone(), root_index);
    }

    return visit_page(root_index, options, graph, page_map, 0).await;
}

#[tokio::main(flavor="current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Spider Crab")
        .version("0.0.1")
        .about("Checks links and images in a webpage.")
        .author("Tyler Sengia")
        .arg(Arg::new("url")
            .action(ArgAction::Set)
            .required(true)
            .help("URL of the webpage to check."))
        .arg(Arg::new("depth")
            .short('d')
            .long("depth")
            .action(ArgAction::Set)
            .default_value("-1")
            .value_parser(clap::value_parser!(i32))
            .help("Depth of links to check. Default is -1 which is unlimited."))
        .arg(Arg::new("quiet")
            .short('q')
            .long("quiet")
            .action(ArgAction::SetTrue)
            .help("Do not print to STDOUT or STDERR."))
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .action(ArgAction::SetTrue)
            .help("Print more log messages."))
        .get_matches();

    let url_str = matches.get_one::<String>("url").expect("No URL supplied!").as_str();

    let url = Url::parse(url_str).unwrap();
    
    let depth: i32 = *matches.get_one::<i32>("depth").expect("Invalid depth!");

    let quiet: bool = matches.get_flag("quiet");
    let verbose: bool = matches.get_flag("verbose");

    if !quiet {
        println!("Spider Crab");
    }

    let client: Client = Client::new();
    let link_selector = Selector::parse("a").expect("Invalid link selector!");
    let title_selector = Selector::parse("title").expect("Invalid title selector!");
    let skip_class = CssLocalName::from("scrab-skip");

    let options = SpiderOptions {
        client: &client,
        max_depth: depth,
        link_selector: &link_selector,
        title_selector: &title_selector,
        domain_name: url.domain().unwrap(),
        quiet,
        verbose,
        skip_class
    };

    let mut map = PageMap::new();
    let mut graph = PageGraph::new();

    const EXPECTED_PAGES: usize = 50;
    graph.reserve_edges(200);
    graph.reserve_nodes(EXPECTED_PAGES);
    map.reserve(EXPECTED_PAGES);

    let graph_mutex = Mutex::new(&mut graph);
    let map_mutex = Mutex::new(&mut map);
    let result = visit_root_page(&url, &options, &graph_mutex, &map_mutex).await;

    if !quiet {
        println!("Discovered {} pages", graph.node_count());
        println!("Discovered {} links", graph.edge_count());
    }

    if result  {
        if !quiet {
            println!("All links good!");
        }
        return Ok(());
    }
    else {
        if !quiet {
            println!("Something failed!");
        }
        let e = Box::new(SpiderError { message: String::from("Check failed!") }) as Box<dyn std::error::Error>;
        return Err(e);
    }
}
