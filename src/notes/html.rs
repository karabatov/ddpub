//! HTML content generation functions.

use crate::config::WebsiteLang;
use crate::error::{Error, Result};
use crate::l10n::Key;
use crate::layout;
use crate::notes::{NoteContent, Store};

pub fn html_for_page(note: &NoteContent) -> Result<String> {
    layout::fill_content_page(&layout::ContentPage {
        title: note.meta.title.clone(),
        content: note.content.clone(),
        has_leading_h1: note.meta.has_leading_h1,
    })
}

pub fn feed_notes_list_items(
    tag: &str,
    w: &WebsiteLang,
    s: &Store,
) -> Vec<layout::NoteListItem> {
    s.notes_for_tag(w, tag)
        .iter()
        .map(|n| layout::NoteListItem {
            list_item: layout::ListItem {
                title: n.meta.title.clone(),
                url: w.url_for_feed_note(&n.meta.slug),
            },
            date: date_for_note(n, w),
        })
        .collect()
}

pub fn tags_list_items(w: &WebsiteLang, s: &Store) -> Vec<layout::TagListItem> {
    let mut tags: Vec<layout::TagListItem> = w
        .tags
        .values()
        .map(|t| {
            let count = feed_notes_list_items(&t.tag, w, s).len();
            layout::TagListItem {
                list_item: layout::ListItem {
                    title: t.title.clone(),
                    url: w.url_for_tag(t),
                },
                count,
            }
        })
        .collect();

    tags.sort_by(|a, b| b.count.cmp(&a.count));
    tags
}

pub fn html_for_builtin_feed(w: &WebsiteLang, s: &Store) -> Result<String> {
    let content = if !w.feed.id.is_empty() {
        s.note_content
            .get(&w.feed.id)
            .map(|n| n.content.clone())
            .ok_or_else(|| Error::NoteContentNotFound { id: w.feed.id.clone() })?
    } else {
        String::new()
    };

    layout::fill_builtin_feed(&layout::BuiltinFeed {
        title: w.feed.title.clone(),
        content,
        notes: feed_notes_list_items(&w.feed.tag, w, s),
    })
}

pub fn html_for_builtin_tags(w: &WebsiteLang, s: &Store) -> Result<String> {
    layout::fill_builtin_tags(&layout::BuiltinTags {
        title: w.str(Key::TagsTitle).to_string(),
        tags: tags_list_items(w, s),
    })
}

pub fn html_for_tag(
    t: &crate::config::tag::TagConfig,
    w: &WebsiteLang,
    s: &Store,
) -> Result<String> {
    let (content, has_leading_h1) = if !t.id.is_empty() {
        let note = s.note_content
            .get(&t.id)
            .ok_or_else(|| Error::NoteContentNotFound { id: t.id.clone() })?;
        (note.content.clone(), note.meta.has_leading_h1)
    } else {
        (String::new(), false)
    };

    layout::fill_content_tag(&layout::ContentTagPage {
        title: t.title.clone(),
        content,
        notes: feed_notes_list_items(&t.tag, w, s),
        has_leading_h1,
    })
}

pub fn html_for_note(note: &NoteContent, w: &WebsiteLang) -> Result<String> {
    let tags: Vec<layout::ListItem> = w
        .tags_to_published(&note.meta.tags)
        .iter()
        .map(|t| layout::ListItem {
            title: t.title.clone(),
            url: w.url_for_tag(t),
        })
        .collect();

    layout::fill_content_note(&layout::ContentNote {
        title: note.meta.title.clone(),
        date: date_for_note(note, w),
        content: note.content.clone(),
        tags,
        suffix: w.note_suffix.clone(),
        has_leading_h1: note.meta.has_leading_h1,
    })
}

pub fn date_for_note(note: &NoteContent, w: &WebsiteLang) -> String {
    let date_format = w.str(Key::DateFormat);
    let date_pub = note.meta.date.format(date_format).to_string();

    if note.meta.updated_date != note.meta.date {
        let date_upd = note.meta.updated_date.format(date_format).to_string();
        w.str(Key::DateUpdatedPublished)
            .replacen("%s", &date_upd, 1)
            .replacen("%s", &date_pub, 1)
    } else {
        w.str(Key::DatePublished).replacen("%s", &date_pub, 1)
    }
}
