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
            format!("/{}/", self.language.to_string())
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
