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
    if !h.id.is_empty() {
        if matcher.is_valid(&h.id) {
            return Ok(Homepage::NoteId(h.id.clone()));
        }
        return Err(Error::Config(format!("invalid note id '{}'", h.id)));
    }

    Ok(Homepage::Feed)
}
