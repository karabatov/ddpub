//! Store, metadata parsing, markdown processing.

pub mod html;
pub mod markdown;
pub mod metadata;
pub mod multirouter;
pub mod multistore;
pub mod router;

use crate::config::{self, WebsiteLang};
use crate::dd::{self, NoteId, Tag};
use crate::error::{Error, Result};
use chrono::NaiveDate;
use std::collections::HashMap;

pub use markdown::html_as_text;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Metadata {
    pub id: NoteId,
    pub filename: String,
    pub date: NaiveDate,
    pub updated_date: NaiveDate,
    pub title: String,
    pub slug: String,
    pub tags: Vec<Tag>,
    pub language: dd::Language,
    pub has_leading_h1: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishTarget {
    Builtin,
    Feed,
    Page,
    Tag,
}

#[derive(Debug, Clone)]
pub struct PublishedNote {
    pub id: NoteId,
    pub target: PublishTarget,
}

#[derive(Debug, Clone)]
pub struct NoteContent {
    pub meta: Metadata,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub link: String,
    pub path: String,
    pub content_type: String,
}

/// A link that could not be resolved, with context for error reporting.
#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub url: String,
    pub text: String,
    pub notes: Vec<String>,
}

/// Store captures the data necessary to publish the notes.
pub struct Store {
    pub meta: HashMap<NoteId, Metadata>,
    pub by_tag: HashMap<Tag, Vec<NoteId>>,
    pub pub_notes: Vec<PublishedNote>,
    pub note_content: HashMap<NoteId, NoteContent>,
    pub files: HashMap<String, FileInfo>,
    /// Unresolved links found in note content.
    pub broken_links: Vec<LinkInfo>,
    /// Resolved internal note links (base URLs without anchors) to validate against routes.
    pub resolved_links: Vec<LinkInfo>,
    /// Notes that failed to parse metadata (filename: reason). These were
    /// skipped during loading — the consumer should decide how to handle them.
    pub warnings: Vec<String>,
}

impl Store {
    pub fn new(w: &WebsiteLang, notes_dir: &str) -> Result<Self> {
        let result = metadata::read_all_metadata(w, notes_dir)?;
        let by_tag = make_notes_by_tag(&result.meta);

        let pub_notes = notes_for_export(w, &by_tag, &result.meta);

        let mut store = Store {
            meta: result.meta,
            by_tag,
            pub_notes,
            note_content: HashMap::new(),
            files: HashMap::new(),
            broken_links: Vec::new(),
            resolved_links: Vec::new(),
            warnings: result.warnings,
        };

        store.read_exported_content(w, notes_dir)?;

        // Check that menu notes exist.
        for m in &w.menu {
            if let config::menu::Menu::NoteId { id, .. } = m
                && !store.is_page_note(w, id)
            {
                return Err(Error::NoteNotPublished { id: id.clone() });
            }
        }

        Ok(store)
    }

    pub fn is_feed_note(&self, w: &WebsiteLang, id: &str) -> bool {
        if w.feed.tag.is_empty() {
            return false;
        }
        if let Some(m) = self.meta.get(id) {
            m.tags.contains(&w.feed.tag)
        } else {
            false
        }
    }

    pub fn is_page_note(&self, w: &WebsiteLang, id: &str) -> bool {
        if w.pages_tag.is_empty() {
            return false;
        }
        if let Some(m) = self.meta.get(id) {
            m.tags.contains(&w.pages_tag)
        } else {
            false
        }
    }

    pub fn notes_for_tag(&self, w: &WebsiteLang, tag: &str) -> Vec<NoteContent> {
        let mut notes: Vec<NoteContent> = self
            .by_tag
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| {
                        if self.is_feed_note(w, id) {
                            self.note_content.get(id).cloned()
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        notes.sort_by(|a, b| b.meta.updated_date.cmp(&a.meta.updated_date));
        notes
    }

    fn read_exported_content(
        &mut self,
        w: &WebsiteLang,
        notes_dir: &str,
    ) -> Result<()> {
        let mut contents: HashMap<NoteId, NoteContent> = HashMap::new();
        let mut new_files: HashMap<String, FileInfo> = HashMap::new();
        let mut broken_links: Vec<LinkInfo> = Vec::new();
        let mut resolved_links: Vec<LinkInfo> = Vec::new();

        let feed_tag = w.feed.tag.clone();
        let pages_tag = w.pages_tag.clone();

        let pub_ids: Vec<NoteId> = self.pub_notes.iter().map(|p| p.id.clone()).collect();

        for pub_id in &pub_ids {
            if contents.contains_key(pub_id) {
                continue;
            }

            let meta = self.meta.get(pub_id).ok_or_else(|| {
                Error::MetadataNotFound { id: pub_id.clone() }
            })?.clone();

            let raw_content = markdown::read_content(&meta.filename, notes_dir)?;

            let is_feed = |id: &str| -> bool {
                is_note_in_tag(id, &feed_tag, &self.meta)
            };
            let is_page = |id: &str| -> bool {
                is_note_in_tag(id, &pages_tag, &self.meta)
            };

            let mut note_broken: HashMap<String, String> = HashMap::new();
            let mut note_resolved: HashMap<String, String> = HashMap::new();

            let rendered = markdown::render_markdown_with_modifications(
                &raw_content,
                w,
                &self.meta,
                notes_dir,
                &mut new_files,
                &mut note_broken,
                &mut note_resolved,
                is_feed,
                is_page,
            );

            fn merge_links(target: &mut Vec<LinkInfo>, source: HashMap<String, String>, filename: &str) {
                for (url, text) in source {
                    if let Some(existing) = target.iter_mut().find(|l| l.url == url) {
                        if !existing.notes.contains(&filename.to_string()) {
                            existing.notes.push(filename.to_string());
                        }
                    } else {
                        target.push(LinkInfo {
                            url,
                            text,
                            notes: vec![filename.to_string()],
                        });
                    }
                }
            }

            merge_links(&mut broken_links, note_broken, &meta.filename);
            merge_links(&mut resolved_links, note_resolved, &meta.filename);

            contents.insert(
                pub_id.clone(),
                NoteContent {
                    meta,
                    content: rendered,
                },
            );
        }

        self.note_content = contents;
        self.files = new_files;
        self.broken_links = broken_links;
        self.resolved_links = resolved_links;
        Ok(())
    }
}

fn is_note_in_tag(id: &str, tag: &str, meta: &HashMap<NoteId, Metadata>) -> bool {
    if tag.is_empty() {
        return false;
    }
    if let Some(m) = meta.get(id) {
        m.tags.iter().any(|t| t == tag)
    } else {
        false
    }
}

fn make_notes_by_tag(meta: &HashMap<NoteId, Metadata>) -> HashMap<Tag, Vec<NoteId>> {
    let mut by_tag: HashMap<Tag, Vec<NoteId>> = HashMap::new();
    for (id, data) in meta {
        for t in &data.tags {
            by_tag.entry(t.clone()).or_default().push(id.clone());
        }
    }
    by_tag
}

fn notes_for_export(
    w: &WebsiteLang,
    by_tag: &HashMap<Tag, Vec<NoteId>>,
    meta: &HashMap<NoteId, Metadata>,
) -> Vec<PublishedNote> {
    let mut e = Vec::new();

    if let config::homepage::Homepage::NoteId(id) = &w.homepage {
        e.push(PublishedNote {
            id: id.clone(),
            target: PublishTarget::Builtin,
        });
    }

    if !w.feed.id.is_empty() {
        e.push(PublishedNote {
            id: w.feed.id.clone(),
            target: PublishTarget::Builtin,
        });
    }

    for t in w.tags.values() {
        if !t.id.is_empty() {
            e.push(PublishedNote {
                id: t.id.clone(),
                target: PublishTarget::Tag,
            });
        }
    }

    if !w.pages_tag.is_empty()
        && let Some(ids) = by_tag.get(&w.pages_tag)
    {
        for id in ids {
            e.push(PublishedNote {
                id: id.clone(),
                target: PublishTarget::Page,
            });
        }
    }

    if !w.feed.tag.is_empty()
        && let Some(ids) = by_tag.get(&w.feed.tag)
    {
        for id in ids {
            if let Some(m) = meta.get(id)
                && m.language == w.language.code
            {
                e.push(PublishedNote {
                    id: id.clone(),
                    target: PublishTarget::Feed,
                });
            }
        }
    }

    e
}
