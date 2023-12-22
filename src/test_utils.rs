//! Contains structs used for creating test cases with less code.

use derive_builder::Builder;
use mockito::{Mock, Server, ServerGuard};
use url::Url;

use crate::error::{SpiderError, SpiderErrorType};
use crate::SpiderCrab;

#[derive(Builder, Debug)]
pub struct SpiderTestPage<'a> {
    /// HTTP Method this mock listens for
    #[builder(default = "\"GET\"")]
    method: &'a str,
    /// URL of the page, relative to the server root
    pub url: &'a str,
    /// HTTP Status code that mockito should return
    #[builder(default = "200")]
    status_code: u16,
    /// Content returned in the response body of the mock
    #[builder(default = "None")]
    #[builder(setter(strip_option))]
    pub content: Option<&'a str>,
    /// Content Type this mock will return
    #[builder(default = "Some(\"text/html\")")]
    pub content_type: Option<&'a str>,
    /// Title of the page
    #[builder(default = "None")]
    #[builder(setter(strip_option))]
    pub title: Option<&'a str>,
    /// Set to true if we expect this mock to be visited, set to false if it should NOT be visited
    #[builder(default = "true")]
    expect_visited: bool,
    /// URL of the page
    #[builder(setter(skip))]
    absolute_url: Option<Url>,
    /// Mockito Mock that this page creates and checks
    #[builder(setter(skip))]
    mock: Option<Mock>,
}

impl SpiderTestPage<'_> {
    pub fn setup_mock(&mut self, server: &mut ServerGuard) {
        self.absolute_url = Some(
            Url::parse(format!("{}{}", server.url().as_str(), self.url).as_str())
                .expect(format!("Invalid URL for test page: {}!", self.url).as_str()),
        );

        let mut mock = server
            .mock(self.method, self.url)
            .with_status(self.status_code.into());

        if self.content_type.is_some() {
            mock = mock.with_header("content-type", self.content_type.unwrap());
        }

        if self.content.is_some() {
            mock = mock.with_body(self.content.unwrap());
        }

        if !self.expect_visited {
            mock = mock.expect(0);
        } else {
            // Crawling algo should only ever request a page once
            mock = mock.expect(1);
        }
        self.mock = Some(mock.create());
    }

    pub fn assert(&self, spider: &SpiderCrab) {
        assert!(self.absolute_url.is_some(), "Make sure the absolute URL of the mock page is recorded. If this fails, it means that you failed to run setup_mock() before this!");

        // Check that mock was visited the expected number of time (0 or 1)
        self.mock.as_ref().expect("Failed to get mock for test page! Did you forget to call setup_mock() before calling assert()?").assert();
        if self.expect_visited {
            assert!(
                spider.contains_page(
                    self.absolute_url
                        .as_ref()
                        .expect("Failed to get absolute URL for test page!")
                ),
                "Page is not in the page graph!\n{:?}",
                self
            );
            assert!(
                spider.map.contains_key(
                    self.absolute_url
                        .as_ref()
                        .expect("Failed to get absolute URL for test page!")
                ),
                "Page is not in the page map!\n{:?}",
                self
            );

            let page = spider.get_page(
                self.absolute_url
                    .as_ref()
                    .expect("Failed to get absolute URL!"),
            );

            assert!(
                page.visited,
                "Page was not marked as visited in the page graph!\n{:?}",
                self
            );
            assert!(
                page.status_code.is_some(),
                "HTTP Status code was not recorded!\n{:?}",
                self
            );
            assert_eq!(
                page.status_code.unwrap(),
                self.status_code,
                "HTTP Status code does not match expected value!\n{:?}",
                self
            );

            if self.content_type.is_some() {
                assert!(
                    page.content_type.is_some(),
                    "Content-Type was not recorded!\n{:?}",
                    self
                );
                assert_eq!(
                    page.content_type.as_ref().unwrap().as_str(),
                    self.content_type.unwrap(),
                    "Content-Type does not match expected value!\n{:?}",
                    self
                );
            }

            if self.title.is_some() {
                assert!(
                    page.title.is_some(),
                    "Expected title to be recorded but it was not! \n{:?}",
                    self
                );
                assert_eq!(
                    page.title.as_ref().unwrap().as_str(),
                    self.title.unwrap(),
                    "Recorded title does not match expected value for page!\n{:?}",
                    self
                );
            } else {
                if self.content_type.is_some() && self.content_type.unwrap() == "text/html" {
                    assert!(
                        page.errors
                            .iter()
                            .any(|e: &SpiderError| e.error_type == SpiderErrorType::MissingTitle),
                        "Page has a title and is HTML, but no title recorded! {:?}",
                        self
                    );
                }
            }

            if self.status_code > 299 {
                assert!(
                    !page.errors.is_empty(),
                    "No error recorded for page with non-2XX HTTP status code!"
                );
                assert!(page.errors.iter().any(|e: &SpiderError| e.error_type == SpiderErrorType::HTTPError), "No SpiderErrorType::HTTPError was recorded when a non-2XX HTTP status code was returned! {:?}",
                self);
            }
        }
    }
}

pub struct SpiderTestServer<'a> {
    pages: Vec<&'a mut SpiderTestPage<'a>>,
    server: ServerGuard,
    pub spider_crab: SpiderCrab,
}

impl Default for SpiderTestServer<'_> {
    fn default() -> Self {
        Self {
            pages: Vec::<&mut SpiderTestPage>::new(),
            server: Server::new(),
            spider_crab: SpiderCrab::default(),
        }
    }
}

impl<'a> SpiderTestServer<'a> {
    pub async fn run_test(&mut self) -> bool {
        // Add the mock server to list of hosts for the traversal options
        self.spider_crab
            .options
            .add_host(self.server.url().as_str());

        for p in self.pages.iter_mut() {
            p.setup_mock(&mut self.server);
        }
        let result = self
            .spider_crab
            .visit_website(self.server.url().as_str())
            .await;
        for p in self.pages.iter_mut() {
            p.assert(&mut self.spider_crab);
        }
        return result;
    }

    pub fn add_page(&mut self, page: &'a mut SpiderTestPage<'a>) {
        self.pages.push(page);
    }

    pub fn assert_page_count(&mut self, expected_pages: usize) {
        assert_eq!(
            self.spider_crab.page_count(),
            expected_pages,
            "Page graph does not contain the expected number of pages!"
        );
    }

    pub fn assert_link_count(&mut self, expected_links: usize) {
        assert_eq!(
            self.spider_crab.link_count(),
            expected_links,
            "Page graph does not contain the expected number of links!"
        );
    }

    pub fn assert_contains_single_error_of_type(&mut self, error_type: SpiderErrorType) {
        if self.spider_crab.errors().count() > 1 {
            panic!(
                "Expected 1 error of type {:?} but found these errors instead:\n{:?}",
                error_type,
                self.spider_crab.errors().collect::<Vec<_>>()
            );
        }
        if self.spider_crab.errors().count() == 0 {
            panic!("Expected 1 error of type {:?} but found none!", error_type);
        }
        assert!(self
            .spider_crab
            .errors()
            .all(|e| e.error_type == error_type));
    }

    pub fn assert_contains_error_of_type(&mut self, error_type: SpiderErrorType) {
        assert!(
            self.spider_crab
                .errors()
                .any(|e| e.error_type == error_type),
            "Failed to find expected error type!"
        );
    }

    pub fn assert_contains_multiple_errors_of_type(
        &mut self,
        number_of_errors: u32,
        error_type: SpiderErrorType,
    ) {
        assert_eq!(
            self.spider_crab
                .errors()
                .filter(|e| e.error_type == error_type)
                .count() as u32,
            number_of_errors,
            "Failed to find expected number of error type!"
        );
    }
}
