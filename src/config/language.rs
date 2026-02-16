//! Language config.

use crate::config::data;
use crate::dd::{Language as LangCode, SUPPORTED_LANGUAGES};

#[derive(Debug, Clone)]
pub struct Language {
    /// Language code.
    pub code: LangCode,
    /// If true, the URL would be /en/, not /en-US/.
    pub use_short: bool,
}

impl Language {
    pub fn full(&self) -> &'static str {
        SUPPORTED_LANGUAGES[&self.code].full
    }

    pub fn to_string(&self) -> String {
        let s = &SUPPORTED_LANGUAGES[&self.code];
        if self.use_short {
            s.short.to_string()
        } else {
            s.full.to_string()
        }
    }
}

pub fn parse_language(d: &data::LanguageData) -> Result<Language, Box<dyn std::error::Error>> {
    let mut l = Language {
        code: LangCode::EnUS,
        use_short: d.short,
    };

    if d.code.is_empty() {
        return Ok(l);
    }

    for (&code, s) in SUPPORTED_LANGUAGES.iter() {
        if s.full == d.code {
            l.code = code;
            return Ok(l);
        }
    }

    Err(format!("language '{}' not supported", d.code).into())
}
