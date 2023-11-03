use clap::{Arg, ArgAction, Command};
use reqwest::Client;


#[tokio::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    let matches = Command::new("Webpage Checker")
        .version("0.0.1")
        .about("Checks links and images in a webpage.")
        .author("Tyler Sengia")
        .arg(Arg::new("url")
            .short('u')
            .long("url")
            .action(ArgAction::Set)
            .help("URL of the webpage to check."))
        .arg(Arg::new("crawl")
            .short('c')
            .long("crawl")
            .action(ArgAction::SetTrue)
            .help("Enable checking of webpages under the same domain that are linked to by the target URL"))
        .get_matches();

    println!("Webpage Checker");
    if !matches.contains_id("url") {
        return Err("No Target URL supplied!");
    }

    let url = matches.get_one::<String>("url").unwrap();
    println!("Target URL: {}", url);

    let client: Client = Client::new();

    let response = client.get(url).send().await;


    //let body = reqwest::get(url).await?.text().await?;

    /*if body {
        println!("Content = {}", body.unwrap());
    }
    else {
        println!("No content returned for request!");
    }*/

    Ok()
}
