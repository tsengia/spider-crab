use clap::{Arg, ArgAction, Command};
use std::sync::Mutex;
use url::Url;

use spider_crab::algo::visit_root_page;
use spider_crab::error::SpiderError;
use spider_crab::{PageGraph, PageMap, SpiderOptions};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Spider Crab")
        .version("0.0.1")
        .about("Checks links and images in a webpage.")
        .author("Tyler Sengia")
        .arg(
            Arg::new("url")
                .action(ArgAction::Set)
                .required(true)
                .help("URL of the webpage to check."),
        )
        .arg(
            Arg::new("depth")
                .short('d')
                .long("depth")
                .action(ArgAction::Set)
                .default_value("-1")
                .value_parser(clap::value_parser!(i32))
                .help("Depth of links to check. Default is -1 which is unlimited."),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Do not print to STDOUT or STDERR."),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Print more log messages."),
        )
        .get_matches();

    let url_str = matches
        .get_one::<String>("url")
        .expect("No URL supplied!")
        .as_str();

    let url = Url::parse(url_str).unwrap();

    let depth: i32 = *matches.get_one::<i32>("depth").expect("Invalid depth!");

    let quiet: bool = matches.get_flag("quiet");
    let verbose: bool = matches.get_flag("verbose");

    if !quiet {
        println!("Spider Crab");
    }

    let mut options = SpiderOptions::new(url.domain().unwrap());
    options.max_depth = depth;
    options.verbose = verbose;
    let client = reqwest::Client::new();

    let mut map = PageMap::new();
    let mut graph = PageGraph::new();

    const EXPECTED_PAGES: usize = 50;
    graph.reserve_edges(200);
    graph.reserve_nodes(EXPECTED_PAGES);
    map.reserve(EXPECTED_PAGES);

    let graph_mutex = Mutex::new(&mut graph);
    let map_mutex = Mutex::new(&mut map);
    let result = visit_root_page(&url, &client, &options, &graph_mutex, &map_mutex).await;

    if !quiet {
        println!("Discovered {} pages", graph.node_count());
        println!("Discovered {} links", graph.edge_count());
    }

    if result {
        if !quiet {
            println!("All links good!");
        }
        return Ok(());
    } else {
        if !quiet {
            println!("Something failed!");
        }
        let e = Box::new(SpiderError {
            message: String::from("Check failed!"),
        }) as Box<dyn std::error::Error>;
        return Err(e);
    }
}
