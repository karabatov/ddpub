//! L10n loader, Key enum, Str lookup.

use crate::dd::Language;
use crate::error::{Error, Result};
use serde::Deserialize;

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
    #[serde(rename = "errors")]
    errors: ErrorStrings,
}

/// Localized error message templates. Each field is a template string
/// with `{name}` placeholders that get substituted at runtime.
#[derive(Debug, Deserialize)]
pub struct ErrorStrings {
    #[serde(rename = "ConfigFileOpen")]
    pub config_file_open: String,
    #[serde(rename = "ConfigFileParse")]
    pub config_file_parse: String,
    #[serde(rename = "InvalidRegex")]
    pub invalid_regex: String,
    #[serde(rename = "HomepageConflict")]
    pub homepage_conflict: String,
    #[serde(rename = "HomepageInvalidNoteId")]
    pub homepage_invalid_note_id: String,
    #[serde(rename = "FeedConflict")]
    pub feed_conflict: String,
    #[serde(rename = "FeedInvalidNoteId")]
    pub feed_invalid_note_id: String,
    #[serde(rename = "EmptyTag")]
    pub empty_tag: String,
    #[serde(rename = "TagConflict")]
    pub tag_conflict: String,
    #[serde(rename = "TagInvalidNoteId")]
    pub tag_invalid_note_id: String,
    #[serde(rename = "DuplicateTag")]
    pub duplicate_tag: String,
    #[serde(rename = "MenuMultipleTypes")]
    pub menu_multiple_types: String,
    #[serde(rename = "MenuUnknownBuiltin")]
    pub menu_unknown_builtin: String,
    #[serde(rename = "MenuInvalidNoteId")]
    pub menu_invalid_note_id: String,
    #[serde(rename = "MenuTagNotPublished")]
    pub menu_tag_not_published: String,
    #[serde(rename = "UnsupportedLanguage")]
    pub unsupported_language: String,
    #[serde(rename = "DomainNotSet")]
    pub domain_not_set: String,
    #[serde(rename = "LanguageMismatch")]
    pub language_mismatch: String,
    #[serde(rename = "NotesDirectoryUnreadable")]
    pub notes_directory_unreadable: String,
    #[serde(rename = "NoteNotPublished")]
    pub note_not_published: String,
    #[serde(rename = "MetadataNotFound")]
    pub metadata_not_found: String,
    #[serde(rename = "NoteIo")]
    pub note_io: String,
    #[serde(rename = "ExportDirConflict")]
    pub export_dir_conflict: String,
    #[serde(rename = "ExportDirNotEmpty")]
    pub export_dir_not_empty: String,
    #[serde(rename = "ExportIo")]
    pub export_io: String,
    #[serde(rename = "HomepageContentNotFound")]
    pub homepage_content_not_found: String,
    #[serde(rename = "RouteConflict")]
    pub route_conflict: String,
    #[serde(rename = "MenuNoteContentNotFound")]
    pub menu_note_content_not_found: String,
    #[serde(rename = "MenuTagNotFound")]
    pub menu_tag_not_found: String,
    #[serde(rename = "BrokenLinks")]
    pub broken_links: String,
    #[serde(rename = "LanguageStringsLoadFailed")]
    pub language_strings_load_failed: String,
}

fn strings_content(lang: Language) -> &'static str {
    match lang {
        Language::EnUS => include_str!("strings/strings.en-US.toml"),
        Language::EnUK => include_str!("strings/strings.en-UK.toml"),
        Language::RuRU => include_str!("strings/strings.ru-RU.toml"),
    }
}

/// Load error strings for a language. Falls back to en-US on parse failure.
pub fn error_strings(lang: Language) -> ErrorStrings {
    let content = strings_content(lang);
    if let Ok(s) = toml::from_str::<Strings>(content) {
        return s.errors;
    }
    // Fallback: try en-US if the requested language failed.
    if lang != Language::EnUS {
        let fallback = strings_content(Language::EnUS);
        if let Ok(s) = toml::from_str::<Strings>(fallback) {
            return s.errors;
        }
    }
    // Should never happen with bundled strings.
    panic!("could not parse bundled language strings");
}

#[derive(Debug)]
pub struct L10n {
    loc: Strings,
}

impl L10n {
    pub fn new(lang: Language) -> Result<Self> {
        let content = strings_content(lang);
        let loc: Strings = toml::from_str(content)
            .map_err(|e| Error::LanguageStringsLoadFailed {
                cause: e.to_string(),
            })?;
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

    #[test]
    fn test_error_strings_en_us() {
        let es = error_strings(Language::EnUS);
        assert!(es.empty_tag.contains("[[tags]]"));
    }

    #[test]
    fn test_error_strings_ru_ru() {
        let es = error_strings(Language::RuRU);
        assert!(es.empty_tag.contains("[[tags]]"));
    }
}
