use axum::{extract::State, Json};
use serde::Serialize;

use crate::{error::AppError, state::AppState};

/// Une classe jouable (donnée de référence).
#[derive(Serialize, sqlx::FromRow)]
pub struct ClasseDto {
    pub id_classe: i32,
    pub nom: String,
    pub description: Option<String>,
    pub pv_base: i32,
    pub force_base: i32,
    pub intelligence_base: i32,
    pub agilite_base: i32,
}

/// GET /classes — liste les classes disponibles (public).
pub async fn classes(State(state): State<AppState>) -> Result<Json<Vec<ClasseDto>>, AppError> {
    let rows = sqlx::query_as::<_, ClasseDto>(
        "SELECT id_classe, nom, description, pv_base, force_base, intelligence_base, agilite_base
         FROM classe ORDER BY id_classe",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}
