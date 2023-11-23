//! Holds the custom SpiderError struct used by spider crab

#[derive(Debug)]
/// Custom error type for Spider Crab
pub struct SpiderError {
    pub message: String,
}

impl std::error::Error for SpiderError {}

impl std::fmt::Display for SpiderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpiderError: {}", self.message)
    }
}
