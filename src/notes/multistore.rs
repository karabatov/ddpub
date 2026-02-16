//! MultiStore.

use crate::config::Website;
use crate::dd::Language;
use crate::notes::Store;
use std::collections::HashMap;

pub struct MultiStore {
    pub main: Store,
    pub sub_stores: HashMap<Language, Store>,
}

impl MultiStore {
    pub fn new(w: &Website, notes_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let main = Store::new(&w.main, notes_dir)?;

        let mut sub_stores = HashMap::new();
        for cfg in &w.sub_configs {
            let s = Store::new(cfg, notes_dir)?;
            sub_stores.insert(cfg.language.code, s);
        }

        Ok(MultiStore { main, sub_stores })
    }
}
