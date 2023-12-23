//! Contains structs and methods for parsing a rules ignore file.

use std::{collections::HashMap, fs::File, str::FromStr, io::{BufReader, BufRead}};
use url::Url;

use crate::error::SpiderErrorType;

pub struct IgnoreRules {
    /// List of patterns that the user has specified to ignore
    pub ignore_patterns: HashMap<SpiderErrorType, Vec::<String>>
}

impl IgnoreRules {
    pub fn is_rule_enabled(&self, rule: SpiderErrorType, url: &Url) -> bool {
        let patterns = self.ignore_patterns.get(&rule);
        if patterns.is_none() {
            return true;
        }
        let patterns = patterns.unwrap();
        for p in patterns {
            if p == url.path() {
                return false;
            }
        }
        return true;
    }

    pub fn read_from_file(filepath: &str) -> Self {
        let ignore_file = File::open(filepath).unwrap();
        let reader = BufReader::new(ignore_file);
        let mut map: HashMap<SpiderErrorType, Vec<String>> = HashMap::new();
        let mut line_num = 0;
        for line in reader.lines() {
            line_num += 1;
            if let Ok(line) = line {
                let line = line.trim();
                if line.starts_with("#") || line.is_empty() {
                    continue;
                }
                let mut parts = line.split_whitespace();
                let rule = parts.next().expect(format!("Could not read ignore rule from line {} in the ignore file!", line_num).as_str());
                let url = parts.next().expect(format!("Could not read URL from line {} in the ignore file!", line_num).as_str());
                let error_type = SpiderErrorType::from_str(rule).expect(format!("Invalid ignore rule on line {} of ignore file!", line_num).as_str());

                if ! map.contains_key(&error_type) {
                    map.insert(error_type.clone(), Vec::new());
                }

                // The get_mut().unwrap() _should_ never panic because we check to make sure that the map contains the key right before this
                map.get_mut(&error_type).unwrap().push(url.to_string());
            }
        }
        return Self {
            ignore_patterns: HashMap::new()
        }
    }
}