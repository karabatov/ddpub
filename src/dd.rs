//! Core types and functions that are integral to DDPub as a whole.

use std::path::Path;

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

impl Language {
    pub const ALL: &[Language] = &[Language::EnUS, Language::EnUK, Language::RuRU];

    pub fn full_code(self) -> &'static str {
        match self {
            Language::EnUS => "en-US",
            Language::EnUK => "en-UK",
            Language::RuRU => "ru-RU",
        }
    }

    pub fn short_code(self) -> &'static str {
        match self {
            Language::EnUS | Language::EnUK => "en",
            Language::RuRU => "ru",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en-US" => Some(Language::EnUS),
            "en-UK" => Some(Language::EnUK),
            "ru-RU" => Some(Language::RuRU),
            _ => None,
        }
    }
}

/// ParseLanguage tries to identify the language from a given code.
/// It returns the "default" language (en-US) and false if it cannot find the code.
pub fn parse_language(l: &str) -> (Language, bool) {
    match Language::from_code(l) {
        Some(lang) => (lang, true),
        None => (Language::EnUS, false),
    }
}

/// Returns all Language variants for iteration.
pub fn all_languages() -> &'static [Language] {
    Language::ALL
}

/// Guess a MIME content type from a file path's extension.
pub fn guess_content_type(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "txt" => "text/plain; charset=utf-8",
        "wasm" => "application/wasm",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
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
    fn test_guess_content_type_common() {
        assert_eq!(guess_content_type(Path::new("style.css")), "text/css");
        assert_eq!(guess_content_type(Path::new("image.png")), "image/png");
        assert_eq!(guess_content_type(Path::new("photo.jpg")), "image/jpeg");
        assert_eq!(guess_content_type(Path::new("icon.svg")), "image/svg+xml");
        assert_eq!(guess_content_type(Path::new("page.html")), "text/html; charset=utf-8");
    }

    #[test]
    fn test_guess_content_type_fonts() {
        assert_eq!(guess_content_type(Path::new("font.woff")), "font/woff");
        assert_eq!(guess_content_type(Path::new("font.woff2")), "font/woff2");
        assert_eq!(guess_content_type(Path::new("font.ttf")), "font/ttf");
        assert_eq!(guess_content_type(Path::new("font.otf")), "font/otf");
    }

    #[test]
    fn test_guess_content_type_unknown() {
        assert_eq!(guess_content_type(Path::new("file.xyz")), "application/octet-stream");
        assert_eq!(guess_content_type(Path::new("noext")), "application/octet-stream");
    }

    #[test]
    fn test_language_full_code() {
        assert_eq!(Language::EnUS.full_code(), "en-US");
        assert_eq!(Language::EnUK.full_code(), "en-UK");
        assert_eq!(Language::RuRU.full_code(), "ru-RU");
    }

    #[test]
    fn test_language_short_code() {
        assert_eq!(Language::EnUS.short_code(), "en");
        assert_eq!(Language::RuRU.short_code(), "ru");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("en-US"), Some(Language::EnUS));
        assert_eq!(Language::from_code("en-UK"), Some(Language::EnUK));
        assert_eq!(Language::from_code("ru-RU"), Some(Language::RuRU));
        assert_eq!(Language::from_code("fr-FR"), None);
    }
}
