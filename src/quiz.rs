use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{ai, auth::AuthUser, error::AppError, rules, state::AppState};

// ============ Génération d'un quiz ============

#[derive(Deserialize)]
pub struct GenerateRequest {
    pub theme: Option<String>,
    pub id_categorie: Option<i32>,
    pub difficulte: String,
    pub nb_questions: Option<i32>,
}

#[derive(Serialize)]
pub struct ChoixPublic {
    pub id_choix: i32,
    pub position: i32,
    pub texte: String,
}

#[derive(Serialize)]
pub struct QuestionPublic {
    pub id_question: i32,
    pub position: i32,
    pub intitule: String,
    pub choix: Vec<ChoixPublic>,
}

#[derive(Serialize)]
pub struct QuizSessionDto {
    pub id_session: i32,
    pub id_quiz: i32,
    pub theme: String,
    pub difficulte: String,
    pub temps_par_question_ms: i32,
    pub questions: Vec<QuestionPublic>,
}

/// POST /quiz — génère un quiz via l'IA et ouvre une session de jeu.
pub async fn generer(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<QuizSessionDto>, AppError> {
    if !rules::difficulte_valide(&req.difficulte) {
        return Err(AppError::BadRequest("difficulté invalide".into()));
    }
    let nb = rules::borne_nb_questions(req.nb_questions.unwrap_or(5));

    // Le thème vient soit d'une catégorie, soit d'une saisie libre.
    let (theme, id_categorie) = match req.id_categorie {
        Some(cat) => {
            let libelle: Option<(String,)> =
                sqlx::query_as("SELECT libelle FROM categorie WHERE id_categorie = $1")
                    .bind(cat)
                    .fetch_optional(&state.pool)
                    .await?;
            let (libelle,) = libelle.ok_or(AppError::BadRequest("catégorie inconnue".into()))?;
            (libelle, Some(cat))
        }
        None => {
            let t = req
                .theme
                .as_deref()
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .ok_or(AppError::BadRequest(
                    "un thème ou une catégorie est requis".into(),
                ))?;
            (t.to_string(), None)
        }
    };

    let genere = ai::generer_quiz(
        &state.openai,
        &state.config.openai_model,
        &theme,
        &req.difficulte,
        nb,
    )
    .await?;

    let mut tx = state.pool.begin().await?;

    let (id_quiz,): (i32,) = sqlx::query_as(
        "INSERT INTO quiz (theme, difficulte, id_categorie, id_utilisateur)
         VALUES ($1, $2, $3, $4) RETURNING id_quiz",
    )
    .bind(&theme)
    .bind(&req.difficulte)
    .bind(id_categorie)
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;

    let mut questions = Vec::new();
    for (qpos, q) in genere.questions.iter().enumerate() {
        let (id_question,): (i32,) = sqlx::query_as(
            "INSERT INTO question (id_quiz, position, intitule, explication)
             VALUES ($1, $2, $3, $4) RETURNING id_question",
        )
        .bind(id_quiz)
        .bind(qpos as i32)
        .bind(&q.intitule)
        .bind(&q.explication)
        .fetch_one(&mut *tx)
        .await?;

        let mut choix_pub = Vec::new();
        for (cpos, texte) in q.choix.iter().enumerate() {
            let (id_choix,): (i32,) = sqlx::query_as(
                "INSERT INTO choix (id_question, position, texte, est_correcte)
                 VALUES ($1, $2, $3, $4) RETURNING id_choix",
            )
            .bind(id_question)
            .bind(cpos as i32)
            .bind(texte)
            .bind(cpos == q.bonne_reponse)
            .fetch_one(&mut *tx)
            .await?;
            choix_pub.push(ChoixPublic {
                id_choix,
                position: cpos as i32,
                texte: texte.clone(),
            });
        }

        questions.push(QuestionPublic {
            id_question,
            position: qpos as i32,
            intitule: q.intitule.clone(),
            choix: choix_pub,
        });
    }

    let (id_session,): (i32,) = sqlx::query_as(
        "INSERT INTO session (id_quiz, id_utilisateur) VALUES ($1, $2) RETURNING id_session",
    )
    .bind(id_quiz)
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(QuizSessionDto {
        id_session,
        id_quiz,
        theme,
        difficulte: req.difficulte,
        temps_par_question_ms: rules::TEMPS_PAR_QUESTION_MS,
        questions,
    }))
}

// ============ Répondre à une question ============

#[derive(Deserialize)]
pub struct AnswerRequest {
    pub id_question: i32,
    pub id_choix: Option<i32>,
    pub temps_ms: i32,
}

#[derive(Serialize)]
pub struct AnswerResponse {
    pub correcte: bool,
    pub id_choix_correct: i32,
    pub explication: Option<String>,
    pub points: i32,
    pub score_total: i32,
    pub serie_actuelle: i32,
    pub nb_bonnes: i32,
}

/// POST /sessions/{id}/answers — enregistre une réponse et met à jour le score.
pub async fn repondre(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id_session): Path<i32>,
    Json(req): Json<AnswerRequest>,
) -> Result<Json<AnswerResponse>, AppError> {
    let sess: Option<(i32, i32, bool, i32, i32, i32, i32)> = sqlx::query_as(
        "SELECT id_utilisateur, id_quiz, termine, score, nb_bonnes, serie_courante, serie_max
         FROM session WHERE id_session = $1",
    )
    .bind(id_session)
    .fetch_optional(&state.pool)
    .await?;
    let (id_user, id_quiz, termine, score, nb_bonnes, serie_courante, serie_max) =
        sess.ok_or(AppError::NotFound)?;
    if id_user != user.id {
        return Err(AppError::NotFound);
    }
    if termine {
        return Err(AppError::BadRequest("cette session est terminée".into()));
    }

    // La question doit appartenir au quiz de la session.
    let correct: Option<(i32, Option<String>)> = sqlx::query_as(
        "SELECT c.id_choix, q.explication
         FROM question q JOIN choix c ON c.id_question = q.id_question
         WHERE q.id_question = $1 AND q.id_quiz = $2 AND c.est_correcte = true",
    )
    .bind(req.id_question)
    .bind(id_quiz)
    .fetch_optional(&state.pool)
    .await?;
    let (id_choix_correct, explication) = correct.ok_or(AppError::NotFound)?;

    let correcte = req.id_choix == Some(id_choix_correct);
    let serie = if correcte { serie_courante + 1 } else { 0 };
    let points = rules::points_reponse(correcte, req.temps_ms, serie);
    let new_score = score + points;
    let new_nb_bonnes = nb_bonnes + i32::from(correcte);
    let new_serie_max = serie_max.max(serie);

    let mut tx = state.pool.begin().await?;

    sqlx::query(
        "INSERT INTO reponse (id_session, id_question, id_choix, correcte, temps_ms, points)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id_session)
    .bind(req.id_question)
    .bind(req.id_choix)
    .bind(correcte)
    .bind(req.temps_ms.max(0))
    .bind(points)
    .execute(&mut *tx)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::BadRequest("question déjà répondue".into())
        }
        _ => AppError::Database(e),
    })?;

    sqlx::query(
        "UPDATE session SET score = $1, nb_bonnes = $2, serie_courante = $3, serie_max = $4
         WHERE id_session = $5",
    )
    .bind(new_score)
    .bind(new_nb_bonnes)
    .bind(serie)
    .bind(new_serie_max)
    .bind(id_session)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(AnswerResponse {
        correcte,
        id_choix_correct,
        explication,
        points,
        score_total: new_score,
        serie_actuelle: serie,
        nb_bonnes: new_nb_bonnes,
    }))
}

// ============ Terminer une session ============

#[derive(Serialize)]
pub struct FinishResponse {
    pub score: i32,
    pub nb_bonnes: i32,
    pub serie_max: i32,
    pub total_questions: i64,
    pub rang: i64,
}

/// POST /sessions/{id}/finish — clôt la session et renvoie le récapitulatif.
pub async fn terminer(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id_session): Path<i32>,
) -> Result<Json<FinishResponse>, AppError> {
    let sess: Option<(i32, i32, i32, i32, i32)> = sqlx::query_as(
        "SELECT id_utilisateur, id_quiz, score, nb_bonnes, serie_max
         FROM session WHERE id_session = $1",
    )
    .bind(id_session)
    .fetch_optional(&state.pool)
    .await?;
    let (id_user, id_quiz, score, nb_bonnes, serie_max) = sess.ok_or(AppError::NotFound)?;
    if id_user != user.id {
        return Err(AppError::NotFound);
    }

    sqlx::query("UPDATE session SET termine = true WHERE id_session = $1")
        .bind(id_session)
        .execute(&state.pool)
        .await?;

    let (total_questions,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM question WHERE id_quiz = $1")
            .bind(id_quiz)
            .fetch_one(&state.pool)
            .await?;

    let (meilleurs,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM session WHERE termine = true AND score > $1")
            .bind(score)
            .fetch_one(&state.pool)
            .await?;

    Ok(Json(FinishResponse {
        score,
        nb_bonnes,
        serie_max,
        total_questions,
        rang: meilleurs + 1,
    }))
}

// ============ Classement & historique ============

#[derive(Deserialize)]
pub struct LeaderboardParams {
    pub id_categorie: Option<i32>,
    pub limit: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    pub pseudo: String,
    pub theme: String,
    pub difficulte: String,
    pub score: i32,
    pub nb_bonnes: i32,
    pub created_at: DateTime<Utc>,
}

/// GET /leaderboard — meilleurs scores (optionnellement filtrés par catégorie).
pub async fn leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardParams>,
) -> Result<Json<Vec<LeaderboardEntry>>, AppError> {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let entries = sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT u.pseudo, q.theme, q.difficulte, s.score, s.nb_bonnes, s.created_at
         FROM session s
         JOIN utilisateur u ON u.id_utilisateur = s.id_utilisateur
         JOIN quiz q ON q.id_quiz = s.id_quiz
         WHERE s.termine = true AND ($1::int IS NULL OR q.id_categorie = $1)
         ORDER BY s.score DESC, s.created_at ASC
         LIMIT $2",
    )
    .bind(params.id_categorie)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(entries))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct HistoryEntry {
    pub id_session: i32,
    pub theme: String,
    pub difficulte: String,
    pub score: i32,
    pub nb_bonnes: i32,
    pub termine: bool,
    pub created_at: DateTime<Utc>,
}

/// GET /sessions — historique des parties de l'utilisateur connecté.
pub async fn historique(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<HistoryEntry>>, AppError> {
    let rows = sqlx::query_as::<_, HistoryEntry>(
        "SELECT s.id_session, q.theme, q.difficulte, s.score, s.nb_bonnes, s.termine, s.created_at
         FROM session s JOIN quiz q ON q.id_quiz = s.id_quiz
         WHERE s.id_utilisateur = $1
         ORDER BY s.created_at DESC
         LIMIT 50",
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

/// Routes du jeu de quiz.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/quiz", post(generer))
        .route("/sessions", get(historique))
        .route("/sessions/{id}/answers", post(repondre))
        .route("/sessions/{id}/finish", post(terminer))
        .route("/leaderboard", get(leaderboard))
}
