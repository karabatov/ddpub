//! Core types and functions that are integral to DDPub as a whole.

use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// A valid note ID.
pub type NoteId = String;

/// Tag represents a tag (no hashtag).
pub type Tag = String;

/// Builtin enumerates built-in DDPub pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Builtin {
    Feed,
    Search,
    Tags,
}

/// Language represents a supported language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    EnUS,
    EnUK,
    RuRU,
}

#[derive(Debug, Clone)]
pub struct LanguageCode {
    pub full: &'static str,
    pub short: &'static str,
}

pub static SUPPORTED_LANGUAGES: LazyLock<HashMap<Language, LanguageCode>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(Language::EnUS, LanguageCode { full: "en-US", short: "en" });
    m.insert(Language::EnUK, LanguageCode { full: "en-UK", short: "en" });
    m.insert(Language::RuRU, LanguageCode { full: "ru-RU", short: "ru" });
    m
});

static REVERSE_LANGUAGES: LazyLock<HashMap<&'static str, Language>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("en-UK", Language::EnUK);
    m.insert("en-US", Language::EnUS);
    m.insert("ru-RU", Language::RuRU);
    m
});

/// ParseLanguage tries to identify the language from a given code.
/// It returns the "default" language (en-US) and false if it cannot find the code.
pub fn parse_language(l: &str) -> (Language, bool) {
    if let Some(&lang) = REVERSE_LANGUAGES.get(l) {
        (lang, true)
    } else {
        (Language::EnUS, false)
    }
}

pub fn first_submatch(re: &Regex, line: &str) -> Option<String> {
    re.captures(line)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Returns all Language variants for iteration.
pub fn all_languages() -> &'static [Language] {
    &[Language::EnUS, Language::EnUK, Language::RuRU]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_language_valid() {
        assert_eq!(parse_language("en-US"), (Language::EnUS, true));
        assert_eq!(parse_language("en-UK"), (Language::EnUK, true));
        assert_eq!(parse_language("ru-RU"), (Language::RuRU, true));
    }

    #[test]
    fn test_parse_language_invalid() {
        let (lang, ok) = parse_language("fr-FR");
        assert_eq!(lang, Language::EnUS);
        assert!(!ok);
    }

    #[test]
    fn test_first_submatch_match() {
        let re = Regex::new(r"^#\s(.*)$").unwrap();
        assert_eq!(first_submatch(&re, "# Hello"), Some("Hello".to_string()));
    }

    #[test]
    fn test_first_submatch_no_match() {
        let re = Regex::new(r"^#\s(.*)$").unwrap();
        assert_eq!(first_submatch(&re, "No match"), None);
    }
}
