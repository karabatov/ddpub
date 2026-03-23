//! Router, handler generation, RSS.

use crate::config::{self, WebsiteLang};
use crate::dd::{self, Builtin};
use crate::error::{Error, Result};
use crate::l10n::Key;
use crate::layout;
use crate::notes::html::*;
use crate::notes::{LinkInfo, PublishTarget, Store, html_as_text};
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;

/// A route maps a URL pattern to pre-rendered content with a content type.
#[derive(Clone)]
pub struct Route {
    pub content: Vec<u8>,
    pub content_type: String,
}

pub struct Router {
    pub routes: HashMap<String, Route>,
    pub redirects: Vec<(String, String)>,
    pub not_found_page: Vec<u8>,
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
    layout::fill_page(&page)
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
        return Err(Error::RouteConflict { pattern: pattern.to_string() });
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
    pub async fn new(w: &WebsiteLang, s: &Store) -> Result<Self> {
        let mut routes: HashMap<String, Route> = HashMap::new();
        let mut redirect_list: Vec<(String, String)> = Vec::new();

        // Homepage redirect: the site is just this one redirect + 404 page.
        if let config::homepage::Homepage::Redirect(ref dest) = w.homepage {
            redirect_list.push((w.url_for_home_page(), dest.clone()));

            // Generate 404 page.
            let not_found_content = format!("<p>{}</p>", w.str(Key::NotFoundMessage));
            let not_found_page = make_page(w, s, "/404.html", w.str(Key::NotFoundTitle), not_found_content)?;

            return Ok(Router { routes, redirects: redirect_list, not_found_page });
        }

        // Add homepage.
        match &w.homepage {
            config::homepage::Homepage::NoteId(id) => {
                let note = s.note_content.get(id).ok_or_else(|| {
                    Error::HomepageContentNotFound { id: id.clone() }
                })?;
                let content = html_for_page(note)?;
                add_page(&mut routes, w, s, &w.url_for_home_page(), &html_as_text(&note.meta.title), content)?;
            }
            config::homepage::Homepage::Feed => {
                let content = html_for_builtin_feed(w, s)?;
                add_page(&mut routes, w, s, &w.url_for_home_page(), &w.feed.title, content)?;
            }
            config::homepage::Homepage::Redirect(_) => unreachable!(),
        }

        // Builtin - feed.
        {
            let pattern = w.url_for_builtin(Builtin::Feed);
            if !routes.contains_key(&pattern) {
                let content = html_for_builtin_feed(w, s)?;
                add_page(&mut routes, w, s, &pattern, &w.feed.title, content)?;
            }
        }

        // Builtin - tags.
        {
            let pattern = w.url_for_builtin(Builtin::Tags);
            let content = html_for_builtin_tags(w, s)?;
            add_page(&mut routes, w, s, &pattern, w.str(Key::TagsTitle), content)?;
        }

        // Builtin - search.
        if w.search {
            let pattern = w.url_for_builtin(Builtin::Search);
            let content = html_for_builtin_search(w)?;
            add_page(&mut routes, w, s, &pattern, w.str(Key::SearchTitle), content)?;
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
                    let content = html_for_note(note, w)?;
                    add_page(&mut routes, w, s, &pattern, &html_as_text(&note.meta.title), content)?;
                }
                PublishTarget::Page => {
                    let pattern = w.url_for_page_note(&note.meta.slug);
                    let content = html_for_page(note)?;
                    add_page(&mut routes, w, s, &pattern, &html_as_text(&note.meta.title), content)?;
                }
            }
        }

        // Add published tags.
        for t in w.tags.values() {
            let pattern = w.url_for_tag(t);
            let content = html_for_tag(t, w, s)?;
            add_page(&mut routes, w, s, &pattern, &t.title, content)?;
        }

        // Add shared files (only for main config).
        if !w.is_child {
            for f in &w.shared_files {
                let pattern = w.url_for_shared_file(&f.filename);
                if routes.contains_key(&pattern) {
                    return Err(Error::RouteConflict { pattern });
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
            let content = std::fs::read(&f.path).map_err(|e| {
                Error::FileReadFailed {
                    path: f.path.clone(),
                    cause: e.to_string(),
                }
            })?;
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

        // Build pagefind search index and add generated files as routes.
        if w.search {
            let search_url = w.url_for_builtin(Builtin::Search);
            let pf_config = pagefind::options::PagefindServiceConfig::builder()
                .root_selector("main".to_string())
                .force_language(w.language.code.short_code().to_string())
                .keep_index_url(false)
                .build();

            let mut index = pagefind::api::PagefindIndex::new(Some(pf_config))
                .map_err(|e| Error::SearchIndexError { cause: format!("{e}") })?;

            for (url, route) in &routes {
                if !route.content_type.starts_with("text/html") {
                    continue;
                }
                // Skip the search page itself — it has no meaningful content to index.
                if *url == search_url {
                    continue;
                }
                let html = String::from_utf8_lossy(&route.content).to_string();
                let _ = index.add_html_file(None, Some(url.clone()), html).await
                    .map_err(|e| Error::SearchIndexError { cause: format!("{e}") })?;
            }

            let pagefind_files = index.get_files().await
                .map_err(|e| Error::SearchIndexError { cause: format!("{e}") })?;

            for file in pagefind_files {
                let filename = file.filename.to_string_lossy();
                let url = w.url_for_pagefind_file(&filename);
                if routes.contains_key(&url) {
                    return Err(Error::ReservedRouteConflict {
                        pattern: url,
                        reserved: "pagefind/".to_string(),
                    });
                }
                let content_type = dd::guess_content_type(Path::new(&*filename)).to_string();
                routes.insert(url, Route {
                    content: file.contents,
                    content_type,
                });
            }
        }

        // Add redirects — source must not conflict with any existing route.
        for r in &w.redirects {
            if routes.contains_key(&r.url) {
                return Err(Error::RedirectRouteConflict { url: r.url.clone() });
            }
            redirect_list.push((r.url.clone(), r.destination.clone()));
        }

        // Generate 404 page.
        let not_found_content = format!("<p>{}</p>", w.str(Key::NotFoundMessage));
        let not_found_page = make_page(w, s, "/404.html", w.str(Key::NotFoundTitle), not_found_content)?;

        // Collect all broken links: unresolved references + links to unpublished notes.
        let mut broken: Vec<String> = Vec::new();
        for link in &s.broken_links {
            broken.push(format_broken_link(link));
        }
        for link in &s.resolved_links {
            if !routes.contains_key(link.url.as_str()) {
                broken.push(format_broken_link(link));
            }
        }
        // Validate internal redirect destinations exist.
        for (url, dest) in &redirect_list {
            if dest.starts_with('/') && !routes.contains_key(dest.as_str()) {
                broken.push(format!("redirect '{url}' → '{dest}' (target route not found)"));
            }
        }
        if !broken.is_empty() {
            broken.sort();
            return Err(Error::BrokenLinks { links: broken });
        }

        Ok(Router { routes, redirects: redirect_list, not_found_page })
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
                        Error::MenuNoteContentNotFound { id: id.clone() }
                    })?;
                    w.url_for_page_note(&note.meta.slug)
                }
                config::menu::Menu::Tag { tag, .. } => {
                    let tag_config = w.tags.get(tag).ok_or_else(|| {
                        Error::MenuTagNotFound { tag: tag.clone() }
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

fn format_broken_link(link: &LinkInfo) -> String {
    let notes = link.notes.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", ");
    if link.text.is_empty() {
        format!("{} in {}", link.url, notes)
    } else {
        format!("'{}' → {} in {}", link.text, link.url, notes)
    }
}
