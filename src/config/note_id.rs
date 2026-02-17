//! NoteIdMatcher: regex-based note ID validation and extraction.

use crate::dd::NoteId;
use crate::error::{Error, Result};
use regex::Regex;

/// Validates and extracts note IDs from various sources.
#[derive(Debug)]
pub struct NoteIdMatcher {
    id_re: Regex,
    link_re: Regex,
}

impl NoteIdMatcher {
    pub fn new(id_format: &str, id_link_format: &str) -> Result<Self> {
        let id_re = Regex::new(id_format)
            .map_err(|e| Error::Config(format!("could not compile regular expression '{id_format}': {e}")))?;
        let link_re = Regex::new(id_link_format)
            .map_err(|e| Error::Config(format!("could not compile regular expression '{id_link_format}': {e}")))?;
        Ok(Self { id_re, link_re })
    }

    /// Check if the given string is a valid note ID (full match).
    pub fn is_valid(&self, test: &str) -> bool {
        if let Some(m) = self.id_re.find(test) {
            !m.as_str().is_empty() && m.as_str() == test
        } else {
            false
        }
    }

    /// Extract a note ID from a link (e.g. `/note/some-id`).
    pub fn extract_link(&self, link: &str) -> Option<NoteId> {
        let caps = self.link_re.captures(link)?;
        let id = caps.get(1)?.as_str();
        if self.is_valid(id) {
            Some(id.to_string())
        } else {
            None
        }
    }

    /// Extract a note ID from a filename (e.g. `some-id.md`).
    pub fn extract_file(&self, filename: &str) -> Option<NoteId> {
        let found = self.id_re.find(filename)?.as_str().to_string();
        if found.is_empty() {
            return None;
        }
        if self.is_valid(&found) {
            Some(found)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_matcher() -> NoteIdMatcher {
        NoteIdMatcher::new(r"[a-z0-9-]+", r"/note/([a-z0-9-]+)").unwrap()
    }

    #[test]
    fn test_is_valid() {
        let m = test_matcher();
        assert!(m.is_valid("hello-world"));
        assert!(m.is_valid("abc123"));
        assert!(!m.is_valid("Hello"));
        assert!(!m.is_valid(""));
    }

    #[test]
    fn test_from_link() {
        let m = test_matcher();
        assert_eq!(m.extract_link("/note/hello-world"), Some("hello-world".to_string()));
        assert_eq!(m.extract_link("/note/INVALID"), None);
        assert_eq!(m.extract_link("/other/hello"), None);
    }

    #[test]
    fn test_from_file() {
        let m = test_matcher();
        assert_eq!(m.extract_file("hello-world.md"), Some("hello-world".to_string()));
        assert_eq!(m.extract_file("abc123"), Some("abc123".to_string()));
    }

    #[test]
    fn test_from_file_no_match() {
        let m = NoteIdMatcher::new(r"\d{14}", r"/note/(\d{14})").unwrap();
        assert_eq!(m.extract_file("short.md"), None);
    }

    #[test]
    fn test_invalid_regex() {
        assert!(NoteIdMatcher::new("[invalid", r"ok").is_err());
        assert!(NoteIdMatcher::new(r"ok", "[invalid").is_err());
    }
}
