//! Language config.

use crate::config::data;
use crate::dd::Language as LangCode;
use crate::error::{Error, Result};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Language {
    /// Language code.
    pub code: LangCode,
    /// If true, the URL would be /en/, not /en-US/.
    pub use_short: bool,
}

impl Language {
    pub fn full(&self) -> &'static str {
        self.code.full_code()
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.use_short {
            f.write_str(self.code.short_code())
        } else {
            f.write_str(self.code.full_code())
        }
    }
}

pub fn parse_language(d: &data::LanguageData) -> Result<Language> {
    let mut l = Language {
        code: LangCode::EnUS,
        use_short: d.short,
    };

    if d.code.is_empty() {
        return Ok(l);
    }

    if let Some(code) = LangCode::from_code(&d.code) {
        l.code = code;
        return Ok(l);
    }

    Err(Error::UnsupportedLanguage { code: d.code.clone() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_full() {
        let lang = Language { code: LangCode::EnUS, use_short: false };
        assert_eq!(lang.to_string(), "en-US");
    }

    #[test]
    fn test_display_short() {
        let lang = Language { code: LangCode::EnUS, use_short: true };
        assert_eq!(lang.to_string(), "en");
    }

    #[test]
    fn test_display_ru() {
        let lang = Language { code: LangCode::RuRU, use_short: false };
        assert_eq!(lang.to_string(), "ru-RU");
    }
}
