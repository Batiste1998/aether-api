use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
    routing::{get, post},
    Router,
};

use crate::{error::AppError, state::AppState};

pub mod handlers;
pub mod jwt;
pub mod password;

/// Utilisateur authentifié, extrait du header `Authorization: Bearer <token>`.
/// Implémente `FromRequestParts` : il suffit de l'ajouter en argument d'un
/// handler pour rendre la route protégée.
pub struct AuthUser {
    pub id: i32,
    // Conservés pour un futur contrôle d'accès par rôle (back-office admin).
    #[allow(dead_code)]
    pub pseudo: String,
    #[allow(dead_code)]
    pub role: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let claims = jwt::decode_token(&state.config.jwt_secret, token)?;
        Ok(AuthUser {
            id: claims.sub,
            pseudo: claims.pseudo,
            role: claims.role,
        })
    }
}

/// Routes d'authentification, montées sous `/auth`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/me", get(handlers::me))
}
