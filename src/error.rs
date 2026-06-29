use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Erreur applicative unifiée, convertie automatiquement en réponse HTTP JSON.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("erreur base de données: {0}")]
    Database(#[from] sqlx::Error),

    #[error("ressource introuvable")]
    NotFound,

    #[error("{0}")]
    BadRequest(String),

    #[error("non autorisé")]
    Unauthorized,

    #[error("erreur interne du serveur")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Database(_) | AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // On loggue les erreurs serveur sans exposer les détails internes au client.
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("{self}");
        }

        let body = Json(json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}
