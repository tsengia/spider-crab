//! Contains structs used for creating test cases with less code.

use derive_builder::Builder;
use mockito::{ServerGuard, Mock};
use url::Url;

use crate::SpiderCrab;
use crate::error::{SpiderErrorType, SpiderError};

#[derive(Default, Builder)]
#[builder(setter(into, strip_option))]
pub struct SpiderTestPage<'a> {
    /// Content returned in the response body of the mock
    pub content: Option<&'a str>,
    /// Content Type this mock will return
    pub content_type: Option<&'a str>,
    /// Title of the page
    pub title: Option<&'a str>,
    /// URL of the page, relative to the server root
    pub url: &'a str,
    /// HTTP Status code that mockito should return
    #[builder(setter(skip))]
    status_code: u16,
    /// Set to true if we expect this mock to be visited, set to false if it should NOT be visited
    #[builder(setter(skip))]
    expect_visited: bool,
    /// URL of the page
    #[builder(setter(skip))]
    absolute_url: Option<Url>,
    /// HTTP Method this mock listens for
    #[builder(setter(skip))]
    method: &'a str,
    /// Mockito Mock that this page creates and checks
    #[builder(setter(skip))]
    mock: Option<Mock>
}

impl SpiderTestPage<'_> {
    pub fn setup_mock(&mut self, server: &mut ServerGuard) {
        self.absolute_url = Some(Url::parse(format!("{}{}",server.url().as_str(), self.url).as_str()).unwrap());

        let mut mock = server.mock(self.method, self.url).with_status(self.status_code.into());
        
        if self.content_type.is_some() {
            mock = mock.with_header("Content-Type", self.content_type.unwrap());
        }
        
        if self.content.is_some() {
            mock = mock.with_body(self.content.unwrap());
        }

        if !self.expect_visited {
            mock = mock.expect(0);
        }
        else {
            // Crawling algo should only ever request a page once
            mock = mock.expect(1);
        }
        self.mock = Some(mock);
    }

    pub fn assert(&self, spider: &SpiderCrab) {
        assert!(self.absolute_url.is_some(), "Make sure the absolute URL of the mock page is recorded. If this fails, it means that you failed to run setup_mock() before this!");

        // Check that mock was visited the expected number of time (0 or 1)
        self.mock.as_ref().unwrap().assert();
        if self.expect_visited {
            
            assert!(spider.contains_page_by_str(self.url), "Page should be in the page graph");
            assert!(spider.map.contains_key(&Url::parse(self.url).unwrap()), "Page should in the page map");

            let page = spider.get_page_by_str(self.url);

            assert!(page.visited, "Test to make sure the page was visited");
            assert!(page.status_code.is_some(), "Test to make sure a status code was recorded");
            assert_eq!(page.status_code.unwrap(), self.status_code, "Test to make sure status code matches expected value");
            
            if self.content_type.is_some() {
                assert!(page.content_type.is_some(), "Test to make sure Content-Type was recorded");
                assert_eq!(page.content_type.as_ref().unwrap().as_str(), self.content_type.unwrap(), "Test to make sure Content-Type matches expected value");
            }

            if self.title.is_some() {
                assert!(page.title.is_some(), "Test to make sure page Title was recorded");
                assert_eq!(page.title.as_ref().unwrap().as_str(), self.title.unwrap(), "Test to make sure page Title matches expected value");
            }
            else {
                if self.content_type.is_some() && self.content_type.unwrap() == "text/html" {
                    assert!(page.errors.iter().any(|e: &SpiderError| e.error_type == SpiderErrorType::MissingTitle), "If the page does not have a title, and it is an HTML document, then there should be an error recorded.");
                }
            }

            if self.status_code > 299 {
                assert!(!page.errors.is_empty(), "Make sure that for non-2XX HTTP status code an error is recorded");
                assert!(page.errors.iter().any(|e: &SpiderError| e.error_type == SpiderErrorType::HTTPError), "Make sure anSpiderErrorType::HTTPError is recorded when a non-2XX HTTP status code is returned");
            }
        }
    }

}

pub struct SpiderTestServer<'a> {
    pages: Vec::<SpiderTestPage<'a>>,
    server: ServerGuard,
    spider_crab: SpiderCrab
}

