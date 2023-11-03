use clap::{Arg, ArgAction, Command};
use scraper::{Html, Selector};
use reqwest::Client;

#[derive(Debug)]
struct LinkPair {
    source_page: String,
    target_page: String,
    message: String
}

#[derive(Debug)]
struct SpiderOptions {
    max_depth: i32,
    domain_name: String,
    link_selector: &Selector
}

#[derive(Debug)]
struct SpiderContext {
    current_depth: i32,
    bad_links: Vec<LinkPair>,
    good_links: Vec<LinkPair>
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

async fn visit_page(url: &str, mut context: SpiderContext, options: &SpiderOptions, client: &Client) 
    -> Result<(), bool> {
    let document = get_document(url, client).await;

    if document.is_err() {
        return Err(false)
    }

    let html = document.unwrap();

    let links = html.select(options.link_selector);

    for l in links {
        let href_attribute = l.attr("href");

        if href_attribute.is_none() {
            context.bad_links.append(LinkPair { 
                message: "Empty href attribute!",
                source_page: url,
                target_page: ""
             });
            continue;
        }

        let new_url = href_attribute.unwrap();

        if new_url.len() == 0 {
            context.bad_links.append(LinkPair { 
                message: "Empty href attribute!",
                source_page: url,
                target_page: ""
             });
            continue;
        }
    }


    Ok(())
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
        .arg(Arg::new("crawl")
            .short('c')
            .long("crawl")
            .action(ArgAction::SetTrue)
            .help("Enable checking of webpages under the same domain that are linked to by the target URL"))
        .get_matches();

    println!("Spider Crab");

    let url = matches.get_one::<String>("url").expect("No URL supplied!");
    println!("Target URL: {}", url);

    let client: Client = Client::new();

    link_selector = Selector::parse("a").expect("Invalid link selector!");

    let contents = get_document(url, &client).await.expect("Failed to retrieve page!");

    let links = contents.select(&link_selector);

    println!("Link count: {}", links.count());

    Ok(())
}
