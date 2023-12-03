use clap::{Arg, ArgAction, Command};
use spider_crab::error::SpiderError;
use spider_crab::SpiderCrab;

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

    let depth: i32 = *matches.get_one::<i32>("depth").expect("Invalid depth!");

    let quiet: bool = matches.get_flag("quiet");
    let verbose: bool = matches.get_flag("verbose");

    let mut spider_crab = SpiderCrab::default();
    spider_crab.options.add_host(url_str);

    spider_crab.options.max_depth = depth;
    spider_crab.options.verbose = verbose;

    const EXPECTED_PAGES: usize = 50;
    spider_crab.graph.reserve_edges(200);
    spider_crab.graph.reserve_nodes(EXPECTED_PAGES);
    spider_crab.map.reserve(EXPECTED_PAGES);

    let result = spider_crab.visit_website(url_str).await;

    if !quiet {
        println!("Discovered {} pages", spider_crab.graph.node_count());
        println!("Visited {} pages", spider_crab.map.len());
        println!("Discovered {} links", spider_crab.graph.edge_count());
    }

    if result {
        if !quiet {
            println!("All links good!");
        }
        return Ok(());
    } else {
        if !quiet {
            for page in spider_crab.graph.node_weights() {
                for error in &page.errors {
                    println!("{}", error);
                }
            }
        }
        let e = Box::new(SpiderError {
            error_type: spider_crab::error::SpiderErrorType::FailedCrawl,
            source_page: None,
            http_error_code: None,
            target_page: None,
            html: None,
        }) as Box<dyn std::error::Error>;
        return Err(e);
    }
}
