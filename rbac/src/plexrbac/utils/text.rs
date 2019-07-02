//#![crate_name = "doc"]

use regex::Regex;

////////////////////////////////////////////////////////////////////////////////
/// Defines helper method to match text using regular expressions
///
pub fn regex_match(rx: &str, s: &str) -> bool {
    //TODO normalize to_lowercase
    if let Ok(re) = Regex::new(rx) {
        re.is_match(s)
    } else {
        false
    }
}

pub fn regex_find(rx: &str, s: &str) -> bool {
    if let Ok(re) = Regex::new(rx) {
        re.find(s) != None
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use plexrbac::utils::text::{regex_find, regex_match};

    #[test]
    fn test_regex_find() {
        assert!(regex_find(r"(?m)^line \d+", "line one\nline 2\n"));
    }

    #[test]
    fn test_regex_match() {
        assert!(regex_match(r"^\d{4}-\d{2}-\d{2}$", "2014-01-01"));
    }
}
