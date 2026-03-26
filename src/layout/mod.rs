//! Page/Head/Header/Footer/Content types, template rendering via minijinja.

use crate::error::{Error, Result};
use minijinja::{Environment, context};
use serde::Serialize;
use std::sync::LazyLock;

#[derive(Serialize)]
pub struct MetaTags {
    pub title: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub image: String,
    pub url: String,
    pub locale: String,
    pub site_name: String,
    pub twitter: String,
}

#[derive(Serialize)]
pub struct Head {
    pub title: String,
    pub website_title: String,
    pub meta_tags: MetaTags,
    pub rss_url: String,
    pub suffix: String,
}

#[derive(Serialize)]
pub struct Header {
    pub homepage_url: String,
    pub title: String,
    pub menu: Vec<ListItem>,
}

#[derive(Serialize)]
pub struct Footer {
    pub prefix: String,
    pub powered_by: String,
    pub rss_url: String,
}

#[derive(Clone, Serialize)]
pub struct ListItem {
    pub title: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct Page {
    pub language: String,
    pub head: Head,
    pub header: Header,
    pub content: String,
    pub footer: Footer,
}

#[derive(Serialize)]
pub struct BuiltinFeed {
    pub title: String,
    pub content: String,
    pub notes: Vec<NoteListItem>,
}

#[derive(Serialize)]
pub struct TagListItem {
    pub list_item: ListItem,
    pub count: usize,
}

#[derive(Serialize)]
pub struct BuiltinTags {
    pub title: String,
    pub tags: Vec<TagListItem>,
}

#[derive(Serialize)]
pub struct BuiltinSearch {
    pub title: String,
    pub pagefind_css_url: String,
    pub pagefind_js_url: String,
}

#[derive(Serialize)]
pub struct ContentPage {
    pub title: String,
    pub content: String,
    pub has_leading_h1: bool,
}

#[derive(Clone, Serialize)]
pub struct NoteListItem {
    pub list_item: ListItem,
    pub date: String,
}

#[derive(Serialize)]
pub struct ContentTagPage {
    pub title: String,
    pub content: String,
    pub notes: Vec<NoteListItem>,
    pub has_leading_h1: bool,
}

#[derive(Serialize)]
pub struct ContentNote {
    pub title: String,
    pub date: String,
    pub tags: Vec<ListItem>,
    pub content: String,
    pub suffix: String,
    pub has_leading_h1: bool,
}

// Template loading uses unwrap() intentionally: templates are compile-time
// embedded via include_str!(), so a parse failure here is a build-time bug
// that must be fixed in the source templates before shipping.
static ENV: LazyLock<Environment<'static>> = LazyLock::new(|| {
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);

    env.add_template("base", include_str!("templates/base.html")).unwrap();
    env.add_template("head", include_str!("templates/head.html")).unwrap();
    env.add_template("metatags", include_str!("templates/metatags.html")).unwrap();
    env.add_template("body", include_str!("templates/body.html")).unwrap();
    env.add_template("header", include_str!("templates/header.html")).unwrap();
    env.add_template("footer", include_str!("templates/footer.html")).unwrap();
    env.add_template("menu", include_str!("templates/menu.html")).unwrap();
    env.add_template("content_page", include_str!("templates/content_page.html")).unwrap();
    env.add_template("content_note", include_str!("templates/content_note.html")).unwrap();
    env.add_template("builtin_feed", include_str!("templates/builtin_feed.html")).unwrap();
    env.add_template("builtin_tags", include_str!("templates/builtin_tags.html")).unwrap();
    env.add_template("builtin_search", include_str!("templates/builtin_search.html")).unwrap();
    env.add_template("content_tag", include_str!("templates/content_tag.html")).unwrap();
    env
});

fn render_error(e: minijinja::Error) -> Error {
    Error::TemplateRender { cause: e.to_string() }
}

pub fn fill_page(p: &Page) -> Result<Vec<u8>> {
    let tmpl = ENV.get_template("base").map_err(render_error)?;
    Ok(tmpl.render(context!(page => p)).map_err(render_error)?.into_bytes())
}

pub fn fill_content_page(p: &ContentPage) -> Result<String> {
    let tmpl = ENV.get_template("content_page").map_err(render_error)?;
    tmpl.render(context!(p => p)).map_err(render_error)
}

pub fn fill_content_note(n: &ContentNote) -> Result<String> {
    let tmpl = ENV.get_template("content_note").map_err(render_error)?;
    tmpl.render(context!(n => n)).map_err(render_error)
}

pub fn fill_builtin_feed(p: &BuiltinFeed) -> Result<String> {
    let tmpl = ENV.get_template("builtin_feed").map_err(render_error)?;
    tmpl.render(context!(p => p)).map_err(render_error)
}

pub fn fill_builtin_tags(p: &BuiltinTags) -> Result<String> {
    let tmpl = ENV.get_template("builtin_tags").map_err(render_error)?;
    tmpl.render(context!(p => p)).map_err(render_error)
}

pub fn fill_builtin_search(p: &BuiltinSearch) -> Result<String> {
    let tmpl = ENV.get_template("builtin_search").map_err(render_error)?;
    tmpl.render(context!(p => p)).map_err(render_error)
}

pub fn fill_content_tag(p: &ContentTagPage) -> Result<String> {
    let tmpl = ENV.get_template("content_tag").map_err(render_error)?;
    tmpl.render(context!(p => p)).map_err(render_error)
}
