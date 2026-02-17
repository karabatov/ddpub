//! Feed config.

use crate::config::data;
use crate::config::note_id::NoteIdMatcher;
use crate::dd::{NoteId, Tag};
use crate::error::{Error, Result};

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
    matcher: &NoteIdMatcher,
) -> Result<Feed> {
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

    if !f.id.is_empty() && !matcher.is_valid(&f.id) {
        return Err(Error::Config(format!(
            "invalid note ID '{}' in feed",
            f.id
        )));
    }
    feed.id = f.id.clone();

    Ok(feed)
}
