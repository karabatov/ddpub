//! TOML serde structs (mirrors config/internal/data/data.go).

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Homepage {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub file: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct LanguageData {
    #[serde(default, rename = "Code")]
    pub code: String,
    #[serde(default)]
    pub short: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Feed {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub url_prefix: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Pages {
    #[serde(default)]
    pub tag: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Menu {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub builtin: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct TagData {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Segments {
    #[serde(default)]
    pub head_suffix: String,
    #[serde(default)]
    pub note_suffix: String,
    #[serde(default)]
    pub footer_prefix: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Notes {
    #[serde(default)]
    pub id_format: String,
    #[serde(default)]
    pub id_link_format: String,
}

/// ConfigFile represents a TOML configuration file for a single website.
#[derive(Debug, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub domain: String,
    #[serde(default, rename = "https")]
    pub https: bool,
    #[serde(default)]
    pub twitter: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub language: LanguageData,
    #[serde(default)]
    pub feed: Feed,
    #[serde(default)]
    pub pages: Pages,
    #[serde(default)]
    pub homepage: Homepage,
    #[serde(default)]
    pub menu: Vec<Menu>,
    #[serde(default)]
    pub notes: Notes,
    #[serde(default)]
    pub tags: Vec<TagData>,
    #[serde(default)]
    pub segments: Segments,
}
