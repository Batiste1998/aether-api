use axum::{routing::get, Router};

use crate::state::AppState;

pub mod health;

/// Construit le routeur principal de l'API.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/classes", get(crate::reference::classes))
        .nest("/auth", crate::auth::router())
        .nest("/personnages", crate::personnage::router())
        .nest("/parties", crate::partie::router())
        .with_state(state)
}
