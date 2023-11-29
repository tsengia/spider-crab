use mockito::Server;

use crate::{SpiderOptions, PageGraph, PageMap, Page};

#[test]
fn test_simple_page() {
    let mut server = Server::new();

    let url = server.url();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body("<!DOCTYPE html><html><body><a href=\"https://example.com\" >Example Link</a></body></html>")
      .create();

    let mut options = SpiderOptions::new(&url);
    
    let mut page_graph = Mutex::<PageGraph>::(PageGraph::new());

}