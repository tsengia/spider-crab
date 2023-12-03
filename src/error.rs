//! Holds the custom SpiderError struct used by spider crab

#[derive(Debug)]
/// Custom error type for Spider Crab
pub struct SpiderError {
    pub source_page: Option<String>,
    pub target_page: Option<String>,
    pub http_error_code: Option<u16>,
    pub error_type: SpiderErrorType,
    pub html: Option<String>,
}

#[derive(Debug)]
pub enum SpiderErrorType {
    InvalidURL,
    BrokenLink,
    MissingHref,
    EmptyHref,
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

impl SpiderError {
    fn get_message(&self) -> String {
        match &self.error_type {
            SpiderErrorType::BrokenLink => format!(
                "Page at {:?} contains a link pointing to {:?}, but {:?} is a bad link!",
                self.source_page.as_ref().unwrap(),
                self.target_page.as_ref().unwrap(),
                self.target_page.as_ref().unwrap()
            ),
            SpiderErrorType::InvalidURL => format!(
                "Page at {:?} contains a link with an invalid URL {:?}!",
                self.source_page.as_ref().unwrap(),
                self.target_page.as_ref().unwrap()
            ),
            SpiderErrorType::MissingHref => format!(
                "Page at {:?} contains a link with no href attribute! Element is: {:?}",
                self.source_page.as_ref().unwrap(),
                self.html.as_ref().unwrap()
            ),
            SpiderErrorType::EmptyHref => format!(
                "Page at {:?} contains a link with an empty href attribute! Element is: {:?}",
                self.source_page.as_ref().unwrap(),
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
