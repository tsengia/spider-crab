///! Tests that are for the "positive" case (ie. no errors, normal execution)
use mockito::Server;
use url::Url;

use crate::Page;
use crate::SpiderCrab;

#[tokio::test]
async fn test_simple_page() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
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

    // Make sure that the page graph contains two pages
    assert_eq!(spider_crab.page_count(), 2);

    // Make sure there is only one link in the page graph
    assert_eq!(spider_crab.link_count(), 1);

    // Make sure that the page map contains the mock page
    assert!(spider_crab.contains_page(&parsed_url));

    // Make sure that the title is set
    assert_eq!(
        spider_crab
            .get_page(&parsed_url)
            .title
            .as_ref()
            .unwrap()
            .as_str(),
        "Page A"
    );
}

#[tokio::test]
async fn test_two_pages() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock1 = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body(include_str!("test_assets/page1.html"))
      .create();

    let mock2 = server.mock("GET", "/page2.html")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body(include_str!("test_assets/page2.html"))
      .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure that visit _website() returned true
    assert!(success);

    // Make sure HTTP requests were made to both mocked endpoints
    mock1.assert();
    mock2.assert();

    // Make sure that the page graph contains two pages
    assert_eq!(spider_crab.page_count(), 2);

    // Make sure that the page graph contains four links
    assert_eq!(spider_crab.link_count(), 4);

    // Make sure that the page map contains the home page
    assert!(spider_crab.contains_page(&parsed_url));
}

#[tokio::test]
async fn test_helper_functions() {
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
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
    let mut server = Server::new();

    let url = server.url();
    let parsed_url = Url::parse(url.as_str()).unwrap();

    let mock = server.mock("GET", "/")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body(include_str!("test_assets/pageC.html"))
      .create();

    let mock_page_b = server.mock("GET", "/pageB.html")
      .with_status(201)
      .with_header("content-type", "text/html")
      .with_body(include_str!("test_assets/pageB.html"))
      .expect(0)
      .create();

    let mut spider_crab = SpiderCrab::new(&[url.as_str()]);

    let success = spider_crab.visit_website(url.as_str()).await;

    // Make sure the HTTP request was made to the first page
    mock.assert();
    mock_page_b.assert();

    // Make sure that visit _website() returned true
    assert!(success);

    // Make sure that the page graph contains one page
    assert_eq!(spider_crab.page_count(), 1);

    // Make sure there are no links in the graph
    assert_eq!(spider_crab.link_count(), 0);

    // Make sure that the page map contains the mock page
    assert!(spider_crab.contains_page(&parsed_url));
    assert!(!spider_crab.contains_page(&parsed_url.join("pageB.html").unwrap()));

    // Make sure there are one page in the page map
    assert_eq!(spider_crab.map.len(), 1);

    // Check the root page
    {
        // Make sure that the root page is correct
        let page_a_weight: &crate::Page = spider_crab.get_page(&parsed_url);
        assert_eq!(page_a_weight.content_type.as_ref().unwrap(), "text/html");
        assert!(page_a_weight.checked);
        assert_eq!(page_a_weight.status_code.unwrap(), 201);
        assert_eq!(page_a_weight.errors.len(), 0);
    }
}
