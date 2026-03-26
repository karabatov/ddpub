//! NoteIdMatcher: length-based note ID validation and extraction.

use crate::dd::NoteId;

/// Validates and extracts note IDs using a fixed-length prefix scheme.
///
/// When `id_length > 0`, a note ID is exactly `id_length` characters extracted
/// from the start of filenames (before a space). Links use a configurable
/// prefix/suffix around the ID (e.g. `§202603242008`).
///
/// When `id_length == 0`, no structured IDs are used — files are identified
/// by their full filename.
#[derive(Debug)]
pub struct NoteIdMatcher {
    id_length: usize,
    link_prefix: String,
    link_suffix: String,
}

impl NoteIdMatcher {
    pub fn new(id_length: usize, link_prefix: &str, link_suffix: &str) -> Self {
        Self {
            id_length,
            link_prefix: link_prefix.to_string(),
            link_suffix: link_suffix.to_string(),
        }
    }

    /// Check if the given string is a valid note ID.
    pub fn is_valid(&self, test: &str) -> bool {
        if self.id_length == 0 {
            !test.is_empty()
        } else {
            test.chars().count() == self.id_length && !test.contains(char::is_whitespace)
        }
    }

    /// Extract a note ID from a link (e.g. `§202603242008` or `$some-id`).
    pub fn extract_link(&self, link: &str) -> Option<NoteId> {
        if self.id_length == 0 && self.link_prefix.is_empty() && self.link_suffix.is_empty() {
            return None;
        }

        let rest = if !self.link_prefix.is_empty() {
            link.strip_prefix(&self.link_prefix)?
        } else {
            link
        };

        let rest = if !self.link_suffix.is_empty() {
            rest.strip_suffix(&self.link_suffix)?
        } else {
            rest
        };

        if self.is_valid(rest) {
            Some(rest.to_string())
        } else {
            None
        }
    }

    /// Extract a note ID from a filename (e.g. `202603242008 something.md`).
    ///
    /// Returns `None` when `id_length == 0` or the filename doesn't conform
    /// to the `<id> <title>.md` / `<id>.md` pattern.
    pub fn extract_file(&self, filename: &str) -> Option<NoteId> {
        if self.id_length == 0 {
            return None;
        }

        let stem = filename.strip_suffix(".md").unwrap_or(filename);
        let char_count = stem.chars().count();

        if char_count < self.id_length {
            return None;
        }

        let id: String = stem.chars().take(self.id_length).collect();

        if char_count == self.id_length {
            // Exact match: filename is just the ID (+ .md)
            Some(id)
        } else {
            // Must be followed by a space
            let next_char = stem.chars().nth(self.id_length)?;
            if next_char == ' ' {
                Some(id)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_with_length() {
        let m = NoteIdMatcher::new(12, "$", "");
        assert!(m.is_valid("202603242008"));
        assert!(m.is_valid("abcdefghijkl"));
        assert!(!m.is_valid("short"));
        assert!(!m.is_valid(""));
        assert!(!m.is_valid("202603 42008")); // contains space
        assert!(!m.is_valid("2026032420081")); // too long
    }

    #[test]
    fn test_is_valid_zero_length() {
        let m = NoteIdMatcher::new(0, "", "");
        assert!(m.is_valid("anything"));
        assert!(m.is_valid("a"));
        assert!(!m.is_valid(""));
    }

    #[test]
    fn test_extract_link_with_prefix() {
        let m = NoteIdMatcher::new(12, "$", "");
        assert_eq!(m.extract_link("$202603242008"), Some("202603242008".to_string()));
        assert_eq!(m.extract_link("202603242008"), None); // no prefix
        assert_eq!(m.extract_link("$short"), None); // wrong length
    }

    #[test]
    fn test_extract_link_with_prefix_and_suffix() {
        let m = NoteIdMatcher::new(12, "<<", ">>");
        assert_eq!(m.extract_link("<<202603242008>>"), Some("202603242008".to_string()));
        assert_eq!(m.extract_link("<<202603242008"), None); // no suffix
        assert_eq!(m.extract_link("202603242008>>"), None); // no prefix
    }

    #[test]
    fn test_extract_link_no_prefix_no_suffix_zero_length() {
        let m = NoteIdMatcher::new(0, "", "");
        assert_eq!(m.extract_link("anything"), None); // cannot distinguish
    }

    #[test]
    fn test_extract_link_prefix_only_zero_length() {
        let m = NoteIdMatcher::new(0, "$", "");
        assert_eq!(m.extract_link("$hello"), Some("hello".to_string()));
        assert_eq!(m.extract_link("hello"), None);
    }

    #[test]
    fn test_extract_file_with_space() {
        let m = NoteIdMatcher::new(12, "$", "");
        assert_eq!(m.extract_file("202603242008 something.md"), Some("202603242008".to_string()));
    }

    #[test]
    fn test_extract_file_exact_length() {
        let m = NoteIdMatcher::new(12, "$", "");
        assert_eq!(m.extract_file("202603242008.md"), Some("202603242008".to_string()));
    }

    #[test]
    fn test_extract_file_no_space_wrong_length() {
        let m = NoteIdMatcher::new(12, "$", "");
        assert_eq!(m.extract_file("about.md"), None);
    }

    #[test]
    fn test_extract_file_no_space_after_id() {
        let m = NoteIdMatcher::new(12, "$", "");
        assert_eq!(m.extract_file("202603242008extra.md"), None); // no space after ID
    }

    #[test]
    fn test_extract_file_zero_length() {
        let m = NoteIdMatcher::new(0, "", "");
        assert_eq!(m.extract_file("anything.md"), None);
    }

    #[test]
    fn test_unicode_id_length() {
        let m = NoteIdMatcher::new(3, "$", "");
        assert!(m.is_valid("абв")); // 3 Cyrillic chars
        assert!(!m.is_valid("аб")); // 2 chars
        assert_eq!(m.extract_link("$абв"), Some("абв".to_string()));
    }
}
