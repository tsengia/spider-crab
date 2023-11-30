use mockito::Server;
use url::Url;

use crate::SpiderCrab;

#[tokio::test]
async fn test_simple_page() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"https://example.com\" >Example Link</a></body></html>")
      .create();

    let mut spider_crab = SpiderCrab::new(url.as_str());
    
    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure that visit _website() returned true
    assert!(success);    

    // Make sure the HTTP request was made
    mock.assert();

    let graph = &spider_crab.graph;
    // Make sure that the page graph contains only one page
    assert_eq!(graph.node_count(), 1);

    let map = &spider_crab.map;

    // Make sure that the page map contains the singular page
    assert!(map.contains_key(&parsed_url));

    // Make sure there is only one page in the page map
    assert_eq!(map.len(), 1);
}
