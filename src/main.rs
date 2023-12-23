use log::{error, info};
use std::fs::File;
use std::io::Write;

use clap::{Arg, ArgAction, Command};
use spider_crab::error::SpiderError;
use spider_crab::SpiderCrab;

fn save_graph_file(
    spider_crab: &SpiderCrab,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut f = File::create(filename)?;
    f.write_all(spider_crab.get_dot_format().as_bytes())?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Spider Crab")
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
                .action(ArgAction::SetTrue)
                .help("Silence logging output."),
        )
        .arg(
            Arg::new("verbosity")
                .short('v')
                .action(ArgAction::Count)
                .help("Print more log messages."),
        )
        .arg(
            Arg::new("dot")
                .short('o')
                .long("dot")
                .action(ArgAction::Set)
                .help("Save output to file in graphiz Dot format."),
        )
        .get_matches();

    let url_str = matches
        .get_one::<String>("url")
        .expect("No URL supplied!")
        .as_str();

    let depth: i32 = *matches.get_one::<i32>("depth").expect("Invalid depth!");

    let verbose = matches.get_count("verbosity");

    let dot_output_file = matches.get_one::<String>("dot");

    stderrlog::new()
        .module(module_path!())
        .quiet(matches.get_flag("quiet"))
        .verbosity(verbose as usize)
        .init()
        .unwrap();

    let mut spider_crab = SpiderCrab::default();
    spider_crab.options.add_host(url_str);

    spider_crab.options.max_depth = depth;

    let f = File::open(".spidercrab-ignore");
    if f.is_ok() {
        info!("Found .spidercrab-ignore file! Parsing rules.");
        spider_crab.options.read_ignore_list_from_file(".spidercrab-ignore");
    }
    else {
        info!("Did not find .spidercrab-ignore file.")
    }

    const EXPECTED_PAGES: usize = 50;
    spider_crab.graph.reserve_edges(200);
    spider_crab.graph.reserve_nodes(EXPECTED_PAGES);
    spider_crab.map.reserve(EXPECTED_PAGES);

    let result = spider_crab.visit_website(url_str).await;

    info!("Discovered {} pages", spider_crab.page_count());
    info!("Visited {} pages", spider_crab.map.len());
    info!("Discovered {} links", spider_crab.link_count());

    if result {
        info!("All links good!");
        if dot_output_file.is_some() {
            let save_result = save_graph_file(&spider_crab, dot_output_file.unwrap());
            if save_result.is_err() {
                return Err(save_result.err().unwrap());
            }
        }
        return Ok(());
    } else {
        for page in spider_crab.graph.node_weights() {
            for error in &page.errors {
                error!("{}", error);
            }
        }

        let e = Box::new(SpiderError {
            error_type: spider_crab::error::SpiderErrorType::FailedCrawl,
            ..Default::default()
        }) as Box<dyn std::error::Error>;
        if dot_output_file.is_some() {
            let save_result = save_graph_file(&spider_crab, dot_output_file.unwrap());
            if save_result.is_err() {
                error!(
                    "Save to Dot output file {} failed!",
                    dot_output_file.unwrap()
                );
                error!("Error: {:?}", save_result.err().unwrap());
            }
        }
        return Err(e);
    }
}
