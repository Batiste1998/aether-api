use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{auth::AuthUser, error::AppError, state::AppState};

/// Représentation d'un personnage renvoyée au client (avec le nom de sa classe).
#[derive(Serialize, sqlx::FromRow)]
pub struct PersonnageDto {
    pub id_personnage: i32,
    pub nom: String,
    pub niveau: i32,
    pub xp: i32,
    pub pv_actuels: i32,
    pub pv_max: i32,
    pub or_pieces: i32,
    pub histoire: Option<String>,
    pub created_at: DateTime<Utc>,
    pub id_classe: i32,
    pub classe_nom: String,
}

#[derive(Deserialize)]
pub struct CreatePersonnage {
    pub nom: String,
    pub id_classe: i32,
    pub histoire: Option<String>,
}

/// SELECT commun : joint la classe pour exposer son nom.
const SELECT_BASE: &str = "
    SELECT p.id_personnage, p.nom, p.niveau, p.xp, p.pv_actuels, p.pv_max,
           p.or_pieces, p.histoire, p.created_at, p.id_classe, c.nom AS classe_nom
    FROM personnage p
    JOIN classe c ON c.id_classe = p.id_classe";

/// Récupère un personnage donné s'il appartient bien à l'utilisateur.
async fn fetch_owned(
    pool: &sqlx::PgPool,
    user_id: i32,
    id: i32,
) -> Result<Option<PersonnageDto>, AppError> {
    let sql = format!("{SELECT_BASE} WHERE p.id_personnage = $1 AND p.id_utilisateur = $2");
    let p = sqlx::query_as::<_, PersonnageDto>(&sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(p)
}

/// POST /personnages — crée un personnage ; PV et stats dérivent de la classe.
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreatePersonnage>,
) -> Result<(StatusCode, Json<PersonnageDto>), AppError> {
    if req.nom.trim().is_empty() {
        return Err(AppError::BadRequest("le nom du personnage est requis".into()));
    }

    // La classe fixe les PV de départ (et plus tard les stats / compétences).
    let pv_base: Option<(i32,)> = sqlx::query_as("SELECT pv_base FROM classe WHERE id_classe = $1")
        .bind(req.id_classe)
        .fetch_optional(&state.pool)
        .await?;
    let (pv,) = pv_base.ok_or(AppError::BadRequest("classe inconnue".into()))?;

    let (id,): (i32,) = sqlx::query_as(
        "INSERT INTO personnage (nom, pv_actuels, pv_max, histoire, id_utilisateur, id_classe)
         VALUES ($1, $2, $2, $3, $4, $5)
         RETURNING id_personnage",
    )
    .bind(&req.nom)
    .bind(pv)
    .bind(&req.histoire)
    .bind(user.id)
    .bind(req.id_classe)
    .fetch_one(&state.pool)
    .await?;

    let perso = fetch_owned(&state.pool, user.id, id)
        .await?
        .ok_or(AppError::Internal)?;
    Ok((StatusCode::CREATED, Json(perso)))
}

/// GET /personnages — liste les personnages de l'utilisateur connecté.
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<PersonnageDto>>, AppError> {
    let sql = format!("{SELECT_BASE} WHERE p.id_utilisateur = $1 ORDER BY p.created_at DESC");
    let rows = sqlx::query_as::<_, PersonnageDto>(&sql)
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

/// GET /personnages/{id} — fiche détaillée d'un de ses personnages.
pub async fn detail(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<PersonnageDto>, AppError> {
    let perso = fetch_owned(&state.pool, user.id, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(perso))
}

/// DELETE /personnages/{id} — supprime un personnage (uniquement le sien).
pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM personnage WHERE id_personnage = $1 AND id_utilisateur = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Routes personnage, montées sous `/personnages`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create).get(list))
        .route("/{id}", get(detail).delete(delete))
}
