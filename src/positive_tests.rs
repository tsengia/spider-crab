///! Tests that are for the "positive" case (ie. no errors, normal execution)
use mockito::Server;
use url::Url;

use crate::test_utils::SpiderTestPageBuilder;
use crate::test_utils::SpiderTestServer;
use crate::Page;
use crate::SpiderCrab;

/// Single page, reference to example.com
#[tokio::test]
async fn test_simple_page() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
        .url("/")
        .content(include_str!("test_assets/pageA.html"))
        .title("Page A")
        .build()
        .unwrap();

    test_server.add_page(&mut test_page);
    assert!(test_server.run_test().await);

    // Make sure that the page graph contains two pages
    test_server.assert_page_count(2);

    // Make sure there is only one link in the page graph
    test_server.assert_link_count(1);
}

/// Two pages, each with a reference to itself and the other page, total of 4 links.
#[tokio::test]
async fn test_two_pages() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page1 = SpiderTestPageBuilder::default()
        .url("/")
        .content(include_str!("test_assets/page1.html"))
        .title("Page 1")
        .build()
        .unwrap();

    let mut test_page2 = SpiderTestPageBuilder::default()
        .url("/page2.html")
        .content(include_str!("test_assets/page2.html"))
        .title("Page 2")
        .build()
        .unwrap();

    test_server.add_page(&mut test_page1);
    test_server.add_page(&mut test_page2);
    assert!(test_server.run_test().await);

    // Make sure that the page graph contains two pages
    test_server.assert_page_count(2);

    // Make sure there is only one link in the page graph
    test_server.assert_link_count(4);
}

#[tokio::test]
async fn test_helper_functions() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server
        .mock("GET", "/")
        .with_status(201)
        .with_header("content-type", "text/html")
        .with_body(include_str!("test_assets/pageA.html"))
        .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure that visit _website() returned true
    assert!(success);

    // Make sure the HTTP request was made
    mock.assert();

    // Make sure that these two functions are equivalent
    assert_eq!(
        spider_crab.get_page(&parsed_url) as *const Page,
        spider_crab.get_page_by_str(url.as_str()) as *const Page
    );

    // Make sure that these two functions are equivalent
    assert_eq!(
        spider_crab.contains_page(&parsed_url),
        spider_crab.contains_page_by_str(url.as_str())
    );

    // Make sure that these two functions are equivalent
    assert_eq!(
        spider_crab.is_page_good(&parsed_url),
        spider_crab.is_page_good_by_str(url.as_str())
    );
}

#[tokio::test]
async fn test_skip_link_class() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page_c = SpiderTestPageBuilder::default()
        .url("/")
        .content(include_str!("test_assets/pageC.html"))
        .build()
        .unwrap();

    let mut test_page_d = SpiderTestPageBuilder::default()
        .url("/page2.html")
        .content("alert(\"Hello world!\");")
        .expect_visited(false)
        .build()
        .unwrap();

    test_server.add_page(&mut test_page_c);
    test_server.add_page(&mut test_page_d);
    assert!(test_server.run_test().await);

    // Make sure that the page graph contains one page
    // Links with the skip class will be excluded from the page graph
    test_server.assert_page_count(1);

    // Make sure there are no links in the page graph
    // Links with the skip class will be excluded from the page graph
    test_server.assert_link_count(0);
    
}
