//! Typed, structured error enum for DDPub with localized messages.

use crate::dd::Language;
use crate::l10n;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

/// Which pipeline stage produced an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorStage {
    Config,
    Notes,
    Routing,
    Localization,
}

/// Structured error type. Each variant carries enough context for a
/// detailed, localized user-facing message.
#[derive(Debug)]
pub enum Error {
    // ── Config stage ───────────────────────────────────────────────
    ConfigFileOpen {
        path: String,
        cause: String,
    },
    ConfigFileParse {
        path: String,
        cause: String,
    },
    HomepageConflict,
    HomepageInvalidNoteId {
        id: String,
    },
    FeedConflict,
    FeedInvalidNoteId {
        id: String,
    },
    EmptyTag,
    TagConflict {
        tag: String,
    },
    TagInvalidNoteId {
        tag: String,
        id: String,
    },
    DuplicateTag {
        tag: String,
    },
    MenuMultipleTypes,
    MenuUnknownBuiltin {
        name: String,
    },
    MenuInvalidNoteId {
        id: String,
    },
    MenuTagNotPublished {
        tag: String,
    },
    UnsupportedLanguage {
        code: String,
    },
    DomainNotSet {
        path: String,
    },
    LanguageMismatch {
        language: String,
    },
    RedirectEmptyUrl,
    RedirectEmptyDestination {
        url: String,
    },
    RedirectInvalidUrl {
        url: String,
    },
    RedirectInvalidDestination {
        url: String,
        destination: String,
    },

    // ── Notes stage ────────────────────────────────────────────────
    DuplicateNoteId {
        id: String,
        file1: String,
        file2: String,
    },
    NotesDirectoryUnreadable {
        dir: String,
        cause: String,
    },
    NoteNotPublished {
        id: String,
    },
    MetadataNotFound {
        id: String,
        source: String,
    },
    NoteIo(std::io::Error),

    // ── Routing stage ──────────────────────────────────────────────
    ExportDirConflict {
        dir: String,
    },
    ExportDirNotEmpty {
        dir: String,
    },
    ExportIo(std::io::Error),
    HomepageContentNotFound {
        id: String,
    },
    RouteConflict {
        pattern: String,
    },
    MenuNoteContentNotFound {
        id: String,
    },
    MenuTagNotFound {
        tag: String,
    },
    BrokenLinks {
        links: Vec<String>,
    },
    FileReadFailed {
        path: String,
        cause: String,
    },
    NoteContentNotFound {
        id: String,
    },
    TemplateRender {
        cause: String,
    },
    RouteCollision {
        pattern: String,
        ids: Vec<String>,
    },
    FileOutsideNotesDir {
        path: String,
    },
    RedirectRouteConflict {
        url: String,
    },
    ReservedRouteConflict {
        pattern: String,
        reserved: String,
    },
    SearchIndexError {
        cause: String,
    },

    // ── Localization stage ─────────────────────────────────────────
    LanguageStringsLoadFailed {
        cause: String,
    },
}

impl Error {
    /// Which pipeline stage produced this error.
    pub fn stage(&self) -> ErrorStage {
        match self {
            // Config
            Error::ConfigFileOpen { .. }
            | Error::ConfigFileParse { .. }
            | Error::HomepageConflict
            | Error::HomepageInvalidNoteId { .. }
            | Error::FeedConflict
            | Error::FeedInvalidNoteId { .. }
            | Error::EmptyTag
            | Error::TagConflict { .. }
            | Error::TagInvalidNoteId { .. }
            | Error::DuplicateTag { .. }
            | Error::MenuMultipleTypes
            | Error::MenuUnknownBuiltin { .. }
            | Error::MenuInvalidNoteId { .. }
            | Error::MenuTagNotPublished { .. }
            | Error::UnsupportedLanguage { .. }
            | Error::DomainNotSet { .. }
            | Error::LanguageMismatch { .. }
            | Error::RedirectEmptyUrl
            | Error::RedirectEmptyDestination { .. }
            | Error::RedirectInvalidUrl { .. }
            | Error::RedirectInvalidDestination { .. } => ErrorStage::Config,

            // Notes
            Error::DuplicateNoteId { .. }
            | Error::NotesDirectoryUnreadable { .. }
            | Error::NoteNotPublished { .. }
            | Error::MetadataNotFound { .. }
            | Error::NoteIo(_) => ErrorStage::Notes,

            // Routing
            Error::ExportDirConflict { .. }
            | Error::ExportDirNotEmpty { .. }
            | Error::ExportIo(_)
            | Error::HomepageContentNotFound { .. }
            | Error::RouteConflict { .. }
            | Error::MenuNoteContentNotFound { .. }
            | Error::MenuTagNotFound { .. }
            | Error::BrokenLinks { .. }
            | Error::FileReadFailed { .. }
            | Error::NoteContentNotFound { .. }
            | Error::TemplateRender { .. }
            | Error::RouteCollision { .. }
            | Error::FileOutsideNotesDir { .. }
            | Error::RedirectRouteConflict { .. }
            | Error::ReservedRouteConflict { .. }
            | Error::SearchIndexError { .. } => ErrorStage::Routing,

            // Localization
            Error::LanguageStringsLoadFailed { .. } => ErrorStage::Localization,
        }
    }

    /// User-facing localized error description.
    pub fn localized(&self, lang: Language) -> String {
        let strings = l10n::error_strings(lang);
        self.substitute(&strings)
    }

    fn substitute(&self, s: &l10n::ErrorStrings) -> String {
        match self {
            Error::ConfigFileOpen { path, cause } =>
                s.config_file_open.replace("{path}", path).replace("{cause}", cause),
            Error::ConfigFileParse { path, cause } =>
                s.config_file_parse.replace("{path}", path).replace("{cause}", cause),
            Error::HomepageConflict =>
                s.homepage_conflict.clone(),
            Error::HomepageInvalidNoteId { id } =>
                s.homepage_invalid_note_id.replace("{id}", id),
            Error::FeedConflict =>
                s.feed_conflict.clone(),
            Error::FeedInvalidNoteId { id } =>
                s.feed_invalid_note_id.replace("{id}", id),
            Error::EmptyTag =>
                s.empty_tag.clone(),
            Error::TagConflict { tag } =>
                s.tag_conflict.replace("{tag}", tag),
            Error::TagInvalidNoteId { tag, id } =>
                s.tag_invalid_note_id.replace("{tag}", tag).replace("{id}", id),
            Error::DuplicateTag { tag } =>
                s.duplicate_tag.replace("{tag}", tag),
            Error::MenuMultipleTypes =>
                s.menu_multiple_types.clone(),
            Error::MenuUnknownBuiltin { name } =>
                s.menu_unknown_builtin.replace("{name}", name),
            Error::MenuInvalidNoteId { id } =>
                s.menu_invalid_note_id.replace("{id}", id),
            Error::MenuTagNotPublished { tag } =>
                s.menu_tag_not_published.replace("{tag}", tag),
            Error::UnsupportedLanguage { code } =>
                s.unsupported_language.replace("{code}", code),
            Error::DomainNotSet { path } =>
                s.domain_not_set.replace("{path}", path),
            Error::LanguageMismatch { language } =>
                s.language_mismatch.replace("{language}", language),
            Error::RedirectEmptyUrl =>
                s.redirect_empty_url.clone(),
            Error::RedirectEmptyDestination { url } =>
                s.redirect_empty_destination.replace("{url}", url),
            Error::RedirectInvalidUrl { url } =>
                s.redirect_invalid_url.replace("{url}", url),
            Error::RedirectInvalidDestination { url, destination } =>
                s.redirect_invalid_destination.replace("{url}", url).replace("{destination}", destination),
            Error::DuplicateNoteId { id, file1, file2 } =>
                s.duplicate_note_id.replace("{id}", id).replace("{file1}", file1).replace("{file2}", file2),
            Error::NotesDirectoryUnreadable { dir, cause } =>
                s.notes_directory_unreadable.replace("{dir}", dir).replace("{cause}", cause),
            Error::NoteNotPublished { id } =>
                s.note_not_published.replace("{id}", id),
            Error::MetadataNotFound { id, source } =>
                s.metadata_not_found.replace("{id}", id).replace("{source}", source),
            Error::NoteIo(e) =>
                s.note_io.replace("{cause}", &e.to_string()),
            Error::ExportDirConflict { dir } =>
                s.export_dir_conflict.replace("{dir}", dir),
            Error::ExportDirNotEmpty { dir } =>
                s.export_dir_not_empty.replace("{dir}", dir),
            Error::ExportIo(e) =>
                s.export_io.replace("{cause}", &e.to_string()),
            Error::HomepageContentNotFound { id } =>
                s.homepage_content_not_found.replace("{id}", id),
            Error::RouteConflict { pattern } =>
                s.route_conflict.replace("{pattern}", pattern),
            Error::MenuNoteContentNotFound { id } =>
                s.menu_note_content_not_found.replace("{id}", id),
            Error::MenuTagNotFound { tag } =>
                s.menu_tag_not_found.replace("{tag}", tag),
            Error::BrokenLinks { links } =>
                s.broken_links.replace("{links}", &links.join("\n  - ")),
            Error::FileReadFailed { path, cause } =>
                s.file_read_failed.replace("{path}", path).replace("{cause}", cause),
            Error::NoteContentNotFound { id } =>
                s.note_content_not_found.replace("{id}", id),
            Error::TemplateRender { cause } =>
                s.template_render.replace("{cause}", cause),
            Error::RouteCollision { pattern, ids } =>
                s.route_collision.replace("{pattern}", pattern).replace("{ids}", &ids.join(", ")),
            Error::FileOutsideNotesDir { path } =>
                s.file_outside_notes_dir.replace("{path}", path),
            Error::RedirectRouteConflict { url } =>
                s.redirect_route_conflict.replace("{url}", url),
            Error::ReservedRouteConflict { pattern, reserved } =>
                s.reserved_route_conflict.replace("{pattern}", pattern).replace("{reserved}", reserved),
            Error::SearchIndexError { cause } =>
                s.search_index_error.replace("{cause}", cause),
            Error::LanguageStringsLoadFailed { cause } =>
                s.language_strings_load_failed.replace("{cause}", cause),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Default Display uses en-US.
        let strings = l10n::error_strings(Language::EnUS);
        write!(f, "{}", self.substitute(&strings))
    }
}

impl std::error::Error for Error {}
