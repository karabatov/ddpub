//! Feed config.

use crate::config::data;
use crate::dd::{NoteId, Tag};

#[derive(Debug, Clone)]
pub struct Feed {
    pub tag: Tag,
    pub url_prefix: String,
    pub id: NoteId,
    pub title: String,
}

pub fn parse_feed(
    f: &data::Feed,
    default_title: &str,
    is_valid: &dyn Fn(&str) -> bool,
) -> Result<Feed, Box<dyn std::error::Error>> {
    let mut feed = Feed {
        tag: f.tag.clone(),
        url_prefix: if f.url_prefix.is_empty() {
            "feed".to_string()
        } else {
            f.url_prefix.clone()
        },
        id: String::new(),
        title: if f.title.is_empty() {
            default_title.to_string()
        } else {
            f.title.clone()
        },
    };

    if !f.id.is_empty() && !is_valid(&f.id) {
        return Err(format!("invalid note ID '{}' in feed", f.id).into());
    }
    feed.id = f.id.clone();

    Ok(feed)
}
