//! Store, metadata parsing, markdown processing.

pub mod html;
pub mod multirouter;
pub mod multistore;
pub mod router;

use crate::config::{self, WebsiteLang};
use crate::dd::{self, NoteId, Tag};
use chrono::NaiveDate;
use comrak::arena_tree::Node;
use comrak::nodes::{Ast, NodeValue};
use comrak::{Arena, ComrakOptions, format_html, parse_document};
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::LazyLock;

static MATCH_MARKDOWN_FILE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.md$").unwrap());
static MATCH_LINE_TITLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#\s(.*)$").unwrap());
static MATCH_LINE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Date:\s(.*)\s*$").unwrap());
static MATCH_LINE_UPDATED_DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Updated:\s(.*)\s*$").unwrap());
static MATCH_LINE_LANGUAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Language:\s(.*)\s*$").unwrap());
static MATCH_LINE_SLUG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Slug:\s(.*)\s*$").unwrap());
static MATCH_LINE_TAGS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Tags:\s(.*)\s*$").unwrap());
static MATCH_ONE_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#(\S+)\s*").unwrap());

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

/// Store captures the data necessary to publish the notes.
pub struct Store {
    pub meta: HashMap<NoteId, Metadata>,
    pub by_tag: HashMap<Tag, Vec<NoteId>>,
    pub pub_notes: Vec<PublishedNote>,
    pub note_content: HashMap<NoteId, NoteContent>,
    pub files: HashMap<String, FileInfo>,
}

impl Store {
    pub fn new(w: &WebsiteLang, notes_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let meta = read_all_metadata(w, notes_dir)?;
        let by_tag = make_notes_by_tag(&meta);

        let pub_notes = notes_for_export(w, &by_tag, &meta);

        let mut store = Store {
            meta,
            by_tag,
            pub_notes,
            note_content: HashMap::new(),
            files: HashMap::new(),
        };

        store.read_exported_content(w, notes_dir)?;

        // Check that menu notes exist.
        for m in &w.menu {
            if let config::menu::Menu::NoteId { id, .. } = m {
                if !store.is_page_note(w, id) {
                    return Err(format!("menu note not published: {id}").into());
                }
            }
        }

        Ok(store)
    }

    #[allow(dead_code)]
    pub fn note_exists(&self, w: &WebsiteLang, test: &str) -> bool {
        if !(w.is_valid_note_id)(test) {
            return false;
        }
        self.meta.contains_key(test)
    }

    pub fn is_feed_note(&self, w: &WebsiteLang, id: &str) -> bool {
        if w.feed.tag.is_empty() {
            return false;
        }
        if let Some(m) = self.meta.get(id) {
            m.tags.iter().any(|t| *t == w.feed.tag)
        } else {
            false
        }
    }

    pub fn is_page_note(&self, w: &WebsiteLang, id: &str) -> bool {
        if w.pages_tag.is_empty() {
            return false;
        }
        if let Some(m) = self.meta.get(id) {
            m.tags.iter().any(|t| *t == w.pages_tag)
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut contents: HashMap<NoteId, NoteContent> = HashMap::new();
        let mut new_files: HashMap<String, FileInfo> = HashMap::new();

        let feed_tag = w.feed.tag.clone();
        let pages_tag = w.pages_tag.clone();

        // Collect what we need before the loop to avoid borrow issues
        let pub_ids: Vec<NoteId> = self.pub_notes.iter().map(|p| p.id.clone()).collect();

        for pub_id in &pub_ids {
            if contents.contains_key(pub_id) {
                continue;
            }

            let meta = self.meta.get(pub_id).ok_or_else(|| {
                format!("metadata not found for note '{pub_id}'")
            })?.clone();

            let raw_content = read_content(&meta.filename, notes_dir)?;

            let is_feed = |id: &str| -> bool {
                is_note_in_tag(id, &feed_tag, &self.meta)
            };
            let is_page = |id: &str| -> bool {
                is_note_in_tag(id, &pages_tag, &self.meta)
            };

            let rendered = render_markdown_with_modifications(
                &raw_content,
                w,
                &self.meta,
                notes_dir,
                &mut new_files,
                is_feed,
                is_page,
            );

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

fn read_all_metadata(
    w: &WebsiteLang,
    notes_dir: &str,
) -> Result<HashMap<NoteId, Metadata>, Box<dyn std::error::Error>> {
    let entries = fs::read_dir(notes_dir)
        .map_err(|e| format!("could not read the notes directory '{notes_dir}': {e}"))?;

    let mut meta = HashMap::new();
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();

        let id = match (w.id_from_file)(&filename) {
            Some(id) => id,
            None => continue,
        };

        if !MATCH_MARKDOWN_FILE.is_match(&filename) {
            continue;
        }

        match read_metadata(w, &id, &filename, notes_dir) {
            Ok(m) => {
                meta.insert(id, m);
            }
            Err(_e) => {
                eprintln!("Could not read metadata from file: {filename}");
                continue;
            }
        }
    }

    Ok(meta)
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

fn read_metadata(
    w: &WebsiteLang,
    id: &str,
    filename: &str,
    directory: &str,
) -> Result<Metadata, Box<dyn std::error::Error>> {
    let path = Path::new(directory).join(filename);
    let file = fs::File::open(&path)?;

    let mod_time = file
        .metadata()
        .ok()
        .and_then(|m| {
            m.modified().ok().map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.date_naive()
            })
        })
        .unwrap_or_else(|| chrono::Local::now().date_naive());

    let mut data = Metadata {
        id: id.to_string(),
        filename: filename.to_string(),
        date: NaiveDate::from_ymd_opt(1, 1, 1).unwrap(),
        updated_date: NaiveDate::from_ymd_opt(1, 1, 1).unwrap(),
        title: String::new(),
        slug: String::new(),
        tags: Vec::new(),
        language: dd::Language::EnUS,
    };

    let mut no_lang_seen = true;
    let mut date_set = false;
    let mut updated_set = false;

    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;

        if let Some(title) = dd::first_submatch(&MATCH_LINE_TITLE, &line) {
            data.title = md_to_html(&title);
            continue;
        }

        if let Some(tags_str) = dd::first_submatch(&MATCH_LINE_TAGS, &line) {
            data.tags = tags_from_line(&tags_str);
            continue;
        }

        if let Some(slug) = dd::first_submatch(&MATCH_LINE_SLUG, &line) {
            data.slug = if slug.is_empty() {
                id.to_string()
            } else {
                slug
            };
            continue;
        }

        if let Some(lang_str) = dd::first_submatch(&MATCH_LINE_LANGUAGE, &line) {
            no_lang_seen = false;
            let (parsed, ok) = dd::parse_language(&lang_str);
            data.language = if ok { parsed } else { w.language.code };
            continue;
        }

        if let Some(date_str) = dd::first_submatch(&MATCH_LINE_DATE, &line) {
            match NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d") {
                Ok(d) => {
                    data.date = d;
                    date_set = true;
                }
                Err(_) => {
                    data.date = mod_time;
                    date_set = true;
                }
            }
            continue;
        }

        if let Some(date_str) = dd::first_submatch(&MATCH_LINE_UPDATED_DATE, &line) {
            match NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d") {
                Ok(d) => {
                    data.updated_date = d;
                    updated_set = true;
                }
                Err(_) => {
                    // Default to date if parsing fails
                    updated_set = false;
                }
            }
            continue;
        }

        // If no matchers match, we are done.
        break;
    }

    if !date_set {
        data.date = mod_time;
    }

    if !updated_set {
        data.updated_date = data.date;
    }

    if data.slug.is_empty() {
        data.slug = id.to_string();
    }

    if no_lang_seen && !w.is_child {
        data.language = w.language.code;
    }

    Ok(data)
}

fn tags_from_line(line: &str) -> Vec<Tag> {
    MATCH_ONE_TAG
        .captures_iter(line)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn notes_for_export(
    w: &WebsiteLang,
    by_tag: &HashMap<Tag, Vec<NoteId>>,
    meta: &HashMap<NoteId, Metadata>,
) -> Vec<PublishedNote> {
    let mut e = Vec::new();

    // Add homepage note ID if present.
    if let config::homepage::Homepage::NoteId(id) = &w.homepage {
        e.push(PublishedNote {
            id: id.clone(),
            target: PublishTarget::Builtin,
        });
    }

    // Add feed's note ID if present.
    if !w.feed.id.is_empty() {
        e.push(PublishedNote {
            id: w.feed.id.clone(),
            target: PublishTarget::Builtin,
        });
    }

    // Add all named notes from tags.
    for t in w.tags.values() {
        if !t.id.is_empty() {
            e.push(PublishedNote {
                id: t.id.clone(),
                target: PublishTarget::Tag,
            });
        }
    }

    // Add all notes with pages tag.
    if !w.pages_tag.is_empty() {
        if let Some(ids) = by_tag.get(&w.pages_tag) {
            for id in ids {
                e.push(PublishedNote {
                    id: id.clone(),
                    target: PublishTarget::Page,
                });
            }
        }
    }

    // Add all notes with feed tag.
    if !w.feed.tag.is_empty() {
        if let Some(ids) = by_tag.get(&w.feed.tag) {
            for id in ids {
                if let Some(m) = meta.get(id) {
                    if m.language == w.language.code {
                        e.push(PublishedNote {
                            id: id.clone(),
                            target: PublishTarget::Feed,
                        });
                    }
                }
            }
        }
    }

    e
}

fn read_content(filename: &str, directory: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = Path::new(directory).join(filename);
    let file = fs::File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut content = String::new();
    let mut append_line = false;

    for line in reader.lines() {
        let line = line?;
        if !append_line {
            append_line = line.is_empty();
            continue;
        }
        content.push_str(&line);
        content.push('\n');
    }

    Ok(content)
}

fn render_markdown_with_modifications(
    content: &str,
    w: &WebsiteLang,
    meta: &HashMap<NoteId, Metadata>,
    notes_dir: &str,
    files: &mut HashMap<String, FileInfo>,
    is_feed_note: impl Fn(&str) -> bool,
    is_page_note: impl Fn(&str) -> bool,
) -> String {
    let arena = Arena::new();

    let mut options = ComrakOptions::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.header_ids = Some(String::new());
    options.parse.smart = true;
    options.render.unsafe_ = true;

    let root = parse_document(&arena, content, &options);

    // Walk AST and modify links/images
    fn walk_nodes<'a>(
        node: &'a Node<'a, RefCell<Ast>>,
        w: &WebsiteLang,
        meta: &HashMap<NoteId, Metadata>,
        notes_dir: &str,
        files: &mut HashMap<String, FileInfo>,
        is_feed_note: &dyn Fn(&str) -> bool,
        is_page_note: &dyn Fn(&str) -> bool,
    ) {
        match &mut node.data.borrow_mut().value {
            NodeValue::Link(link) => {
                let link_str = link.url.clone();

                // Parse URL — Go's url.Parse handles both absolute and relative.
                // We split path and fragment manually since url::Url requires absolute URLs.
                let (path_part, fragment) = if let Some(idx) = link_str.find('#') {
                    (&link_str[..idx], Some(&link_str[idx + 1..]))
                } else {
                    (link_str.as_str(), None)
                };

                // Try to extract note ID from the path.
                if let Some(id) = (w.id_from_link)(path_part) {
                    if let Some(linked_meta) = meta.get(&id) {
                        let mut new_link = link_str.clone();
                        if is_feed_note(&id) {
                            new_link = w.url_for_feed_note(&linked_meta.slug);
                        } else if is_page_note(&id) {
                            new_link = w.url_for_page_note(&linked_meta.slug);
                        }
                        // Append fragment if present.
                        if let Some(f) = fragment {
                            if !f.is_empty() {
                                new_link.push('#');
                                new_link.push_str(f);
                            }
                        }
                        link.url = new_link;
                    }
                }

                // Only try file link for relative (non-external) URLs.
                let is_abs = link_str.starts_with("http://") || link_str.starts_with("https://");
                if !is_abs {
                    if let Some(file_info) = try_file_from_link(&link.url, notes_dir, w) {
                        link.url = file_info.link.clone();
                        files.insert(file_info.link.clone(), file_info);
                    }
                }
            }
            NodeValue::Image(link) => {
                let link_str = link.url.clone();
                if let Some(file_info) = try_file_from_link(&link_str, notes_dir, w) {
                    link.url = file_info.link.clone();
                    files.insert(file_info.link.clone(), file_info);
                }
            }
            _ => {}
        }

        for child in node.children() {
            walk_nodes(child, w, meta, notes_dir, files, is_feed_note, is_page_note);
        }
    }

    walk_nodes(root, w, meta, notes_dir, files, &is_feed_note, &is_page_note);

    // Render to HTML
    let mut html_output = Vec::new();
    format_html(root, &options, &mut html_output).unwrap();

    // Post-process: add target="_blank" to external links
    let html_str = String::from_utf8_lossy(&html_output).to_string();
    add_target_blank_to_external_links(&html_str)
}

fn add_target_blank_to_external_links(html: &str) -> String {
    // Add target="_blank" to links that start with http:// or https://
    let re = Regex::new(r#"<a href="(https?://[^"]*)">"#).unwrap();
    re.replace_all(html, r#"<a href="$1" target="_blank">"#).to_string()
}

fn try_file_from_link(link: &str, notes_dir: &str, w: &WebsiteLang) -> Option<FileInfo> {
    // Not a file if it looks like a URL
    if link.starts_with("http://") || link.starts_with("https://") {
        return None;
    }

    // Remove fragment
    let path_part = if let Some(idx) = link.find('#') {
        &link[..idx]
    } else {
        link
    };

    let path = Path::new(notes_dir).join(path_part);
    if !path.exists() {
        return None;
    }

    let content_type = guess_file_content_type(&path);
    let new_link = w.url_for_file(&path.to_string_lossy());

    Some(FileInfo {
        link: new_link,
        path: path.to_string_lossy().to_string(),
        content_type,
    })
}

fn guess_file_content_type(path: &Path) -> String {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn md_to_html(md: &str) -> String {
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    options.parse.smart = true;
    options.render.unsafe_ = true;

    let root = parse_document(&arena, md, &options);
    let mut html = Vec::new();
    format_html(root, &options, &mut html).unwrap();

    // Strip wrapping <p> tags
    let s = String::from_utf8_lossy(&html).to_string();
    let s = s.trim();
    let s = s.strip_prefix("<p>").unwrap_or(s);
    let s = s.strip_suffix("</p>").unwrap_or(s);
    s.to_string()
}

/// Strip HTML tags for plain text (used for RSS titles).
pub fn html_as_text(html: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(html, "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tags_from_line() {
        let tags = tags_from_line("#tag1 #tag2 #tag3");
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_tags_from_line_empty() {
        let tags = tags_from_line("");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_md_to_html() {
        let result = md_to_html("Hello *world*");
        assert!(result.contains("Hello"));
        assert!(result.contains("<em>world</em>"));
    }

    #[test]
    fn test_html_as_text() {
        assert_eq!(html_as_text("<p>Hello <em>world</em></p>"), "Hello world");
    }
}
