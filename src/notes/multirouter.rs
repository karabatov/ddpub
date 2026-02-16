//! MultiRouter → axum Router.

use crate::config::Website;
use crate::notes::multistore::MultiStore;
use crate::notes::router::Router;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use std::sync::Arc;

pub struct MultiRouter {
    pub main: Router,
    pub sub_routers: Vec<Router>,
}

impl MultiRouter {
    pub fn new(w: &Website, m: &MultiStore) -> Result<Self, Box<dyn std::error::Error>> {
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
        // from the non-slash version, matching Go's http.ServeMux behavior.
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
