//! Redirect config.

use crate::config::data;
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Redirect {
    pub url: String,
    pub destination: String,
}

pub fn parse_redirect(r: &data::Redirect) -> Result<Redirect> {
    if r.url.is_empty() {
        return Err(Error::RedirectEmptyUrl);
    }
    if r.destination.is_empty() {
        return Err(Error::RedirectEmptyDestination { url: r.url.clone() });
    }
    if !r.url.starts_with('/') {
        return Err(Error::RedirectInvalidUrl { url: r.url.clone() });
    }
    let is_external = r.destination.starts_with("http://") || r.destination.starts_with("https://");
    let is_internal = r.destination.starts_with('/');
    if !is_external && !is_internal {
        return Err(Error::RedirectInvalidDestination {
            url: r.url.clone(),
            destination: r.destination.clone(),
        });
    }
    Ok(Redirect {
        url: r.url.clone(),
        destination: r.destination.clone(),
    })
}
