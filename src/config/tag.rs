//! Tag config.

use crate::config::data;
use crate::dd::{NoteId, Tag as DdTag};

#[derive(Debug, Clone)]
pub struct TagConfig {
    pub tag: DdTag,
    pub id: NoteId,
    pub slug: String,
    pub title: String,
}

pub fn parse_tag(
    t: &data::TagData,
    is_valid: &dyn Fn(&str) -> bool,
) -> Result<TagConfig, Box<dyn std::error::Error>> {
    if t.tag.is_empty() {
        return Err("tag in [[tags]] cannot be empty".into());
    }

    let slug = if t.slug.is_empty() {
        t.tag.clone()
    } else {
        t.slug.clone()
    };

    let title = if t.title.is_empty() {
        slug.clone()
    } else {
        t.title.clone()
    };

    if !t.id.is_empty() && !is_valid(&t.id) {
        return Err(format!("invalid note ID '{}' in tag '{}'", t.id, t.tag).into());
    }

    Ok(TagConfig {
        tag: t.tag.clone(),
        id: t.id.clone(),
        slug,
        title,
    })
}
