//! ddpub is a static site generator that serves one set of notes as many websites.

pub mod config;
pub mod dd;
pub mod error;
mod l10n;
mod layout;
pub mod notes;

pub use config::Website;
pub use dd::Language;
pub use error::{Error, ErrorStage, Result};
pub use notes::multistore::MultiStore;
pub use notes::multirouter::MultiRouter;
