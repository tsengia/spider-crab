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

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure that visit _website() returned true
    assert!(success);

    // Make sure the HTTP request was made
    mock.assert();

    let graph = &spider_crab.graph;
    // Make sure that the page graph contains two pages
    assert_eq!(graph.node_count(), 2);

    // Make sure there is only one link in the page graph
    assert_eq!(graph.edge_count(), 1);

    let map = &spider_crab.map;

    // Make sure that the page map contains the mock page
    assert!(map.contains_key(&parsed_url));

    // Make sure there are two pages in the page map
    assert_eq!(map.len(), 2);
}

#[tokio::test]
async fn test_two_pages() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock1 = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"page2.html\" >Example Link2</a><a href=\"/\" >Example Link1</a></body></html>")
      .create();

    let mock2 = server.mock("GET", "/page2.html")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"page2.html\" >Example Link2</a><a href=\"/\" >Example Link1</a></body></html>")
      .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure that visit _website() returned true
    assert!(success);

    // Make sure HTTP requests were made to both mocked endpoints
    mock1.assert();
    mock2.assert();

    let graph = &spider_crab.graph;
    // Make sure that the page graph contains two pages
    assert_eq!(graph.node_count(), 2);

    // Make sure that the page graph contains four links
    assert_eq!(graph.edge_count(), 4);

    let map = &spider_crab.map;

    // Make sure that the page map contains the home page
    assert!(map.contains_key(&parsed_url));

    // Make sure there are two pages in the page map
    assert_eq!(map.len(), 2);
}


#[tokio::test]
async fn test_missing_page() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"page2.html\" >This points to a missing page!</a></body></html>")
      .create();

    let missing_page_mock = server.mock("GET", "/page2.html")
      .with_status(404)
      .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure the HTTP request was made to the first page
    mock.assert();

    // Make sure the HTTP request was made to the missing page
    missing_page_mock.assert();

    // Make sure that visit _website() returned false
    assert!(!success);

    let graph = &spider_crab.graph;
    // Make sure that the page graph contains two pages
    assert_eq!(graph.node_count(), 2);

    // Make sure there is only one link in the page graph
    assert_eq!(graph.edge_count(), 1);

    let map = &spider_crab.map;

    // Make sure that the page map contains the mock page
    assert!(map.contains_key(&parsed_url));

    // Make sure there are two pages in the page map
    assert_eq!(map.len(), 2);
}


#[tokio::test]
async fn test_missing_href() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a>This link doesn't have an href attribute!</a></body></html>")
      .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure the HTTP request was made to the first page
    mock.assert();

    // Make sure that visit _website() returned false
    assert!(!success);

    let graph = &spider_crab.graph;
    // Make sure that the page graph contains one page
    assert_eq!(graph.node_count(), 1);

    // Make sure there are no links in the page graph
    assert_eq!(graph.edge_count(), 0);

    let map = &spider_crab.map;

    // Make sure that the page map contains the mock page
    assert!(map.contains_key(&parsed_url));

    // Make sure there is only one page in the page map
    assert_eq!(map.len(), 1);
}


#[tokio::test]
async fn test_empty_href() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"\">This link's href attribute is empty!</a></body></html>")
      .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure the HTTP request was made to the first page
    mock.assert();

    // Make sure that visit _website() returned false
    assert!(!success);

    let graph = &spider_crab.graph;
    // Make sure that the page graph contains one page
    assert_eq!(graph.node_count(), 1);

    // Make sure there are no links in the page graph
    assert_eq!(graph.edge_count(), 0);

    let map = &spider_crab.map;

    // Make sure that the page map contains the mock page
    assert!(map.contains_key(&parsed_url));

    // Make sure there is only one page in the page map
    assert_eq!(map.len(), 1);
}


#[tokio::test]
async fn test_empty_href_in_second_page() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"pageB.html\">This is a link to page B.</a></body></html>")
      .create();

    let mock_page_b = server.mock("GET", "/pageB.html")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"\">This link has an empty href attribute!</a></body></html>")
      .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure the HTTP request was made to the first page
    mock.assert();
    mock_page_b.assert();

    // Make sure that visit _website() returned false
    assert!(!success);

    let graph = &spider_crab.graph;
    // Make sure that the page graph contains two pages
    assert_eq!(graph.node_count(), 2);

    // Make sure there are is only one link in the graph
    assert_eq!(graph.edge_count(), 1);

    let map = &spider_crab.map;

    // Make sure that the page map contains the mock page
    assert!(map.contains_key(&parsed_url));
    assert!(map.contains_key(&parsed_url.join("pageB.html").unwrap()));

    // Make sure there are two pages in the page map
    assert_eq!(map.len(), 2);
}