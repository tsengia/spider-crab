//! Tests that are for the "negative case", errors, missing pages, etc.

use mockito::Server;
use url::Url;

use crate::SpiderCrab;
use crate::error::SpiderErrorType;

#[tokio::test]
async fn test_missing_page() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body(include_str!("test_assets/page1.html"))
      .create();

    let missing_page_mock = server.mock("GET", "/page2.html").with_status(404).create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure the HTTP request was made to the first page
    mock.assert();

    // Make sure the HTTP request was made to the missing page
    missing_page_mock.assert();

    // Make sure that visit _website() returned false
    assert!(!success);

    // Make sure that the page graph contains two pages
    assert_eq!(spider_crab.page_count(), 2);

    // Make sure there is only two links in the page graph
    assert_eq!(spider_crab.link_count(), 2);

    // Make sure that the page map contains the mock page
    assert!(spider_crab.contains_page(&parsed_url));
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

    // Make sure that the page graph contains one page
    assert_eq!(spider_crab.page_count(), 1);

    // Make sure there are no links in the page graph
    assert_eq!(spider_crab.link_count(), 0);

    // Make sure that the page map contains the mock page
    assert!(spider_crab.contains_page(&parsed_url));
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

    // Make sure that the page graph contains one page
    assert_eq!(spider_crab.page_count(), 1);

    // Make sure there are no links in the page graph
    assert_eq!(spider_crab.link_count(), 0);

    // Make sure that the page map contains the mock page
    assert!(spider_crab.contains_page(&parsed_url));
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

    // Make sure that the page graph contains two pages
    assert_eq!(spider_crab.page_count(), 2);

    // Make sure there are is only one link in the graph
    assert_eq!(spider_crab.link_count(), 1);

    // Make sure that the page map contains the mock page
    assert!(spider_crab.contains_page(&parsed_url));
    assert!(spider_crab.contains_page(&parsed_url.join("pageB.html").unwrap()));
}

#[tokio::test]
async fn test_empty_content_type() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"pageB.html\">This is a link to page B.</a></body></html>")
      .create();

    let mock_page_b = server
        .mock("GET", "/pageB.html")
        .with_status(201)
        .with_body("alert(\"Hello world!\");")
        .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure the HTTP request was made to the first page
    mock.assert();
    mock_page_b.assert();

    // Make sure that visit _website() returned true
    assert!(success);

    // Make sure that the page graph contains two pages
    assert_eq!(spider_crab.page_count(), 2);

    // Make sure there are is only one link in the graph
    assert_eq!(spider_crab.link_count(), 1);

    // Make sure that the page map contains the mock page
    assert!(spider_crab.contains_page(&parsed_url));
    assert!(spider_crab.contains_page(&parsed_url.join("pageB.html").unwrap()));

    // Make sure there are two pages in the page map
    assert_eq!(spider_crab.map.len(), 2);

    // Check the root page
    {
        // Make sure that the root page's content type is correct
        let page_a_weight: &crate::Page = spider_crab.get_page(&parsed_url);
        assert_eq!(page_a_weight.content_type.as_ref().unwrap(), "text/html");
        assert!(page_a_weight.checked);
        assert_eq!(page_a_weight.status_code.unwrap(), 201);
        assert!(page_a_weight.errors.is_empty());
    }

    {
        // Make sure that page B's content type is correct
        let page_b_weight = spider_crab.get_page(&parsed_url.join("/pageB.html").unwrap());
        assert!(page_b_weight.content_type.is_none());
        assert!(page_b_weight.checked);
        assert_eq!(page_b_weight.status_code.unwrap(), 201);
        assert!(page_b_weight.errors.is_empty());
    }
}

#[tokio::test]
async fn test_missing_image() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"test_image.png\" >This points to a missing page!</a></body></html>")
      .create();

    let img_mock = server
        .mock("GET", "/test_image.png")
        .with_status(404)
        .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure the HTTP request was made to the first page
    mock.assert();

    // Make sure the HTTP request was made to the missing page
    img_mock.assert();

    // Make sure that visit _website() returned false
    assert!(!success);

    // Make sure that the page graph contains two pages
    assert_eq!(spider_crab.page_count(), 2);

    // Make sure there is only one link in the page graph
    assert_eq!(spider_crab.link_count(), 1);

    // Make sure that the page map contains the mock page
    assert!(spider_crab.contains_page(&parsed_url));

    // Make sure the we've reported an HTTP error for the missing image
    assert!(spider_crab.errors().all(|e| e.error_type == SpiderErrorType::HTTPError));
    assert_eq!(spider_crab.errors().count(), 1);
}
