//! Helper functions called by the page traversal algorithm

use scraper::ElementRef;
use url::{ParseError, Url};

/// Attempt to extract and parse a URL from a `<a>` HTML element
/// Returns `Some(Url)` if extract + parse was successful
/// Returns `None` if extraction or parsing failed
pub fn get_url_from_element(element: ElementRef, current_url: &Url) -> Option<Url> {
    let href_attribute = element.attr("href");

    if href_attribute.is_none() {
        // Element is missing href attribute
        return None;
    }

    let next_url_str = href_attribute.unwrap();

    if next_url_str.len() == 0 {
        // href attribute value is ""
        return None;
    }

    let next_url = parse_relative_or_absolute_url(current_url, next_url_str);
    if next_url.is_none() {
        // Failed to parse URL in the href
        return None;
    }

    next_url
}

/// Attempts to grab the domain name from `url` and compare it against `domain_name`.
/// Returns `true` if domain names match.
/// Returns `false` if domain names are different, or if failed to obtain domain name for `url`
pub fn check_domain(domain_name: &str, url: &Url) -> bool {
    let url_domain = url.domain();
    if url_domain.is_none() {
        // URL doesn't have a domain associated with it
        return false;
    }
    let url_domain = url_domain.unwrap();

    // Return true if the two domains match
    domain_name == url_domain
}

/// Parses a string into a URL. String can be an absolute URL, or a relative URL.
/// If `url_str` is a relative URL, then it will be parsed relative to `current_url`
/// Returns `None` if no valid URL could be parsed
pub fn parse_relative_or_absolute_url(current_url: &Url, url_str: &str) -> Option<Url> {
    // Try to parse an absolute URL from the string
    let mut parsed_url = Url::parse(url_str);

    if parsed_url.is_err() {
        // URL parse failed, is it a relative
        let err = parsed_url.err().unwrap();
        match err {
            ParseError::RelativeUrlWithoutBase => {
                // Error code tells us it is a relative URL,
                //  supply the base URL to parse the relative URL against
                parsed_url = current_url.join(url_str);
                if parsed_url.is_err() {
                    // Relative URL parse failed
                    return None;
                }
            }
            // URL parse failed entirely, not a valid URL
            _ => return None,
        }
    }

    // Remove anything with a # in the parsed URL to deduplicate
    //  URLs pointing to same page but different sections
    let mut parsed_url = parsed_url.unwrap();
    parsed_url.set_fragment(None);

    return Some(parsed_url);
}