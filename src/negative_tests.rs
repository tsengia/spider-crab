//! Tests that are for the "negative case", errors, missing pages, etc.
use crate::error::SpiderErrorType;
use crate::test_utils::SpiderTestPageBuilder;
use crate::test_utils::SpiderTestServer;

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
async fn test_missing_href_hyperlink() {
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
async fn test_missing_href_link() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
      .url("/")
      .content("<!DOCTYPE html><html><head><title>Test Page</title><link /></head><body></body></html>")
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
async fn test_missing_src_img() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
      .url("/")
      .content("<!DOCTYPE html><html><title>Test Page</title><body><img height=\"300\" width=\"200\">This img doesn't have a src attribute!</a></body></html>")
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
async fn test_empty_script() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
        .url("/")
        .content(
            "<!DOCTYPE html><html><title>Test Page</title><body><script></script></body></html>",
        )
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
    test_server.assert_contains_single_error_of_type(SpiderErrorType::EmptyScript);
}

#[tokio::test]
async fn test_empty_href_hyperlink() {
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
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
          .url("/")
          .content("<!DOCTYPE html><html><title>Test Page</title><body><a href=\"pageB.html\">This is a link to page B.</a></body></html>")
          .title("Test Page")
          .build()
          .unwrap();

    let mut test_page_b = SpiderTestPageBuilder::default()
        .url("/pageB.html")
        .content("<!DOCTYPE html><html><body><title>Test Page 2</title><a href=\"\">This link has an empty href attribute!</a></body></html>")
        .title("Test Page 2")
        .build()
        .unwrap();

    test_server
        .add_page(&mut test_page)
        .add_page(&mut test_page_b);
    assert!(!test_server.run_test().await);

    // Make sure that the page graph contains two pages
    test_server.assert_page_count(2);

    // Make sure there is one link in the page graph
    test_server.assert_link_count(1);

    // Make sure there is an HTTP Error recorded
    test_server.assert_contains_single_error_of_type(SpiderErrorType::EmptyAttribute);
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

    test_server.add_page(&mut test_page).add_page(&mut test_js);

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

    test_server
        .add_page(&mut test_page)
        .add_page(&mut test_image);
    assert!(!test_server.run_test().await);

    // Make sure that the page graph contains two pages
    test_server.assert_page_count(2);

    // Make sure there is are is one link in the page graph
    test_server.assert_link_count(1);

    // Make sure there is an HTTP Error recorded
    test_server.assert_contains_single_error_of_type(SpiderErrorType::HTTPError);
}

#[tokio::test]
async fn test_missing_script() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
        .url("/")
        .content("<!DOCTYPE html><html><head><title>Test Page</title></head><body><script src=\"test.js\" ></script></body></html>")
        .title("Test Page")
        .build()
        .unwrap();

    let mut test_image = SpiderTestPageBuilder::default()
        .url("/test.js")
        .status_code(404)
        .content_type(None)
        .build()
        .unwrap();

    test_server
        .add_page(&mut test_page)
        .add_page(&mut test_image);
    assert!(!test_server.run_test().await);

    // Make sure that the page graph contains two pages
    test_server.assert_page_count(2);

    // Make sure there is are is one link in the page graph
    test_server.assert_link_count(1);

    // Make sure there is an HTTP Error recorded
    test_server.assert_contains_single_error_of_type(SpiderErrorType::HTTPError);
}

#[tokio::test]
async fn test_missing_script_and_missing_img() {
    let mut test_server = SpiderTestServer::default();

    let mut test_page = SpiderTestPageBuilder::default()
        .url("/")
        .content("<!DOCTYPE html><html><head><title>Test Page</title></head><body><script src=\"test.js\" ></script><img src=\"test.png\" /></body></html>")
        .title("Test Page")
        .build()
        .unwrap();

    let mut test_image = SpiderTestPageBuilder::default()
        .url("/test.png")
        .status_code(404)
        .content_type(None)
        .build()
        .unwrap();

    let mut test_script = SpiderTestPageBuilder::default()
        .url("/test.js")
        .status_code(404)
        .content_type(None)
        .build()
        .unwrap();

    test_server
        .add_page(&mut test_page)
        .add_page(&mut test_script)
        .add_page(&mut test_image);
    assert!(!test_server.run_test().await);

    // Make sure that the page graph contains three pages
    test_server.assert_page_count(3);

    // Make sure there is are is two links in the page graph
    test_server.assert_link_count(2);

    // Make sure there is an HTTP Error recorded
    test_server.assert_contains_multiple_errors_of_type(2, SpiderErrorType::HTTPError);
}
