//! MultiRouter → axum Router or static export.

use crate::config::Website;
use crate::error::Result;
use crate::notes::multistore::MultiStore;
use crate::notes::router::Router;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use std::path::Path;
use std::sync::Arc;

pub struct MultiRouter {
    pub main: Router,
    pub sub_routers: Vec<Router>,
}

impl MultiRouter {
    pub fn new(w: &Website, m: &MultiStore) -> Result<Self> {
        let main = Router::new(&w.main, &m.main)?;

        let mut sub_routers = Vec::new();
        for cfg in &w.sub_configs {
            if let Some(store) = m.sub_stores.get(&cfg.language.code) {
                let router = Router::new(cfg, store)?;
                sub_routers.push(router);
            }
        }

        Ok(MultiRouter { main, sub_routers })
    }

    pub fn export(&self, dir: &Path, force: bool, config_dir: &Path, notes_dir: &Path) -> Result<()> {
        use crate::error::Error;

        // Safety: never export into config or notes directories.
        if dir.exists() {
            let export_path = std::fs::canonicalize(dir).map_err(Error::ExportIo)?;
            if export_path == config_dir || export_path == notes_dir {
                return Err(Error::ExportDirConflict {
                    dir: dir.display().to_string(),
                });
            }
        }

        if dir.exists() {
            let has_entries = dir
                .read_dir()
                .map_err(Error::ExportIo)?
                .next()
                .is_some();
            if has_entries {
                if !force {
                    return Err(Error::ExportDirNotEmpty {
                        dir: dir.display().to_string(),
                    });
                }
                // Clear contents but keep the directory itself.
                for entry in dir.read_dir().map_err(Error::ExportIo)? {
                    let entry = entry.map_err(Error::ExportIo)?;
                    let path = entry.path();
                    if path.is_dir() {
                        std::fs::remove_dir_all(&path).map_err(Error::ExportIo)?;
                    } else {
                        std::fs::remove_file(&path).map_err(Error::ExportIo)?;
                    }
                }
            }
        } else {
            std::fs::create_dir_all(dir).map_err(Error::ExportIo)?;
        }

        let all_routes = std::iter::once(&self.main)
            .chain(self.sub_routers.iter());

        for router in all_routes {
            for (pattern, route) in &router.routes {
                let file_path = if pattern.ends_with('/') {
                    dir.join(pattern.trim_start_matches('/')).join("index.html")
                } else {
                    dir.join(pattern.trim_start_matches('/'))
                };

                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent).map_err(Error::ExportIo)?;
                }

                std::fs::write(&file_path, &route.content).map_err(Error::ExportIo)?;
            }
        }

        Ok(())
    }

    pub fn into_axum_router(self) -> axum::Router {
        let mut router = axum::Router::new();

        // Register all routes from main router
        router = register_routes(router, self.main);

        // Register all routes from sub routers
        for sub in self.sub_routers {
            router = register_routes(router, sub);
        }

        // Add fallback for 404
        router = router.fallback(|| async { StatusCode::NOT_FOUND });

        router
    }
}

fn register_routes(mut axum_router: axum::Router, router: Router) -> axum::Router {
    for (pattern, route) in router.routes {
        let route = Arc::new(route);
        axum_router = axum_router.route(
            &pattern,
            get(move || {
                let route = Arc::clone(&route);
                async move {
                    (
                        [(header::CONTENT_TYPE, route.content_type.clone())],
                        route.content.clone(),
                    )
                        .into_response()
                }
            }),
        );

        // For patterns ending with "/" (except "/" itself), add a redirect
        // from the non-slash version.
        if pattern.ends_with('/') && pattern.len() > 1 {
            let without_slash = pattern[..pattern.len() - 1].to_string();
            let with_slash = pattern.clone();
            axum_router = axum_router.route(
                &without_slash,
                get(move || async move { Redirect::permanent(&with_slash) }),
            );
        }
    }
    axum_router
}
