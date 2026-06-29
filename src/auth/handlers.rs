use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{jwt, password, AuthUser},
    error::AppError,
    state::AppState,
};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub pseudo: String,
    pub email: String,
    pub mot_de_passe: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub mot_de_passe: String,
}

#[derive(Serialize)]
pub struct UserDto {
    pub id: i32,
    pub pseudo: String,
    pub email: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub utilisateur: UserDto,
}

/// POST /auth/register — crée un compte et renvoie un token.
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if req.pseudo.trim().is_empty() || req.email.trim().is_empty() {
        return Err(AppError::BadRequest("pseudo et email requis".into()));
    }
    if req.mot_de_passe.len() < 8 {
        return Err(AppError::BadRequest(
            "le mot de passe doit faire au moins 8 caractères".into(),
        ));
    }

    let hash = password::hash_password(&req.mot_de_passe)?;

    let (id, role): (i32, String) = sqlx::query_as(
        "INSERT INTO utilisateur (pseudo, email, mot_de_passe)
         VALUES ($1, $2, $3)
         RETURNING id_utilisateur, role",
    )
    .bind(&req.pseudo)
    .bind(&req.email)
    .bind(&hash)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::BadRequest("pseudo ou email déjà utilisé".into())
        }
        _ => AppError::Database(e),
    })?;

    let token = jwt::create_token(&state.config.jwt_secret, id, &req.pseudo, &role)?;
    Ok(Json(AuthResponse {
        token,
        utilisateur: UserDto {
            id,
            pseudo: req.pseudo,
            email: req.email,
            role,
        },
    }))
}

/// POST /auth/login — authentifie par email + mot de passe.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let row: Option<(i32, String, String, String)> = sqlx::query_as(
        "SELECT id_utilisateur, pseudo, mot_de_passe, role
         FROM utilisateur WHERE email = $1",
    )
    .bind(&req.email)
    .fetch_optional(&state.pool)
    .await?;

    let (id, pseudo, hash, role) = row.ok_or(AppError::Unauthorized)?;

    if !password::verify_password(&req.mot_de_passe, &hash)? {
        return Err(AppError::Unauthorized);
    }

    let token = jwt::create_token(&state.config.jwt_secret, id, &pseudo, &role)?;
    Ok(Json(AuthResponse {
        token,
        utilisateur: UserDto {
            id,
            pseudo,
            email: req.email,
            role,
        },
    }))
}

/// GET /auth/me — route protégée : renvoie l'utilisateur courant.
/// L'extracteur `AuthUser` valide le token avant d'atteindre le handler.
pub async fn me(State(state): State<AppState>, user: AuthUser) -> Result<Json<UserDto>, AppError> {
    let (id, pseudo, email, role): (i32, String, String, String) = sqlx::query_as(
        "SELECT id_utilisateur, pseudo, email, role FROM utilisateur WHERE id_utilisateur = $1",
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(UserDto {
        id,
        pseudo,
        email,
        role,
    }))
}
