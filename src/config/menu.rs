//! Menu types and parsing.

use crate::config::data;
use crate::config::note_id::NoteIdMatcher;
use crate::dd::{Builtin, NoteId, Tag};
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub enum Menu {
    Builtin { title: String, builtin: Builtin },
    NoteId { title: String, id: NoteId },
    Tag { title: String, tag: Tag },
    Url { title: String, url: String },
}

impl Menu {
    pub fn title(&self) -> &str {
        match self {
            Menu::Builtin { title, .. } => title,
            Menu::NoteId { title, .. } => title,
            Menu::Tag { title, .. } => title,
            Menu::Url { title, .. } => title,
        }
    }
}

fn validate(m: &data::Menu) -> Result<()> {
    let mut filled = 0;
    if !m.builtin.is_empty() { filled += 1; }
    if !m.id.is_empty() { filled += 1; }
    if !m.tag.is_empty() { filled += 1; }
    if !m.url.is_empty() { filled += 1; }

    if filled != 1 {
        return Err(Error::Config("menu entry can only have one type".into()));
    }

    Ok(())
}

pub fn parse_menu(
    m: &data::Menu,
    matcher: &NoteIdMatcher,
    is_tag_published: &dyn Fn(&str) -> bool,
) -> Result<Menu> {
    validate(m)?;

    if !m.builtin.is_empty() {
        let builtin = match m.builtin.as_str() {
            "feed" => Builtin::Feed,
            "search" => Builtin::Search,
            "tags" => Builtin::Tags,
            _ => return Err(Error::Config(format!("unknown builtin '{}'", m.builtin))),
        };
        return Ok(Menu::Builtin { title: m.title.clone(), builtin });
    }

    if !m.id.is_empty() {
        if !matcher.is_valid(&m.id) {
            return Err(Error::Config(format!("invalid note id '{}'", m.id)));
        }
        return Ok(Menu::NoteId { title: m.title.clone(), id: m.id.clone() });
    }

    if !m.tag.is_empty() {
        if !is_tag_published(&m.tag) {
            return Err(Error::Config(format!(
                "non-published tag '{}' in menu",
                m.tag
            )));
        }
        return Ok(Menu::Tag { title: m.title.clone(), tag: m.tag.clone() });
    }

    if !m.url.is_empty() {
        return Ok(Menu::Url { title: m.title.clone(), url: m.url.clone() });
    }

    unreachable!()
}
