//! Holds the custom SpiderError struct used by spider crab

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

#[derive(Debug, PartialEq)]
pub enum SpiderErrorType {
    InvalidURL,
    HTTPError,
    UnableToRetrieve,
    MissingAttribute,
    EmptyAttribute,
    MissingTitle,
    FailedCrawl,
}

impl std::error::Error for SpiderError {}

impl std::fmt::Display for SpiderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.get_message();
        write!(f, "SpiderError ({:?}): {}", self.error_type, message)
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
        match &self.error_type {
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
            SpiderErrorType::FailedCrawl => {
                String::from("Found a problem while crawling the target webpage!")
            }
        }
    }
}
