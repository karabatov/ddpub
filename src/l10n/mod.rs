//! L10n loader, Key enum, Str lookup.

use crate::dd::Language;
use serde::Deserialize;
use std::fmt;

/// L10n keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    DateFormat,
    DatePublished,
    DateUpdatedPublished,
    FooterPoweredBy,
    TagsTitle,
}

#[derive(Debug, Deserialize)]
struct Strings {
    #[serde(rename = "DateFormat")]
    date_format: String,
    #[serde(rename = "DatePublished")]
    date_published: String,
    #[serde(rename = "DateUpdatedPublished")]
    date_updated_published: String,
    #[serde(rename = "FooterPoweredBy")]
    footer_powered_by: String,
    #[serde(rename = "TagsTitle")]
    tags_title: String,
}

pub struct L10n {
    loc: Strings,
}

impl L10n {
    pub fn new(lang: Language) -> Result<Self, L10nError> {
        let content = match lang {
            Language::EnUS => include_str!("../../l10n/strings/strings.en-US.toml"),
            Language::EnUK => include_str!("../../l10n/strings/strings.en-UK.toml"),
            Language::RuRU => include_str!("../../l10n/strings/strings.ru-RU.toml"),
        };
        let loc: Strings = toml::from_str(content)
            .map_err(|e| L10nError(format!("could not load language strings: {e}")))?;
        Ok(L10n { loc })
    }

    pub fn str(&self, key: Key) -> &str {
        match key {
            Key::DateFormat => &self.loc.date_format,
            Key::DatePublished => &self.loc.date_published,
            Key::DateUpdatedPublished => &self.loc.date_updated_published,
            Key::FooterPoweredBy => &self.loc.footer_powered_by,
            Key::TagsTitle => &self.loc.tags_title,
        }
    }
}

#[derive(Debug)]
pub struct L10nError(String);

impl fmt::Display for L10nError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for L10nError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_en_us() {
        let l = L10n::new(Language::EnUS).unwrap();
        assert_eq!(l.str(Key::TagsTitle), "Tags");
        assert_eq!(l.str(Key::DatePublished), "Published %s");
    }

    #[test]
    fn test_load_en_uk() {
        let l = L10n::new(Language::EnUK).unwrap();
        assert_eq!(l.str(Key::TagsTitle), "Tags");
    }

    #[test]
    fn test_load_ru_ru() {
        let l = L10n::new(Language::RuRU).unwrap();
        assert_eq!(l.str(Key::TagsTitle), "Теги");
    }
}
