use clap::{Arg, ArgAction, Command};
use scraper::{Html, Selector};
use reqwest::Client;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

#[derive(Debug)]
struct SpiderError {
    url: String,
    referenced_by: String,
    error_code: reqwest::Error
}

struct SpiderOptions<'a> {
    max_depth: i32,
    domain_name: &'a str,
    link_selector: &'a Selector,
    title_selector: &'a Selector,
    client: &'a Client
}

struct Link {
    visited: bool
}

struct Page {
    title: String,
    filetype: String,
    good: bool,
    checked: bool,
    url: String
}

struct SpiderContext<'a> {
    page_map: &'a mut HashMap<String, NodeIndex>,
    graph: &'a mut DiGraph<Page, Link>,
    current_depth: i32
}

async fn get_document(url: &str, client: &Client) 
    -> Result<Html, Box<dyn std::error::Error>> {
    // Send an HTTP(S) GET request for the desired URL
    let response = client.get(url).send().await?;
    
    // Get the Contents of the page
    let contents = response.text().await?;

    // TODO: Detect the type of document that we've retrieved by using the Content-Type header

    // Parse the page into an HTML document
    let document = Html::parse_document(contents.as_str());

    Ok(document)
}

async fn visit_page<'a>(node_index: NodeIndex, options: &SpiderOptions<'a>, context: &mut SpiderContext<'a>) 
    -> Result<(),Box<dyn std::error::Error>> {

    let &mut page = context.graph[node_index];

    // let document = get_document(page.url.as_str(), options.client).await;

    // page.checked = true;
    // if document.is_err() {
    //     page.good = false;
    //     //return Err(false)
    // }

    // let html = document.unwrap();

    // let links = html.select(options.link_selector);

    // for l in links {
    //     let href_attribute = l.attr("href");

    //     if href_attribute.is_none() {
    //         /// TODO: Add bad edge to graph for missing href attribute
    //         continue;
    //     }

    //     let next_url = href_attribute.unwrap();

    //     if next_url.len() == 0 {
    //         /// TODO: Add bad edge to graph for empty href attribute
    //         continue;
    //     }

    //     let existing_page = context.page_map.get(next_url);
    //     if existing_page.is_some() {
    //         // Target page has already been visited
    //         context.graph.add_edge(node_index, *existing_page.unwrap(), Link { visited: true });
    //         continue;
    //     }

    //     // Target page has not been visited yet
    //     let new_page = context.graph.add_node(Page {
    //         url: next_url.to_string(),
    //         title: "".to_string(),
    //         filetype: "".to_string(),
    //         good: false,
    //         checked: false
    //     });

    //     // TODO: Spawn up tasks for visiting added pages
    // }

    Ok(())
}

async fn visit_root_page<'a>(url: &str, options: &SpiderOptions<'a>, context: &mut SpiderContext<'a>)
    -> Result<(), Box<dyn std::error::Error>> {

    let root_index = context.graph.add_node(Page {
        title: String::new(),
        filetype: String::new(),
        good: false,
        checked: false,
        url: String::from(url)
    });

    return visit_page(root_index, options, context).await;
}

#[tokio::main]
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
            .help("Depth of links to check. Default is -1 which is unlimited."))
        .get_matches();

    println!("Spider Crab");

    let url = matches.get_one::<String>("url").expect("No URL supplied!").as_str();
    
    let depth: i32 = *matches.get_one::<i32>("depth").expect("Invalid depth!");

    let client: Client = Client::new();
    let link_selector = Selector::parse("a").expect("Invalid link selector!");
    let title_selector = Selector::parse("title").expect("Invalid title selector!");

    let options = SpiderOptions {
        client: &client,
        max_depth: depth,
        link_selector: &link_selector,
        title_selector: &title_selector,
        domain_name: url
    };

    let mut map = HashMap::<String, NodeIndex>::new();
    let mut graph = DiGraph::<Page, Link>::new();

    graph.reserve_edges(200);
    graph.reserve_nodes(50);

    let mut context = SpiderContext {
        page_map: &mut map,
        graph: &mut graph,
        current_depth: 0
    };

    let result = visit_root_page(url, &options, &mut context).await;

    if result.is_ok() {
        println!("All links good!");
    }
    else {
        println!("Something failed!");
    }

    // TODO: Check value of result and report back error code
    Ok(())
}
