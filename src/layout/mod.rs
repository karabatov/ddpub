//! Page/Head/Header/Footer/Content types, template rendering via minijinja.

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
pub struct ContentPage {
    pub title: String,
    pub content: String,
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
}

#[derive(Serialize)]
pub struct ContentNote {
    pub title: String,
    pub date: String,
    pub tags: Vec<ListItem>,
    pub content: String,
    pub suffix: String,
}

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
    env.add_template("content_tag", include_str!("templates/content_tag.html")).unwrap();
    env
});

pub fn fill_page(p: &Page) -> Vec<u8> {
    let tmpl = ENV.get_template("base").unwrap();
    tmpl.render(context!(page => p)).unwrap().into_bytes()
}

pub fn fill_content_page(p: &ContentPage) -> String {
    let tmpl = ENV.get_template("content_page").unwrap();
    tmpl.render(context!(p => p)).unwrap()
}

pub fn fill_content_note(n: &ContentNote) -> String {
    let tmpl = ENV.get_template("content_note").unwrap();
    tmpl.render(context!(n => n)).unwrap()
}

pub fn fill_builtin_feed(p: &BuiltinFeed) -> String {
    let tmpl = ENV.get_template("builtin_feed").unwrap();
    tmpl.render(context!(p => p)).unwrap()
}

pub fn fill_builtin_tags(p: &BuiltinTags) -> String {
    let tmpl = ENV.get_template("builtin_tags").unwrap();
    tmpl.render(context!(p => p)).unwrap()
}

pub fn fill_content_tag(p: &ContentTagPage) -> String {
    let tmpl = ENV.get_template("content_tag").unwrap();
    tmpl.render(context!(p => p)).unwrap()
}
