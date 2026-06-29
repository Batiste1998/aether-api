use axum::{extract::State, Json};
use serde::Serialize;

use crate::{error::AppError, state::AppState};

/// Une catégorie de thème suggérée.
#[derive(Serialize, sqlx::FromRow)]
pub struct CategorieDto {
    pub id_categorie: i32,
    pub libelle: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
}

/// GET /categories — liste les catégories suggérées (public).
pub async fn categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<CategorieDto>>, AppError> {
    let rows = sqlx::query_as::<_, CategorieDto>(
        "SELECT id_categorie, libelle, description, emoji FROM categorie ORDER BY id_categorie",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}
