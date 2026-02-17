//! Website, WebsiteLang, config loading.

pub mod data;
pub mod feed;
pub mod homepage;
pub mod language;
pub mod menu;
pub mod note_id;
pub mod shared_file;
pub mod tag;
mod url;

use crate::dd::{self, Language, Tag};
use crate::error::{Error, Result};
use crate::l10n::{self, L10n};
use note_id::NoteIdMatcher;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Website represents the configuration of a website.
pub struct Website {
    pub main: WebsiteLang,
    pub sub_configs: Vec<WebsiteLang>,
}

/// WebsiteLang represents the configuration of one language of a website.
#[derive(Debug)]
pub struct WebsiteLang {
    pub is_child: bool,
    pub domain: String,
    pub https: bool,
    pub twitter: String,
    pub title: String,
    pub note_ids: NoteIdMatcher,
    pub homepage: homepage::Homepage,
    pub language: language::Language,
    pub tags: HashMap<Tag, tag::TagConfig>,
    pub menu: Vec<menu::Menu>,
    pub feed: feed::Feed,
    pub pages_tag: Tag,
    pub shared_files: Vec<shared_file::SharedFile>,
    pub head_suffix: String,
    pub note_suffix: String,
    pub footer_prefix: String,
    localizer: L10n,
}

impl WebsiteLang {
    pub fn tags_to_published(&self, tags: &[Tag]) -> Vec<tag::TagConfig> {
        let mut result: Vec<tag::TagConfig> = tags
            .iter()
            .filter_map(|t| self.tags.get(t).cloned())
            .collect();
        result.sort_by(|a, b| a.title.cmp(&b.title));
        result
    }

    pub fn str(&self, key: l10n::Key) -> &str {
        self.localizer.str(key)
    }
}

static THEME_CSS: &[u8] = include_bytes!("../../config/files/theme.css");
static FAVICON: &[u8] = include_bytes!("../../config/files/favicon.ico");
static OG_IMAGE: &[u8] = include_bytes!("../../config/files/og.jpg");

impl Website {
    pub fn new(config_dir: &str) -> Result<Self> {
        let mut shared_files = vec![
            shared_file::SharedFile {
                filename: "theme.css".to_string(),
                content: THEME_CSS.to_vec(),
                content_type: "text/css".to_string(),
            },
            shared_file::SharedFile {
                filename: "favicon.ico".to_string(),
                content: FAVICON.to_vec(),
                content_type: "image/svg+xml".to_string(),
            },
            shared_file::SharedFile {
                filename: "og.jpg".to_string(),
                content: OG_IMAGE.to_vec(),
                content_type: "image/jpg".to_string(),
            },
        ];

        for sf in &mut shared_files {
            let keep = sf.filename == "theme.css";
            sf.overload(config_dir, keep);
        }

        let cfg_path = config_path(config_dir, Language::EnUS, false);
        let mut main = new_lang(&cfg_path, Language::EnUS, false)?;
        main.shared_files = shared_files;

        if main.domain.is_empty() {
            return Err(Error::Config(format!(
                "domain field must be set in config file: {}",
                cfg_path.display()
            )));
        }

        let mut sub_configs = Vec::new();
        for &lang in dd::all_languages() {
            if lang == main.language.code {
                continue;
            }

            let path = config_path(config_dir, lang, true);
            if !path.exists() {
                continue;
            }

            let mut cfg = new_lang(&path, lang, true)?;
            cfg.domain = main.domain.clone();
            cfg.https = main.https;
            sub_configs.push(cfg);
        }

        Ok(Website { main, sub_configs })
    }
}

fn config_path(config_dir: &str, lang: Language, is_child: bool) -> PathBuf {
    let dir = Path::new(config_dir);
    let name = if is_child {
        format!("config.{}.toml", lang.full_code())
    } else {
        "config.toml".to_string()
    };
    dir.join(name)
}

fn read_config_file(path: &Path) -> Result<data::ConfigFile> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::Config(format!("could not open config file '{}': {e}", path.display())))?;
    let cfg: data::ConfigFile = toml::from_str(&content)
        .map_err(|e| Error::Config(format!("could not load config file '{}': {e}", path.display())))?;
    Ok(cfg)
}

/// Build a WebsiteLang from a parsed config file, without filesystem access.
/// Used for testing and when config is already in memory.
#[cfg(test)]
pub fn from_config(
    cfg: data::ConfigFile,
    lang: Language,
    is_child: bool,
) -> Result<WebsiteLang> {
    from_config_inner(cfg, lang, is_child)
}

fn new_lang(
    config_path: &Path,
    lang: Language,
    is_child: bool,
) -> Result<WebsiteLang> {
    let cfg = read_config_file(config_path)?;
    from_config_inner(cfg, lang, is_child)
}

fn from_config_inner(
    cfg: data::ConfigFile,
    lang: Language,
    is_child: bool,
) -> Result<WebsiteLang> {
    let language = language::parse_language(&cfg.language)?;

    if is_child && language.code != lang {
        return Err(Error::Config(format!(
            "mismatched language in config: {}",
            language
        )));
    }

    let localizer = L10n::new(language.code)?;

    let note_ids = NoteIdMatcher::new(&cfg.notes.id_format, &cfg.notes.id_link_format)?;

    let homepage = homepage::parse_homepage(&cfg.homepage, &note_ids)?;

    let mut tags = HashMap::new();
    for t in &cfg.tags {
        let parsed = tag::parse_tag(t, &note_ids)?;
        if tags.contains_key(&parsed.tag) {
            return Err(Error::Config(format!(
                "tag '{}' already published",
                parsed.tag
            )));
        }
        tags.insert(parsed.tag.clone(), parsed);
    }

    let is_tag_published = |tag: &str| -> bool { tags.contains_key(tag) };

    let mut menu_items = Vec::new();
    for m in &cfg.menu {
        let parsed = menu::parse_menu(m, &note_ids, &is_tag_published)?;
        menu_items.push(parsed);
    }

    let feed = feed::parse_feed(&cfg.feed, "Feed", &note_ids)?;

    Ok(WebsiteLang {
        is_child,
        domain: cfg.domain,
        https: cfg.https,
        twitter: cfg.twitter,
        title: cfg.title,
        note_ids,
        homepage,
        language,
        tags,
        menu: menu_items,
        feed,
        pages_tag: cfg.pages.tag,
        shared_files: Vec::new(),
        head_suffix: cfg.segments.head_suffix,
        note_suffix: cfg.segments.note_suffix,
        footer_prefix: cfg.segments.footer_prefix,
        localizer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_config() -> data::ConfigFile {
        data::ConfigFile {
            domain: "example.com".to_string(),
            https: true,
            title: "Test".to_string(),
            notes: data::Notes {
                id_format: "[a-z0-9-]+".to_string(),
                id_link_format: "/note/([a-z0-9-]+)".to_string(),
            },
            feed: data::Feed {
                tag: "blog".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_minimal_valid_config() {
        let w = from_config(minimal_config(), Language::EnUS, false).unwrap();
        assert_eq!(w.domain, "example.com");
        assert!(w.https);
        assert_eq!(w.title, "Test");
        assert!(matches!(w.homepage, homepage::Homepage::Feed));
    }

    #[test]
    fn test_empty_code_defaults_to_en_us() {
        let cfg = minimal_config();
        let w = from_config(cfg, Language::EnUS, false).unwrap();
        assert_eq!(w.language.code, Language::EnUS);
    }

    #[test]
    fn test_duplicate_tag_error() {
        let mut cfg = minimal_config();
        cfg.tags = vec![
            data::TagData { tag: "rust".to_string(), slug: "rust".to_string(), title: "Rust".to_string(), ..Default::default() },
            data::TagData { tag: "rust".to_string(), slug: "rust2".to_string(), title: "Rust2".to_string(), ..Default::default() },
        ];
        let err = from_config(cfg, Language::EnUS, false).unwrap_err();
        assert!(err.to_string().contains("already published"));
    }

    #[test]
    fn test_invalid_note_id_in_homepage() {
        let mut cfg = minimal_config();
        cfg.homepage = data::Homepage { id: "INVALID!!!".to_string() };
        let err = from_config(cfg, Language::EnUS, false).unwrap_err();
        assert!(err.to_string().contains("invalid note id"));
    }

    #[test]
    fn test_unsupported_language_error() {
        let mut cfg = minimal_config();
        cfg.language = data::LanguageData { code: "fr-FR".to_string(), short: false };
        let err = from_config(cfg, Language::EnUS, false).unwrap_err();
        assert!(err.to_string().contains("not supported"));
    }

    #[test]
    fn test_menu_validation_error() {
        let mut cfg = minimal_config();
        cfg.menu = vec![data::Menu {
            tag: "nonexistent".to_string(),
            title: "Bad".to_string(),
            ..Default::default()
        }];
        let err = from_config(cfg, Language::EnUS, false).unwrap_err();
        assert!(err.to_string().contains("non-published tag"));
    }
}
