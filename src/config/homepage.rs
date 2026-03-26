//! Homepage enum.

use crate::config::data;
use crate::config::note_id::NoteIdMatcher;
use crate::dd::NoteId;
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub enum Homepage {
    Feed,
    NoteId(NoteId),
    Redirect(String),
}

fn is_valid_destination(s: &str) -> bool {
    s.starts_with('/') || s.starts_with("http://") || s.starts_with("https://")
}

pub fn parse_homepage(
    h: &data::Homepage,
    matcher: &NoteIdMatcher,
) -> Result<Homepage> {
    let has_id = !h.id.is_empty();
    let has_file = !h.file.is_empty();
    let has_redirect = !h.redirect.is_empty();

    let set_count = has_id as u8 + has_file as u8 + has_redirect as u8;
    if set_count > 1 {
        return Err(Error::HomepageConflict);
    }

    if has_redirect {
        if !is_valid_destination(&h.redirect) {
            return Err(Error::RedirectInvalidDestination {
                url: "/".to_string(),
                destination: h.redirect.clone(),
            });
        }
        return Ok(Homepage::Redirect(h.redirect.clone()));
    }

    if has_id {
        if matcher.is_valid(&h.id) {
            return Ok(Homepage::NoteId(h.id.clone()));
        }
        return Err(Error::HomepageInvalidNoteId { id: h.id.clone() });
    }

    if has_file {
        return Ok(Homepage::NoteId(h.file.clone()));
    }

    Ok(Homepage::Feed)
}
