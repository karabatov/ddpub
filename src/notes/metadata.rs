//! Metadata parsing from markdown note files.

use crate::config::WebsiteLang;
use crate::dd::{self, NoteId, Tag};
use crate::error::Result;
use chrono::NaiveDate;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::LazyLock;

static MATCH_LINE_TITLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#\s(.*)$").unwrap());
static MATCH_LINE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Date:\s(.*)\s*$").unwrap());
static MATCH_LINE_UPDATED_DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Updated:\s(.*)\s*$").unwrap());
static MATCH_LINE_LANGUAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Language:\s(.*)\s*$").unwrap());
static MATCH_LINE_SLUG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Slug:\s(.*)\s*$").unwrap());
static MATCH_LINE_TAGS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Tags:\s(.*)\s*$").unwrap());
static MATCH_ONE_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#(\S+)\s*").unwrap());

use super::Metadata;
use super::markdown::md_to_html;

pub fn tags_from_line(line: &str) -> Vec<Tag> {
    MATCH_ONE_TAG
        .captures_iter(line)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

pub fn read_metadata(
    w: &WebsiteLang,
    id: &str,
    filename: &str,
    directory: &str,
) -> Result<Metadata> {
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

pub fn read_all_metadata(
    w: &WebsiteLang,
    notes_dir: &str,
) -> Result<HashMap<NoteId, Metadata>> {
    static MATCH_MARKDOWN_FILE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.md$").unwrap());

    let entries = fs::read_dir(notes_dir)
        .map_err(|e| crate::error::Error::Note(format!("could not read the notes directory '{notes_dir}': {e}")))?;

    let mut meta = HashMap::new();
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();

        let id = match w.note_ids.extract_file(&filename) {
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
}
