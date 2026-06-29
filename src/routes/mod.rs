use axum::{routing::get, Router};

use crate::state::AppState;

pub mod health;

/// Construit le routeur principal de l'API.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .nest("/auth", crate::auth::router())
        .nest("/personnages", crate::personnage::router())
        .with_state(state)
}
