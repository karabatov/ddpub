//! MultiStore.

use crate::config::{Website, WebsiteLang};
use crate::dd::Language;
use crate::error::Result;
use crate::notes::Store;
use std::collections::HashMap;

pub struct MultiStore {
    pub main: Store,
    pub sub_stores: HashMap<Language, Store>,
}

impl MultiStore {
    pub fn new(w: &Website, notes_dir: &str) -> Result<Self> {
        let all_configs: Vec<&WebsiteLang> = std::iter::once(&w.main)
            .chain(w.sub_configs.iter())
            .collect();

        let main = Store::new(&w.main, notes_dir, &all_configs)?;

        let mut sub_stores = HashMap::new();
        for cfg in &w.sub_configs {
            let s = Store::new(cfg, notes_dir, &all_configs)?;
            sub_stores.insert(cfg.language.code, s);
        }

        Ok(MultiStore { main, sub_stores })
    }
}
