use mockito::Server;

use crate::SpiderCrab;

#[tokio::test]
async fn test_simple_page() {
    let mut server = Server::new();

    let url = server.url();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"https://example.com\" >Example Link</a></body></html>")
      .create();

    let spider_crab = SpiderCrab::new(url.as_str());

    let success = spider_crab.visit_website(url.as_str()).await;
    assert!(success);

    mock.assert();
}