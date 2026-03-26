//! Tag config.

use crate::config::data;
use crate::config::note_id::NoteIdMatcher;
use crate::dd::{NoteId, Tag as DdTag};
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct TagConfig {
    pub tag: DdTag,
    pub id: NoteId,
    pub slug: String,
    pub title: String,
}

pub fn parse_tag(
    t: &data::TagData,
    matcher: &NoteIdMatcher,
) -> Result<TagConfig> {
    if t.tag.is_empty() {
        return Err(Error::EmptyTag);
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

    if !t.id.is_empty() && !t.file.is_empty() {
        return Err(Error::TagConflict { tag: t.tag.clone() });
    }

    if !t.id.is_empty() && !matcher.is_valid(&t.id) {
        return Err(Error::TagInvalidNoteId {
            tag: t.tag.clone(),
            id: t.id.clone(),
        });
    }

    let note_id = if !t.file.is_empty() {
        t.file.clone()
    } else {
        t.id.clone()
    };

    Ok(TagConfig {
        tag: t.tag.clone(),
        id: note_id,
        slug,
        title,
    })
}
