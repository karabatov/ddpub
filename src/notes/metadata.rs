//! Metadata parsing from markdown note files.

use crate::config::WebsiteLang;
use crate::dd::{self, NoteId, Tag};
use crate::error::Result;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use super::Metadata;
use super::markdown::md_to_html;

/// Convert an ID or filename into a URL-safe slug.
fn slugify(s: &str) -> String {
    let s = s.strip_suffix(".md").unwrap_or(s);
    let slug: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-');
    let mut result = String::with_capacity(slug.len());
    let mut prev_dash = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_dash {
                result.push(c);
            }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }
    result
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

    let nome_meta = nome::NoteMetadata::from_reader(io::BufReader::new(file), id)?;

    let title = nome_meta.title.map(|t| md_to_html(&t)).unwrap_or_default();
    let tags: Vec<Tag> = nome_meta.tags;
    let slug = slugify(&nome_meta.slug.filter(|s| !s.is_empty()).unwrap_or_else(|| id.to_string()));

    let date = nome_meta.date.unwrap_or(mod_time);
    let updated_date = nome_meta.updated.unwrap_or(date);

    let language = if let Some(lang_str) = nome_meta.language {
        let (parsed, ok) = dd::parse_language(&lang_str);
        if ok { parsed } else { w.language.code }
    } else if !w.is_child {
        w.language.code
    } else {
        dd::Language::EnUS
    };

    Ok(Metadata {
        id: id.to_string(),
        filename: filename.to_string(),
        date,
        updated_date,
        title,
        slug,
        tags,
        language,
    })
}

pub fn read_all_metadata(
    w: &WebsiteLang,
    notes_dir: &str,
) -> Result<HashMap<NoteId, Metadata>> {
    let entries = fs::read_dir(notes_dir)
        .map_err(|e| crate::error::Error::Note(format!("could not read the notes directory '{notes_dir}': {e}")))?;

    let mut meta = HashMap::new();
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();

        if !filename.ends_with(".md") {
            continue;
        }

        let id = w.note_ids.extract_file(&filename)
            .unwrap_or_else(|| filename.clone());

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
    fn test_tags_parsed_from_metadata() {
        let content = "# Note\nTags: #tag1 #tag2 #tag3\n\nBody";
        let meta = nome::NoteMetadata::parse(content, "id1");
        assert_eq!(meta.tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_tags_empty() {
        let content = "# Note\nTags: \n\nBody";
        let meta = nome::NoteMetadata::parse(content, "id1");
        assert!(meta.tags.is_empty());
    }

    #[test]
    fn test_slugify_simple() {
        assert_eq!(slugify("about.md"), "about");
    }

    #[test]
    fn test_slugify_spaces() {
        assert_eq!(slugify("My Cool Article.md"), "my-cool-article");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("hello_world (copy).md"), "hello-world-copy");
    }

    #[test]
    fn test_slugify_already_clean() {
        assert_eq!(slugify("hello-world"), "hello-world");
    }

    #[test]
    fn test_slugify_numeric_id() {
        assert_eq!(slugify("20241201120000"), "20241201120000");
    }

    #[test]
    fn test_slugify_consecutive_dashes() {
        assert_eq!(slugify("a--b---c.md"), "a-b-c");
    }
}
