//! Tests that are for the "negative case", errors, missing pages, etc.

use mockito::Server;
use url::Url;

use crate::error::SpiderErrorType;
use crate::test_utils::SpiderTestPageBuilder;
use crate::test_utils::SpiderTestServer;
use crate::SpiderCrab;

#[tokio::test]
async fn test_missing_page() {
  let mut test_server = SpiderTestServer::default();

  let mut test_page = SpiderTestPageBuilder::default()
      .url("/")
      .content(include_str!("test_assets/page1.html"))
      .title("Page 1")
      .build()
      .unwrap();

  test_server.add_page(&mut test_page);
  assert!(!test_server.run_test().await);

  // Make sure that the page graph contains two pages
  test_server.assert_page_count(2);

  // Make sure there is are two links in the page graph
  test_server.assert_link_count(2);

  // Make sure there is an HTTP Error recorded
  test_server.assert_contains_single_error_of_type(SpiderErrorType::HTTPError);
}

#[tokio::test]
async fn test_missing_href() {
  let mut test_server = SpiderTestServer::default();

  let mut test_page = SpiderTestPageBuilder::default()
      .url("/")
      .content("<!DOCTYPE html><html><title>Test Page</title><body><a>This link doesn't have an href attribute!</a></body></html>")
      .title("Test Page")
      .build()
      .unwrap();

  test_server.add_page(&mut test_page);
  assert!(!test_server.run_test().await);

  // Make sure that the page graph contains one page
  test_server.assert_page_count(1);

  // Make sure there is are no links in the page graph
  test_server.assert_link_count(0);

  // Make sure there is an HTTP Error recorded
  test_server.assert_contains_single_error_of_type(SpiderErrorType::MissingAttribute);
}

#[tokio::test]
async fn test_empty_href() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
        .url("/")
        .content("<!DOCTYPE html><html><title>Test Page</title><body><a href=\"\">This link's href attribute is empty!</a></body></html>")
        .title("Test Page")
        .build()
        .unwrap();

    test_server.add_page(&mut test_page);
    assert!(!test_server.run_test().await);
  
    // Make sure that the page graph contains one page
    test_server.assert_page_count(1);
  
    // Make sure there is are no links in the page graph
    test_server.assert_link_count(0);
  
    // Make sure there is an HTTP Error recorded
    test_server.assert_contains_single_error_of_type(SpiderErrorType::EmptyAttribute);
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

    // Make sure that visit_website() returned false
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
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
        .url("/")
        .content("<!DOCTYPE html><html><head><title>Test Page</title></head><body><a href=\"pageB.html\" >This points to a missing page!</a></body></html>")
        .title("Test Page")
        .build()
        .unwrap();

    let mut test_js: crate::test_utils::SpiderTestPage<'_> = SpiderTestPageBuilder::default()
        .url("/pageB.html")
        .status_code(200)
        .content("alert(\"Hello world!\");")
        .content_type(None)
        .build()
        .unwrap();
  
    test_server.add_page(&mut test_page);
    test_server.add_page(&mut test_js);

    // Note that in this case, we expect the traversal to succeed
    assert!(test_server.run_test().await);
  
    // Make sure that the page graph contains two pages
    test_server.assert_page_count(2);
  
    // Make sure there is are is one link in the page graph
    test_server.assert_link_count(1);
}

#[tokio::test]
async fn test_missing_image() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
        .url("/")
        .content("<!DOCTYPE html><html><head><title>Test Page</title></head><body><a href=\"test_image.png\" >This points to a missing page!</a></body></html>")
        .title("Test Page")
        .build()
        .unwrap();

    let mut test_image = SpiderTestPageBuilder::default()
        .url("/test_image.png")
        .status_code(404)
        .content_type(None)
        .build()
        .unwrap();
  
    test_server.add_page(&mut test_page);
    test_server.add_page(&mut test_image);
    assert!(!test_server.run_test().await);
  
    // Make sure that the page graph contains two pages
    test_server.assert_page_count(2);
  
    // Make sure there is are is one link in the page graph
    test_server.assert_link_count(1);
  
    // Make sure there is an HTTP Error recorded
    test_server.assert_contains_single_error_of_type(SpiderErrorType::HTTPError);
}
