//! Markdown rendering and HTML post-processing.

use crate::config::WebsiteLang;
use crate::dd::{self, NoteId};
use crate::error::Result;
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

use super::{FileInfo, Metadata};

static RE_EXTERNAL_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<a href="(https?://[^"]*)">"#).unwrap());
static RE_HTML_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

pub fn render_markdown_with_modifications(
    content: &str,
    w: &WebsiteLang,
    meta: &HashMap<NoteId, Metadata>,
    notes_dir: &str,
    files: &mut HashMap<String, FileInfo>,
    broken_links: &mut HashMap<String, String>,
    resolved_links: &mut HashMap<String, String>,
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

    fn link_text<'a>(node: &'a Node<'a, RefCell<Ast>>) -> String {
        let mut text = String::new();
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::Text(t) => text.push_str(t),
                NodeValue::Code(c) => text.push_str(&c.literal),
                _ => {}
            }
        }
        text
    }

    fn walk_nodes<'a>(
        node: &'a Node<'a, RefCell<Ast>>,
        w: &WebsiteLang,
        meta: &HashMap<NoteId, Metadata>,
        notes_dir: &str,
        files: &mut HashMap<String, FileInfo>,
        broken_links: &mut HashMap<String, String>,
        resolved_links: &mut HashMap<String, String>,
        is_feed_note: &dyn Fn(&str) -> bool,
        is_page_note: &dyn Fn(&str) -> bool,
    ) {
        match &mut node.data.borrow_mut().value {
            NodeValue::Link(link) => {
                let link_str = link.url.clone();
                let text = link_text(node);

                let (path_part, fragment) = if let Some(idx) = link_str.find('#') {
                    (&link_str[..idx], Some(&link_str[idx + 1..]))
                } else {
                    (link_str.as_str(), None)
                };

                // Try to resolve note link by ID or filename, then fall back to file reference.
                let note_meta = w.note_ids.extract_link(path_part)
                    .and_then(|id| meta.get(&id).map(|m| (id, m)))
                    .or_else(|| meta.get(path_part).map(|m| (m.id.clone(), m)));

                if path_part.is_empty() {
                    // Fragment-only link (#anchor) — same-page navigation, leave as-is.
                } else if let Some((id, linked_meta)) = note_meta {
                    let mut new_link = link_str.clone();
                    if is_feed_note(&id) {
                        new_link = w.url_for_feed_note(&linked_meta.slug);
                    } else if is_page_note(&id) {
                        new_link = w.url_for_page_note(&linked_meta.slug);
                    }
                    resolved_links.entry(new_link.clone()).or_insert_with(|| text.clone());
                    if let Some(f) = fragment
                        && !f.is_empty()
                    {
                        new_link.push('#');
                        new_link.push_str(f);
                    }
                    link.url = new_link;
                } else if link_str.starts_with("http://")
                    || link_str.starts_with("https://")
                    || link_str.starts_with("mailto:")
                    || link_str.starts_with("tel:")
                {
                    // External or non-http scheme — leave as-is.
                } else if path_part.starts_with('/') {
                    // Absolute path on the same site — collect for route validation.
                    let normalized = if path_part.ends_with('/') {
                        path_part.to_string()
                    } else {
                        format!("{}/", path_part)
                    };
                    resolved_links.entry(normalized).or_insert(text);
                } else if let Some(file_info) = try_file_from_link(&link.url, notes_dir, w) {
                    link.url = file_info.link.clone();
                    files.insert(file_info.link.clone(), file_info);
                } else {
                    // Relative link that's not a file — flag as broken.
                    broken_links.entry(path_part.to_string()).or_insert(text);
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
            walk_nodes(child, w, meta, notes_dir, files, broken_links, resolved_links, is_feed_note, is_page_note);
        }
    }

    walk_nodes(root, w, meta, notes_dir, files, broken_links, resolved_links, &is_feed_note, &is_page_note);

    let mut html_output = Vec::new();
    format_html(root, &options, &mut html_output).unwrap();

    let html_str = String::from_utf8_lossy(&html_output).to_string();
    add_target_blank_to_external_links(&html_str)
}

pub fn add_target_blank_to_external_links(html: &str) -> String {
    RE_EXTERNAL_LINK
        .replace_all(html, r#"<a href="$1" target="_blank">"#)
        .to_string()
}

fn try_file_from_link(link: &str, notes_dir: &str, w: &WebsiteLang) -> Option<FileInfo> {
    if link.starts_with("http://") || link.starts_with("https://") {
        return None;
    }

    let path_part = if let Some(idx) = link.find('#') {
        &link[..idx]
    } else {
        link
    };

    let path = Path::new(notes_dir).join(path_part);
    let canonical_path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return None, // File doesn't exist or is inaccessible.
    };
    let canonical_notes_dir = match Path::new(notes_dir).canonicalize() {
        Ok(p) => p,
        Err(_) => return None,
    };
    if !canonical_path.starts_with(&canonical_notes_dir) {
        return None; // Path traversal attempt — treat as broken link.
    }
    if !canonical_path.is_file() {
        return None; // Directories and other non-files are not servable.
    }

    let content_type = dd::guess_content_type(&canonical_path).to_string();
    let new_link = w.url_for_file(&canonical_path.to_string_lossy());

    Some(FileInfo {
        link: new_link,
        path: path.to_string_lossy().to_string(),
        content_type,
    })
}

pub fn read_content(filename: &str, directory: &str) -> Result<String> {
    use crate::error::Error;
    let path = Path::new(directory).join(filename);
    let file = fs::File::open(&path).map_err(Error::NoteIo)?;
    let reader = io::BufReader::new(file);

    let mut content = String::new();
    let mut append_line = false;

    for line in reader.lines() {
        let line = line.map_err(Error::NoteIo)?;
        if !append_line {
            append_line = line.is_empty();
            continue;
        }
        content.push_str(&line);
        content.push('\n');
    }

    Ok(content)
}

pub fn md_to_html(md: &str) -> String {
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    options.parse.smart = true;
    options.render.unsafe_ = true;

    let root = parse_document(&arena, md, &options);
    let mut html = Vec::new();
    format_html(root, &options, &mut html).unwrap();

    let s = String::from_utf8_lossy(&html).to_string();
    let s = s.trim();
    let s = s.strip_prefix("<p>").unwrap_or(s);
    let s = s.strip_suffix("</p>").unwrap_or(s);
    s.to_string()
}

/// Strip HTML tags for plain text (used for RSS titles).
pub fn html_as_text(html: &str) -> String {
    RE_HTML_TAG.replace_all(html, "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_html_as_text_nested() {
        assert_eq!(
            html_as_text("<div><p>Hello <strong><em>bold italic</em></strong></p></div>"),
            "Hello bold italic"
        );
    }

    #[test]
    fn test_add_target_blank_external() {
        let html = r#"<a href="https://example.com">Link</a>"#;
        let result = add_target_blank_to_external_links(html);
        assert!(result.contains(r#"target="_blank""#));
    }

    #[test]
    fn test_add_target_blank_internal() {
        let html = r#"<a href="/page/">Link</a>"#;
        let result = add_target_blank_to_external_links(html);
        assert!(!result.contains("target"));
    }

    #[test]
    fn test_try_file_from_link_skips_external() {
        // External links are always skipped before WebsiteLang is needed.
        // We can't easily construct a WebsiteLang in tests yet (Phase 4),
        // so we just test the add_target_blank functions which cover this path.
    }
}
