use clap::{Arg, ArgAction, Command};
use scraper::{Html, Selector, Element, selector::CssLocalName};
use reqwest::{Client, Response};
use url::{Url, ParseError};
use petgraph::graph::{DiGraph, NodeIndex};
use std::{collections::HashMap, sync::Mutex};
use async_recursion::async_recursion;

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
    title: String,
    content_type: String,
    good: bool,
    checked: bool,
    url: Url
}

type PageMap = HashMap<Url, NodeIndex>;
type PageGraph = DiGraph<Page, Link>;

#[async_recursion]
async fn visit_page<'a>(node_index: NodeIndex, options: &SpiderOptions<'a>, graph_mutex: &Mutex<&mut PageGraph>, page_map_mutex: &Mutex<&mut PageMap>, current_depth: i32) -> bool {
    let url: Url;
    let mut new_nodes = Vec::<NodeIndex>::new();
    let mut found_problem: bool = false;
    // Reserve some space for our new node indices. 
    // Sadly, we can't use links.count() here because it consumes the iterator, which leaves us
    // with no iterator to iterate over the selected elements with
    new_nodes.reserve(32);

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

            if response.headers().contains_key("Content-Type") {
                let content_type = response.headers().get("Content-Type").unwrap().to_str();
                if content_type.is_ok() {
                    page.content_type = content_type.unwrap().to_string();
                    if page.content_type != "text/html" {
                        // Don't attempt to discover more links if it is not an HTML page
                        println!("Page {} is not text/html, skipping link discovery!", url);
                        page.good = true;
                        return true
                    }
                }
            }
        }

        if url.domain().unwrap() != options.domain_name {
            println!("Not parsing HTML due to being outside of domain! {}", url);
            return true;
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
            let href_attribute = l.attr("href");

            if l.has_class(&options.skip_class, scraper::CaseSensitivity::CaseSensitive) {
                // Link is marked with the spider-crab-skip class, so skip it
                continue
            }

            if href_attribute.is_none() {
                // TODO: Add bad edge to graph for missing href attribute?
                println!("Link missing href attribute! {}", l.html());
                found_problem = true;
                continue;
            }

            let next_url_str = href_attribute.unwrap();

            if next_url_str.len() == 0 {
                // TODO: Add bad edge to graph for empty href attribute
                println!("Found empty href attribute! Link: {}", l.html());
                found_problem = true;
                continue;
            }

            let parsed_url = Url::parse(next_url_str);

            let mut next_url: Url;
            if parsed_url.is_err() {
                let err = parsed_url.err().unwrap();
                match err {
                    ParseError::RelativeUrlWithoutBase => {
                        let parsed_url = url.join(next_url_str);
                        if parsed_url.is_err() {
                            // TODO: Add bad edge to graph for failed parse
                            println!("Failed to parse URL! {}", l.html());
                            found_problem = true;
                            continue
                        }
                        next_url = parsed_url.unwrap();
                    }
                    _ => {
                        // TODO: Add bad edge to graph for failed parse
                        println!("Failed to parse URL! {}", l.html());
                        found_problem = true;
                        continue
                    }
                }
            }
            else {
                next_url = parsed_url.unwrap();
            }

            // Remove anything with a # in it to deduplicate URLs pointing to same page but different sections
            next_url.set_fragment(None);

            let existing_page = page_map.get(&next_url);
            if existing_page.is_some() {
                // Target page has already been visited
                graph.add_edge(node_index, 
                    *existing_page.unwrap(), 
                     Link { html: l.html() }
                    );
                continue;
            }
            
            // Target page has not been visited yet, add a node to the graph
            let new_node = graph.add_node(
                    Page {
                    url: next_url.clone(),
                    title: "".to_string(),
                    content_type: "".to_string(),
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
        root_index = graph.lock().unwrap().add_node(Page {
            title: String::new(),
            content_type: String::new(),
            good: false,
            checked: false,
            url: url.clone()
        });

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

    let mut options = SpiderOptions {
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
        // Ok(())
    }
    else {
        if !quiet {
            println!("Something failed!");
        }
        // TODO: Return an error code
    }

    // TODO: Check value of result and report back error code
    Ok(())
}
