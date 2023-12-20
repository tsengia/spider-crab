use derive_builder::Builder;
use mockito::{ServerGuard, Mock};
use url::Url;

use crate::{SpiderCrab, Page};
use crate::error::SpiderErrorType;

#[derive(Default, Builder)]
pub struct SpiderTestPage {
    /// HTTP Status code that mockito should return
    status_code: u16,
    /// Set to true if we expect this mock to be visited, set to false if it should NOT be visited
    expect_visited: bool,
    /// Content Type this mock will return
    content_type: Option<String>,
    /// Title of the page
    title: Option<String>,
    /// URL of the page
    url: String,
    /// Content returned in the response body of the mock
    content: Option<String>,
    /// HTTP Method this mock listens for
    method: String
}

impl SpiderTestPage {
    pub fn create_mock(&self, server: &mut ServerGuard) -> Mock {
        let mut mock = server.mock(self.method.as_str(), self.url.as_str()).with_status(self.status_code.into());
        if self.content_type.is_some() {
            mock = mock.with_header("Content-Type", self.content_type.as_ref().unwrap().as_str());
        }
        if self.content.is_some() {
            mock = mock.with_body(self.content.as_ref().unwrap().as_str());
        }

        return mock;
    }

    pub fn assert_mock(&self, mock: &Mock, page: &Page) {
        if self.expect_visited {
            mock.assert();
            assert!(page.checked, "Test to make sure the page was visited");
            assert!(page.status_code.is_some(), "Test to make sure a status code was recorded");
            assert_eq!(page.status_code, self.status_code, "Test to make sure status code received by test is correctly recorded");
        }
        else {
            assert!(!page.checked, "Test to make sure the page was not visited.")
        }
    }

}