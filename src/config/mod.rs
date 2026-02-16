//! Website, WebsiteLang, config loading.

pub mod data;
pub mod feed;
pub mod homepage;
pub mod language;
pub mod menu;
pub mod shared_file;
pub mod tag;
mod url;

use crate::dd::{self, Language, NoteId, Tag, SUPPORTED_LANGUAGES};
use crate::l10n::{self, L10n};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Website represents the configuration of a website.
pub struct Website {
    pub main: WebsiteLang,
    pub sub_configs: Vec<WebsiteLang>,
}

/// WebsiteLang represents the configuration of one language of a website.
#[allow(dead_code)]
pub struct WebsiteLang {
    pub is_child: bool,
    pub domain: String,
    pub https: bool,
    pub twitter: String,
    pub title: String,
    pub is_valid_note_id: Box<dyn Fn(&str) -> bool + Send + Sync>,
    pub id_from_link: Box<dyn Fn(&str) -> Option<NoteId> + Send + Sync>,
    pub id_from_file: Box<dyn Fn(&str) -> Option<NoteId> + Send + Sync>,
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
    #[allow(dead_code)]
    pub fn is_tag_published(&self, tag: &str) -> bool {
        self.tags.contains_key(tag)
    }

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
    pub fn new(config_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
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
            return Err(format!("domain field must be set in config file: {}", cfg_path.display()).into());
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
        format!("config.{}.toml", SUPPORTED_LANGUAGES[&lang].full)
    } else {
        "config.toml".to_string()
    };
    dir.join(name)
}

fn read_config_file(path: &Path) -> Result<data::ConfigFile, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("could not open config file '{}': {e}", path.display()))?;
    let cfg: data::ConfigFile = toml::from_str(&content)
        .map_err(|e| format!("could not load config file '{}': {e}", path.display()))?;
    Ok(cfg)
}

fn make_note_id_validator(r: &str) -> Result<Box<dyn Fn(&str) -> bool + Send + Sync>, Box<dyn std::error::Error>> {
    let re = Regex::new(r)
        .map_err(|e| format!("could not compile regular expression '{r}': {e}"))?;
    Ok(Box::new(move |test: &str| {
        if let Some(m) = re.find(test) {
            !m.as_str().is_empty() && m.as_str() == test
        } else {
            false
        }
    }))
}

fn make_id_from_link_func(
    r: &str,
    id_format: &str,
) -> Result<Box<dyn Fn(&str) -> Option<NoteId> + Send + Sync>, Box<dyn std::error::Error>> {
    let re = Regex::new(r)
        .map_err(|e| format!("could not compile regular expression '{r}': {e}"))?;

    // Validate extracted IDs against id_format (not id_link_format)
    let valid_re = Regex::new(id_format)
        .map_err(|e| format!("could not compile regular expression '{id_format}': {e}"))?;

    Ok(Box::new(move |link: &str| {
        let caps = re.captures(link)?;
        let id = caps.get(1)?.as_str();
        // Validate with id_format
        if let Some(m) = valid_re.find(id) {
            if !m.as_str().is_empty() && m.as_str() == id {
                return Some(id.to_string());
            }
        }
        None
    }))
}

fn make_id_from_file_func(
    r: &str,
    _is_valid: &(dyn Fn(&str) -> bool + Send + Sync),
) -> Result<Box<dyn Fn(&str) -> Option<NoteId> + Send + Sync>, Box<dyn std::error::Error>> {
    let re = Regex::new(r)
        .map_err(|e| format!("could not compile regular expression '{r}': {e}"))?;

    let valid_re_str = r.to_string();
    let valid_re = Regex::new(&valid_re_str).unwrap();

    Ok(Box::new(move |test: &str| {
        let m = valid_re.find(test)?;
        let id = m.as_str();
        if !id.is_empty() && id == test.trim_end_matches(".md").split('.').next().unwrap_or("") {
            // Actually replicate Go behavior: FindString on test, then check isValid
            let found = valid_re.find(test)?.as_str().to_string();
            if !found.is_empty() {
                // Check is_valid equivalent
                if let Some(vm) = valid_re.find(&found) {
                    if !vm.as_str().is_empty() && vm.as_str() == found {
                        return Some(found);
                    }
                }
            }
        }
        // Simpler: just do what Go does
        let found = re.find(test)?.as_str().to_string();
        if found.is_empty() {
            return None;
        }
        // Check valid
        if let Some(vm) = valid_re.find(&found) {
            if !vm.as_str().is_empty() && vm.as_str() == found {
                return Some(found);
            }
        }
        None
    }))
}

fn new_lang(
    config_path: &Path,
    lang: Language,
    is_child: bool,
) -> Result<WebsiteLang, Box<dyn std::error::Error>> {
    let cfg = read_config_file(config_path)?;

    let language = language::parse_language(&cfg.language)?;

    if is_child && language.code != lang {
        return Err(format!("mismatched language in config: {}", language.to_string()).into());
    }

    let localizer = L10n::new(language.code)?;

    let is_valid_note_id = make_note_id_validator(&cfg.notes.id_format)?;

    let id_from_file = make_id_from_file_func(&cfg.notes.id_format, &*is_valid_note_id)?;
    let id_from_link = make_id_from_link_func(&cfg.notes.id_link_format, &cfg.notes.id_format)?;

    let homepage = homepage::parse_homepage(&cfg.homepage, &*is_valid_note_id)?;

    let mut tags = HashMap::new();
    for t in &cfg.tags {
        let parsed = tag::parse_tag(t, &*is_valid_note_id)?;
        if tags.contains_key(&parsed.tag) {
            return Err(format!("tag '{}' already published", parsed.tag).into());
        }
        tags.insert(parsed.tag.clone(), parsed);
    }

    let is_tag_published = |tag: &str| -> bool { tags.contains_key(tag) };

    let mut menu_items = Vec::new();
    for m in &cfg.menu {
        let parsed = menu::parse_menu(m, &*is_valid_note_id, &is_tag_published)?;
        menu_items.push(parsed);
    }

    let feed = feed::parse_feed(&cfg.feed, "Feed", &*is_valid_note_id)?;

    Ok(WebsiteLang {
        is_child,
        domain: cfg.domain,
        https: cfg.https,
        twitter: cfg.twitter,
        title: cfg.title,
        is_valid_note_id,
        id_from_link,
        id_from_file,
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
