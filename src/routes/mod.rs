use axum::{routing::get, Router};

use crate::state::AppState;

pub mod health;

/// Construit le routeur principal de l'API.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/categories", get(crate::reference::categories))
        .nest("/auth", crate::auth::router())
        .merge(crate::quiz::router())
        .with_state(state)
}
