use clap::{Arg, ArgAction, Command};
use scraper::{Html, Selector};
use reqwest::Client;

async fn get_document(url: &String, client: &Client) 
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

    let contents = get_document(url, &client).await.expect("Failed to retrieve page!");
    let link_selector = Selector::parse("a").expect("Invalid link selector!");
    let links = contents.select(&link_selector);

    println!("Link count: {}", links.count());

    Ok(())
}
