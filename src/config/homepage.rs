//! Homepage enum.

use crate::config::data;
use crate::config::note_id::NoteIdMatcher;
use crate::dd::NoteId;
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub enum Homepage {
    Feed,
    NoteId(NoteId),
}

pub fn parse_homepage(
    h: &data::Homepage,
    matcher: &NoteIdMatcher,
) -> Result<Homepage> {
    if !h.id.is_empty() && !h.file.is_empty() {
        return Err(Error::HomepageConflict);
    }

    if !h.id.is_empty() {
        if matcher.is_valid(&h.id) {
            return Ok(Homepage::NoteId(h.id.clone()));
        }
        return Err(Error::HomepageInvalidNoteId { id: h.id.clone() });
    }

    if !h.file.is_empty() {
        return Ok(Homepage::NoteId(h.file.clone()));
    }

    Ok(Homepage::Feed)
}
