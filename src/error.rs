//! Holds the custom SpiderError struct used by spider crab

use std::str::FromStr;
use enum_iterator::{all, Sequence};


#[derive(Debug, Eq, PartialEq, Hash, Sequence, Clone)]
pub enum SpiderErrorType {
    InvalidURL,
    HTTPError,
    UnableToRetrieve,
    MissingAttribute,
    EmptyAttribute,
    MissingTitle,
    EmptyScript,
    #[doc(hidden)]
    FailedCrawl,
    #[doc(hidden)]
    ParseError
}

impl SpiderErrorType {
    fn get_rule_name(&self) -> &'static str {
        match self {
            SpiderErrorType::UnableToRetrieve => "unable-to-retrieve",
            SpiderErrorType::HTTPError => "http-error",
            SpiderErrorType::InvalidURL => "invalid-url",
            SpiderErrorType::MissingAttribute => "missing-attribute",
            SpiderErrorType::EmptyAttribute => "empty-attribute",
            SpiderErrorType::MissingTitle => "missing-title",
            SpiderErrorType::EmptyScript => "empty-script",
            SpiderErrorType::FailedCrawl => "failed-crawl",
            SpiderErrorType::ParseError => "parse-error"
        }
    }
}

impl FromStr for SpiderErrorType {
    type Err = SpiderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        for e in all::<SpiderErrorType>().collect::<Vec<_>>() {
            if s == e.get_rule_name() {
                return Ok(e);
            }
        }
        Err(SpiderError { error_type: SpiderErrorType::ParseError, ..SpiderError::default() })
    }
}

#[derive(Debug)]
/// Custom error type for Spider Crab
pub struct SpiderError {
    pub source_page: Option<String>,
    pub target_page: Option<String>,
    pub http_error_code: Option<u16>,
    pub error_type: SpiderErrorType,
    pub html: Option<String>,
    pub attribute: Option<String>,
}

impl std::error::Error for SpiderError {}

impl std::fmt::Display for SpiderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.get_message();
        write!(f, "SpiderError ({}): {}", self.error_type.get_rule_name(), message)
    }
}

impl Default for SpiderError {
    fn default() -> Self {
        Self {
            error_type: SpiderErrorType::FailedCrawl,
            source_page: None,
            target_page: None,
            http_error_code: None,
            html: None,
            attribute: None,
        }
    }
}

impl SpiderError {
    fn get_message(&self) -> String {
        match self.error_type {
            SpiderErrorType::UnableToRetrieve => format!(
                "Failed to retrieve content for page {:?}!",
                self.target_page.as_ref().unwrap()
            ),
            SpiderErrorType::HTTPError => format!(
                "HTTP GET request received status code {:?} for page {:?}!",
                self.http_error_code.as_ref().unwrap(),
                self.target_page.as_ref().unwrap()
            ),
            SpiderErrorType::InvalidURL => format!(
                "Page at {:?} contains a reference to an invalid URL {:?}!",
                self.source_page.as_ref().unwrap(),
                self.target_page.as_ref().unwrap()
            ),
            SpiderErrorType::MissingAttribute => format!(
                "Page at {:?} contains an element with no {:?} attribute! Element is: {:?}",
                self.source_page.as_ref().unwrap(),
                self.attribute.as_ref().unwrap(),
                self.html.as_ref().unwrap()
            ),
            SpiderErrorType::EmptyAttribute => format!(
                "Page at {:?} contains a link with an empty {:?} attribute! Element is: {:?}",
                self.source_page.as_ref().unwrap(),
                self.attribute.as_ref().unwrap(),
                self.html.as_ref().unwrap()
            ),
            SpiderErrorType::MissingTitle => format!(
                "Page at {:?} does not have a title!",
                self.source_page.as_ref().unwrap()
            ),
            SpiderErrorType::EmptyScript => format!(
                "Page at {:?} has a <script> tag with no `src` attribute and no JavaScript code inside!",
                self.source_page.as_ref().unwrap()
            ),
            SpiderErrorType::FailedCrawl => {
                String::from("Found a problem while crawling the target webpage!")
            },
            SpiderErrorType::ParseError => {
                String::from("Could not parse string into error type!")
            }
        }
    }
}
