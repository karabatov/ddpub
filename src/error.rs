//! Typed error enum for DDPub.

use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Config(String),

    #[error("{0}")]
    Note(String),

    #[error("{0}")]
    Route(String),

    #[error("{0}")]
    L10n(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Regex(#[from] regex::Error),
}
