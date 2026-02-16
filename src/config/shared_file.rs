//! SharedFile with overload.

use std::fs;
use std::path::Path;

/// SharedFile is a file common to the whole website, embedded by default
/// but can be overloaded if a certain file is present in the config dir.
#[derive(Debug, Clone)]
pub struct SharedFile {
    pub filename: String,
    pub content: Vec<u8>,
    pub content_type: String,
}

impl SharedFile {
    /// overload tries to read the file from config_dir and replaces content and content_type.
    pub fn overload(&mut self, config_dir: &str, keep_content_type: bool) {
        let path = Path::new(config_dir).join(&self.filename);
        if let Ok(f) = fs::read(&path) {
            self.content = f;
            if !keep_content_type {
                self.content_type = guess_content_type(&path);
            }
        }
    }
}

fn guess_content_type(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "html" | "htm" => "text/html",
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
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => "application/octet-stream",
    }
    .to_string()
}
