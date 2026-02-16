//! Homepage enum.

use crate::config::data;
use crate::dd::NoteId;

#[derive(Debug, Clone)]
pub enum Homepage {
    Feed,
    NoteId(NoteId),
}

pub fn parse_homepage(
    h: &data::Homepage,
    is_valid: &dyn Fn(&str) -> bool,
) -> Result<Homepage, Box<dyn std::error::Error>> {
    if !h.id.is_empty() {
        if is_valid(&h.id) {
            return Ok(Homepage::NoteId(h.id.clone()));
        }
        return Err(format!("invalid note id '{}'", h.id).into());
    }

    Ok(Homepage::Feed)
}
