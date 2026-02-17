//! URL generation methods.

use crate::config::WebsiteLang;
use crate::dd::Builtin;
use sha1::Digest;
use std::path::Path;

impl WebsiteLang {
    fn base_url(&self) -> String {
        if !self.is_child {
            "/".to_string()
        } else {
            format!("/{}/", self.language)
        }
    }

    pub fn url_for_home_page(&self) -> String {
        self.base_url()
    }

    pub fn url_for_builtin(&self, b: Builtin) -> String {
        match b {
            Builtin::Feed => format!("{}{}/", self.base_url(), self.feed.url_prefix),
            Builtin::Search => format!("{}search/", self.base_url()),
            Builtin::Tags => format!("{}tags/", self.base_url()),
        }
    }

    pub fn url_for_tag(&self, t: &super::tag::TagConfig) -> String {
        format!("{}{}/", self.url_for_builtin(Builtin::Tags), t.slug)
    }

    pub fn url_for_page_note(&self, slug: &str) -> String {
        format!("{}{}/", self.base_url(), slug)
    }

    pub fn url_for_feed_note(&self, slug: &str) -> String {
        format!("{}{}/", self.url_for_builtin(Builtin::Feed), slug)
    }

    pub fn url_for_file(&self, file: &str) -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update(file.as_bytes());
        let filename = hex::encode(hasher.finalize());
        let extension = Path::new(file)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        format!("{}files/{}{}", self.base_url(), filename, extension)
    }

    pub fn url_for_shared_file(&self, file: &str) -> String {
        format!("/{file}")
    }

    pub fn url_for_rss_feed(&self) -> String {
        format!("{}rss.xml", self.base_url())
    }

    pub fn absolute_url(&self, pattern: &str) -> String {
        format!("{}://{}{}", self.protocol(), self.domain, pattern)
    }

    fn protocol(&self) -> &str {
        if self.https { "https" } else { "http" }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{self, data};
    use crate::dd::{Builtin, Language};

    fn make_main() -> config::WebsiteLang {
        config::from_config(
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
                    url_prefix: "feed".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Language::EnUS,
            false,
        )
        .unwrap()
    }

    fn make_child() -> config::WebsiteLang {
        config::from_config(
            data::ConfigFile {
                domain: "example.com".to_string(),
                https: false,
                title: "Test RU".to_string(),
                language: data::LanguageData {
                    code: "ru-RU".to_string(),
                    short: false,
                },
                notes: data::Notes {
                    id_format: "[a-z0-9-]+".to_string(),
                    id_link_format: "/note/([a-z0-9-]+)".to_string(),
                },
                feed: data::Feed {
                    tag: "blog".to_string(),
                    url_prefix: "feed".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Language::RuRU,
            true,
        )
        .unwrap()
    }

    #[test]
    fn test_home_page_main() {
        let w = make_main();
        assert_eq!(w.url_for_home_page(), "/");
    }

    #[test]
    fn test_home_page_child() {
        let w = make_child();
        assert_eq!(w.url_for_home_page(), "/ru-RU/");
    }

    #[test]
    fn test_builtin_feed() {
        let w = make_main();
        assert_eq!(w.url_for_builtin(Builtin::Feed), "/feed/");
    }

    #[test]
    fn test_feed_note() {
        let w = make_main();
        assert_eq!(w.url_for_feed_note("my-post"), "/feed/my-post/");
    }

    #[test]
    fn test_file_url_deterministic() {
        let w = make_main();
        let url1 = w.url_for_file("image.png");
        let url2 = w.url_for_file("image.png");
        assert_eq!(url1, url2);
        assert!(url1.starts_with("/files/"));
        assert!(url1.ends_with(".png"));
    }

    #[test]
    fn test_absolute_url_https() {
        let w = make_main();
        assert_eq!(w.absolute_url("/feed/"), "https://example.com/feed/");
    }

    #[test]
    fn test_absolute_url_http() {
        let w = make_child();
        assert_eq!(w.absolute_url("/ru-RU/"), "http://example.com/ru-RU/");
    }

    #[test]
    fn test_rss_url() {
        let w = make_main();
        assert_eq!(w.url_for_rss_feed(), "/rss.xml");
    }
}
