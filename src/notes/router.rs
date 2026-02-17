//! Router, handler generation, RSS.

use crate::config::{self, WebsiteLang};
use crate::dd::Builtin;
use crate::error::{Error, Result};
use crate::l10n::Key;
use crate::layout;
use crate::notes::html::*;
use crate::notes::{PublishTarget, Store, html_as_text};
use chrono::Utc;
use std::collections::HashMap;

/// A route maps a URL pattern to pre-rendered content with a content type.
#[derive(Clone)]
pub struct Route {
    pub content: Vec<u8>,
    pub content_type: String,
}

pub struct Router {
    pub routes: HashMap<String, Route>,
}

fn make_page(w: &WebsiteLang, s: &Store, pattern: &str, title: &str, content: String) -> Result<Vec<u8>> {
    let page = layout::Page {
        language: w.language.to_string(),
        head: layout::Head {
            title: title.to_string(),
            website_title: w.title.clone(),
            meta_tags: layout::MetaTags {
                title: title.to_string(),
                type_: "website".to_string(),
                image: w.absolute_url(&w.url_for_shared_file("og.jpg")),
                url: w.absolute_url(pattern),
                locale: w.language.full().to_string(),
                site_name: w.title.clone(),
                twitter: w.twitter.clone(),
            },
            rss_url: w.url_for_rss_feed(),
            suffix: w.head_suffix.clone(),
        },
        header: layout::Header {
            homepage_url: w.url_for_home_page(),
            title: w.title.clone(),
            menu: layout_menu(w, s)?,
        },
        content,
        footer: layout::Footer {
            prefix: w.footer_prefix.clone(),
            powered_by: w.str(Key::FooterPoweredBy).to_string(),
            rss_url: w.url_for_rss_feed(),
        },
    };
    Ok(layout::fill_page(&page))
}

fn add_page(
    routes: &mut HashMap<String, Route>,
    w: &WebsiteLang,
    s: &Store,
    pattern: &str,
    title: &str,
    content: String,
) -> Result<()> {
    if routes.contains_key(pattern) {
        return Err(Error::Route(format!(
            "pattern '{pattern}' already registered with router"
        )));
    }
    let bytes = make_page(w, s, pattern, title, content)?;
    routes.insert(
        pattern.to_string(),
        Route {
            content: bytes,
            content_type: "text/html; charset=utf-8".to_string(),
        },
    );
    Ok(())
}

impl Router {
    pub fn new(w: &WebsiteLang, s: &Store) -> Result<Self> {
        let mut routes: HashMap<String, Route> = HashMap::new();

        // Add homepage.
        match &w.homepage {
            config::homepage::Homepage::NoteId(id) => {
                let note = s.note_content.get(id).ok_or_else(|| {
                    Error::Route(format!("homepage note content not found: {id}"))
                })?;
                let content = html_for_page(note);
                add_page(&mut routes, w, s, &w.url_for_home_page(), &html_as_text(&note.meta.title), content)?;
            }
            config::homepage::Homepage::Feed => {
                let content = html_for_builtin_feed(w, s);
                add_page(&mut routes, w, s, &w.url_for_home_page(), &w.feed.title, content)?;
            }
        }

        // Builtin - feed.
        {
            let pattern = w.url_for_builtin(Builtin::Feed);
            if !routes.contains_key(&pattern) {
                let content = html_for_builtin_feed(w, s);
                add_page(&mut routes, w, s, &pattern, &w.feed.title, content)?;
            }
        }

        // Builtin - tags.
        {
            let pattern = w.url_for_builtin(Builtin::Tags);
            let content = html_for_builtin_tags(w, s);
            add_page(&mut routes, w, s, &pattern, w.str(Key::TagsTitle), content)?;
        }

        // Add published pages and notes.
        for p in &s.pub_notes {
            let note = match s.note_content.get(&p.id) {
                Some(n) => n,
                None => continue,
            };
            match p.target {
                PublishTarget::Builtin | PublishTarget::Tag => continue,
                PublishTarget::Feed => {
                    let pattern = w.url_for_feed_note(&note.meta.slug);
                    if routes.contains_key(&pattern) {
                        continue;
                    }
                    let content = html_for_note(note, w);
                    add_page(&mut routes, w, s, &pattern, &html_as_text(&note.meta.title), content)?;
                }
                PublishTarget::Page => {
                    let pattern = w.url_for_page_note(&note.meta.slug);
                    if routes.contains_key(&pattern) {
                        continue;
                    }
                    let content = html_for_page(note);
                    add_page(&mut routes, w, s, &pattern, &html_as_text(&note.meta.title), content)?;
                }
            }
        }

        // Add published tags.
        for t in w.tags.values() {
            let pattern = w.url_for_tag(t);
            if routes.contains_key(&pattern) {
                continue;
            }
            let content = html_for_tag(t, w, s);
            add_page(&mut routes, w, s, &pattern, &t.title, content)?;
        }

        // Add shared files (only for main config).
        if !w.is_child {
            for f in &w.shared_files {
                let pattern = w.url_for_shared_file(&f.filename);
                if routes.contains_key(&pattern) {
                    return Err(Error::Route(format!(
                        "pattern '{pattern}' already registered with router"
                    )));
                }
                routes.insert(
                    pattern,
                    Route {
                        content: f.content.clone(),
                        content_type: f.content_type.clone(),
                    },
                );
            }
        }

        // Add files.
        for f in s.files.values() {
            if routes.contains_key(&f.link) {
                continue;
            }
            let content = std::fs::read(&f.path).unwrap_or_default();
            routes.insert(
                f.link.clone(),
                Route {
                    content,
                    content_type: f.content_type.clone(),
                },
            );
        }

        // Add RSS.
        let rss_content = build_rss_feed(w, s);
        let rss_pattern = w.url_for_rss_feed();
        routes.insert(
            rss_pattern,
            Route {
                content: rss_content.into_bytes(),
                content_type: "application/rss+xml".to_string(),
            },
        );

        Ok(Router { routes })
    }
}

fn layout_menu(w: &WebsiteLang, s: &Store) -> Result<Vec<layout::ListItem>> {
    w.menu
        .iter()
        .map(|m| {
            let url = match m {
                config::menu::Menu::Builtin { builtin, .. } => w.url_for_builtin(*builtin),
                config::menu::Menu::NoteId { id, .. } => {
                    let note = s.note_content.get(id).ok_or_else(|| {
                        Error::Route(format!("menu note content not found: {id}"))
                    })?;
                    w.url_for_page_note(&note.meta.slug)
                }
                config::menu::Menu::Tag { tag, .. } => {
                    let tag_config = w.tags.get(tag).ok_or_else(|| {
                        Error::Route(format!("menu tag not found: {tag}"))
                    })?;
                    w.url_for_tag(tag_config)
                }
                config::menu::Menu::Url { url, .. } => url.clone(),
            };
            Ok(layout::ListItem {
                title: m.title().to_string(),
                url,
            })
        })
        .collect()
}

fn build_rss_feed(w: &WebsiteLang, s: &Store) -> String {
    let mut items = Vec::new();

    for p in &s.pub_notes {
        if p.target != PublishTarget::Feed {
            continue;
        }
        let note = match s.note_content.get(&p.id) {
            Some(n) => n,
            None => continue,
        };

        let link = w.absolute_url(&w.url_for_feed_note(&note.meta.slug));

        let pub_date = note
            .meta
            .updated_date
            .and_hms_opt(0, 0, 0)
            .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .map(|dt| dt.to_rfc2822());

        let item = rss::ItemBuilder::default()
            .title(Some(html_as_text(&note.meta.title)))
            .link(Some(link.clone()))
            .guid(Some(rss::GuidBuilder::default().value(link).build()))
            .pub_date(pub_date)
            .content(Some(note.content.clone()))
            .build();
        items.push(item);
    }

    let channel = rss::ChannelBuilder::default()
        .title(&w.title)
        .link(w.absolute_url(&w.url_for_home_page()))
        .last_build_date(Some(Utc::now().to_rfc2822()))
        .items(items)
        .build();

    channel.to_string()
}
