//! Helper functions called by the page traversal algorithm

use crate::error::{SpiderError, SpiderErrorType};
use scraper::ElementRef;
use url::{Host, ParseError, Url};

/// Attempt to extract and parse a URL from an HTML element depending on the element tag.
/// `img``, and `script` elements will extract the URL from the `src` attribute
/// `a`, and `link` elements will extract the URL from the `href` attribute
/// Returns `Ok(Some(Url))` if extract + parse was successful
/// Returns `Ok(None)` if element did not have a URL, but it is not required to have one (such as the `script` elemnt)
/// Returns `Err(SpiderError)` if element did not have a URL, and is required to have one
pub fn get_url_from_element(
    element: ElementRef,
    current_url: &Url,
) -> Result<Option<Url>, SpiderError> {
    let (attribute_name, required) = match element.value().name() {
        "a" | "link" => ("href", true),
        "script" => ("src", false),
        "img" => ("src", true),
        &_ => panic!("Unsupported element type passed to get_url_from_element!"),
    };

    let attribute = element.attr(attribute_name);

    if attribute.is_none() {
        if required {
            // Element does not have the needed attribute to find the source
            return Err(SpiderError {
                error_type: SpiderErrorType::MissingAttribute,
                attribute: Some(attribute_name.to_string()),
                source_page: Some(current_url.to_string()),
                html: Some(element.html()),
                ..Default::default()
            });
        }
        return Ok(None);
    }

    let next_url_str = attribute.unwrap();

    if next_url_str.is_empty() {
        // Element's href attribute value is ""
        return Err(SpiderError {
            error_type: SpiderErrorType::EmptyAttribute,
            attribute: Some(attribute_name.to_string()),
            source_page: Some(current_url.to_string()),
            html: Some(element.html()),
            ..Default::default()
        });
    }

    let next_url = parse_relative_or_absolute_url(current_url, next_url_str);

    if next_url.is_none() {
        // Failed to parse the URL, report it as an error
        return Err(SpiderError {
            error_type: SpiderErrorType::InvalidURL,
            source_page: Some(current_url.to_string()),
            target_page: Some(next_url_str.to_string()),
            html: Some(element.html()),
            ..Default::default()
        });
    }

    Ok(Some(next_url.unwrap()))
}

/// Attempts to grab the host from `url` and see if it matches any element listed in `hosts`
/// Returns `true` if `url` matches any entry of `hosts`
/// Returns `false` if `url` fails to match any entry in `hosts`, or if failed to obtain a host for `url`
pub fn check_host(hosts: &[Host<String>], url: &Url) -> bool {
    let url_host = url.host();
    if url_host.is_none() {
        // URL doesn't have a host associated with it
        return false;
    }
    let url_host = url_host.unwrap().to_owned();

    // Return true if the domain/IP + port matches any entry in domain_names
    hosts.iter().any(|h| *h == url_host)
}

#[test]
fn test_check_host_match() {
    let url = Url::parse("https://example.net").unwrap();
    let host_name = "example.net";
    assert!(check_host(
        &[Host::parse(host_name).unwrap().to_owned()],
        &url
    ));
}

#[test]
fn test_check_host_match_ipv4() {
    let url = Url::parse("https://172.0.0.1").unwrap();
    let host_name = "172.0.0.1";
    assert!(check_host(
        &[Host::parse(host_name).unwrap().to_owned()],
        &url
    ));
}

#[test]
fn test_check_host_match_ipv6() {
    let url = Url::parse("https://[::1]").unwrap();
    let host_name = "[::1]";
    assert!(check_host(
        &[Host::parse(host_name).unwrap().to_owned()],
        &url
    ));
}

#[test]
fn test_check_domain_match_with_params() {
    let url = Url::parse("https://abcd123.com/another/file?q=3&c=234234").unwrap();
    let host_name = "abcd123.com";
    assert!(check_host(
        &[Host::parse(host_name).unwrap().to_owned()],
        &url
    ));
}

#[test]
fn test_check_domain_match_with_params_and_fragment() {
    let url = Url::parse("http://example.com/another/file?param=2#fragment3").unwrap();
    let host_name = "example.com";
    assert!(check_host(
        &[Host::parse(host_name).unwrap().to_owned()],
        &url
    ));
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

    Some(parsed_url)
}

#[test]
fn test_parse_relative_url() {
    let base = Url::parse("https://example.com/").unwrap();
    let expected = Url::parse("https://example.com/relative/path").unwrap();

    let result = parse_relative_or_absolute_url(&base, "relative/path").unwrap();

    assert_eq!(expected, result);
}

#[test]
fn test_parse_relative_url2() {
    let base = Url::parse("https://example.com/").unwrap();
    let expected = Url::parse("https://example.com/another_relative_path.html").unwrap();

    let result = parse_relative_or_absolute_url(&base, "another_relative_path.html").unwrap();

    assert_eq!(expected, result);
}

#[test]
fn test_parse_absolute_url() {
    let base = Url::parse("https://example.com/").unwrap();
    let expected = Url::parse("https://this-is-another-website.org/").unwrap();

    let result =
        parse_relative_or_absolute_url(&base, "https://this-is-another-website.org").unwrap();

    assert_eq!(expected, result);
}
