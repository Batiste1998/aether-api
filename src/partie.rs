use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{ai, auth::AuthUser, error::AppError, state::AppState};

#[derive(Serialize, sqlx::FromRow)]
pub struct PartieDto {
    pub id_partie: i32,
    pub titre: String,
    pub statut: String,
    pub theme: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id_personnage: i32,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct TourDto {
    pub id_tour: i32,
    pub numero: i32,
    pub action_joueur: Option<String>,
    pub narration_ia: String,
    pub etat_jeu: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct QueteDto {
    pub id_quete: i32,
    pub titre: String,
    pub description: Option<String>,
    pub statut: String,
    pub recompense_xp: i32,
    pub recompense_or: i32,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct PnjDto {
    pub id_pnj: i32,
    pub nom: String,
    pub description: Option<String>,
    pub attitude: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct InventaireItem {
    pub id_objet: i32,
    pub nom: String,
    pub type_objet: String,
    pub description: Option<String>,
    pub effet: Option<String>,
    pub quantite: i32,
    pub equipe: bool,
}

const PARTIE_COLS: &str = "id_partie, titre, statut, theme, created_at, updated_at, id_personnage";

/// Vérifie qu'une partie appartient à l'utilisateur ; renvoie (id_personnage, statut).
async fn partie_owned(
    pool: &sqlx::PgPool,
    user_id: i32,
    partie_id: i32,
) -> Result<Option<(i32, String)>, AppError> {
    let row = sqlx::query_as(
        "SELECT p.id_personnage, p.statut
         FROM partie p
         JOIN personnage pe ON pe.id_personnage = p.id_personnage
         WHERE p.id_partie = $1 AND pe.id_utilisateur = $2",
    )
    .bind(partie_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

// ============ Gestion des parties ============

#[derive(Deserialize)]
pub struct CreatePartie {
    pub id_personnage: i32,
    pub titre: Option<String>,
    pub theme: Option<String>,
}

/// POST /parties — démarre une nouvelle partie pour un de ses personnages.
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreatePartie>,
) -> Result<(StatusCode, Json<PartieDto>), AppError> {
    let owns: Option<(i32,)> =
        sqlx::query_as("SELECT id_personnage FROM personnage WHERE id_personnage = $1 AND id_utilisateur = $2")
            .bind(req.id_personnage)
            .bind(user.id)
            .fetch_optional(&state.pool)
            .await?;
    owns.ok_or(AppError::NotFound)?;

    let titre = req
        .titre
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| "Nouvelle aventure".to_string());

    let partie: PartieDto = sqlx::query_as(&format!(
        "INSERT INTO partie (titre, theme, id_personnage) VALUES ($1, $2, $3) RETURNING {PARTIE_COLS}"
    ))
    .bind(&titre)
    .bind(&req.theme)
    .bind(req.id_personnage)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(partie)))
}

/// GET /parties — liste les parties de l'utilisateur connecté.
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<PartieDto>>, AppError> {
    let parties = sqlx::query_as::<_, PartieDto>(
        "SELECT p.id_partie, p.titre, p.statut, p.theme, p.created_at, p.updated_at, p.id_personnage
         FROM partie p
         JOIN personnage pe ON pe.id_personnage = p.id_personnage
         WHERE pe.id_utilisateur = $1
         ORDER BY p.updated_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(parties))
}

#[derive(Serialize)]
pub struct PartieDetail {
    #[serde(flatten)]
    pub partie: PartieDto,
    pub tours: Vec<TourDto>,
    pub quetes: Vec<QueteDto>,
    pub pnj: Vec<PnjDto>,
    pub inventaire: Vec<InventaireItem>,
}

/// GET /parties/{id} — détail complet : fil narratif, quêtes, PNJ, inventaire.
pub async fn detail(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<PartieDetail>, AppError> {
    let (id_personnage, _) = partie_owned(&state.pool, user.id, id)
        .await?
        .ok_or(AppError::NotFound)?;

    let partie: PartieDto =
        sqlx::query_as(&format!("SELECT {PARTIE_COLS} FROM partie WHERE id_partie = $1"))
            .bind(id)
            .fetch_one(&state.pool)
            .await?;

    let tours = sqlx::query_as::<_, TourDto>(
        "SELECT id_tour, numero, action_joueur, narration_ia, etat_jeu, created_at
         FROM tour WHERE id_partie = $1 ORDER BY numero ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let quetes = sqlx::query_as::<_, QueteDto>(
        "SELECT id_quete, titre, description, statut, recompense_xp, recompense_or
         FROM quete WHERE id_partie = $1 ORDER BY id_quete ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let pnj = sqlx::query_as::<_, PnjDto>(
        "SELECT id_pnj, nom, description, attitude FROM pnj WHERE id_partie = $1 ORDER BY id_pnj ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let inventaire = sqlx::query_as::<_, InventaireItem>(
        "SELECT o.id_objet, o.nom, o.type AS type_objet, o.description, o.effet, i.quantite, i.equipe
         FROM inventaire i JOIN objet o ON o.id_objet = i.id_objet
         WHERE i.id_personnage = $1 ORDER BY o.nom ASC",
    )
    .bind(id_personnage)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PartieDetail {
        partie,
        tours,
        quetes,
        pnj,
        inventaire,
    }))
}

// ============ Le tour de jeu (Maître du Jeu IA) ============

#[derive(Deserialize)]
pub struct ActionRequest {
    #[serde(default)]
    pub action: String,
}

#[derive(Serialize)]
pub struct PersoSnapshot {
    pub niveau: i32,
    pub xp: i32,
    pub pv_actuels: i32,
    pub pv_max: i32,
    pub or_pieces: i32,
}

#[derive(Serialize)]
pub struct TourResponse {
    pub numero: i32,
    pub narration: String,
    pub jets_de_des: Vec<ai::De>,
    pub personnage: PersoSnapshot,
}

#[derive(sqlx::FromRow)]
struct PersoEtat {
    nom: String,
    niveau: i32,
    xp: i32,
    pv_actuels: i32,
    pv_max: i32,
    or_pieces: i32,
    histoire: Option<String>,
    classe_nom: String,
    pv_base: i32,
}

/// POST /parties/{id}/tours — joue un tour : appelle le MJ IA, valide et persiste.
pub async fn jouer(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<ActionRequest>,
) -> Result<Json<TourResponse>, AppError> {
    let (id_personnage, statut) = partie_owned(&state.pool, user.id, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if statut != "en_cours" {
        return Err(AppError::BadRequest("cette partie est terminée".into()));
    }

    // 1) État courant du personnage (avec sa classe).
    let perso: PersoEtat = sqlx::query_as(
        "SELECT p.nom, p.niveau, p.xp, p.pv_actuels, p.pv_max, p.or_pieces,
                p.histoire, c.nom AS classe_nom, c.pv_base
         FROM personnage p JOIN classe c ON c.id_classe = p.id_classe
         WHERE p.id_personnage = $1",
    )
    .bind(id_personnage)
    .fetch_one(&state.pool)
    .await?;

    // 2) Contexte narratif : quêtes actives + derniers tours.
    let quetes_actives: Vec<(String,)> =
        sqlx::query_as("SELECT titre FROM quete WHERE id_partie = $1 AND statut = 'active'")
            .bind(id)
            .fetch_all(&state.pool)
            .await?;

    let derniers: Vec<(i32, Option<String>, String)> = sqlx::query_as(
        "SELECT numero, action_joueur, narration_ia FROM tour
         WHERE id_partie = $1 ORDER BY numero DESC LIMIT 8",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let action = req.action.trim();
    let contexte = construire_contexte(&perso, &quetes_actives, &derniers, action);

    // 3) Appel du Maître du Jeu.
    let sortie = ai::jouer_tour(&state.openai, &state.config.openai_model, &contexte).await?;

    // 4) Calcul du nouvel état (XP -> niveau -> PV max -> PV).
    let xp = (perso.xp + sortie.changements_etat.xp_delta).max(0);
    let niveau = 1 + xp / 100;
    let pv_max = perso.pv_base + (niveau - 1) * 20;
    let pv_actuels = (perso.pv_actuels + sortie.changements_etat.pv_delta).clamp(0, pv_max);
    let or_pieces = (perso.or_pieces + sortie.changements_etat.or_delta).max(0);

    let numero = derniers.first().map(|t| t.0 + 1).unwrap_or(1);
    let action_db: Option<&str> = if action.is_empty() { None } else { Some(action) };
    let snapshot = serde_json::to_value(&sortie).map_err(|_| AppError::Internal)?;

    // 5) Persistance atomique.
    let mut tx = state.pool.begin().await?;

    sqlx::query(
        "UPDATE personnage SET niveau = $1, xp = $2, pv_actuels = $3, pv_max = $4, or_pieces = $5
         WHERE id_personnage = $6",
    )
    .bind(niveau)
    .bind(xp)
    .bind(pv_actuels)
    .bind(pv_max)
    .bind(or_pieces)
    .bind(id_personnage)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO tour (numero, action_joueur, narration_ia, etat_jeu, id_partie)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(numero)
    .bind(action_db)
    .bind(&sortie.narration)
    .bind(&snapshot)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    for q in &sortie.quetes {
        sqlx::query(
            "INSERT INTO quete (titre, description, statut, recompense_xp, recompense_or, id_partie)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&q.titre)
        .bind(&q.description)
        .bind(q.statut.as_deref().unwrap_or("active"))
        .bind(q.recompense_xp)
        .bind(q.recompense_or)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    for p in &sortie.pnj {
        sqlx::query("INSERT INTO pnj (nom, description, attitude, id_partie) VALUES ($1, $2, $3, $4)")
            .bind(&p.nom)
            .bind(&p.description)
            .bind(p.attitude.as_deref().unwrap_or("neutre"))
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    // Objets gagnés : on référence (ou crée) l'objet par son nom, puis on cumule la quantité.
    for o in &sortie.objets_ajoutes {
        let qte = o.quantite.unwrap_or(1).max(1);
        let type_objet = match o.type_objet.as_deref() {
            Some(t @ ("arme" | "armure" | "consommable" | "cle")) => t,
            _ => "consommable",
        };
        let (id_objet,): (i32,) = sqlx::query_as(
            "INSERT INTO objet (nom, type, effet) VALUES ($1, $2, $3)
             ON CONFLICT (nom) DO UPDATE SET nom = EXCLUDED.nom
             RETURNING id_objet",
        )
        .bind(&o.nom)
        .bind(type_objet)
        .bind(&o.effet)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO inventaire (id_personnage, id_objet, quantite) VALUES ($1, $2, $3)
             ON CONFLICT (id_personnage, id_objet)
             DO UPDATE SET quantite = inventaire.quantite + EXCLUDED.quantite",
        )
        .bind(id_personnage)
        .bind(id_objet)
        .bind(qte)
        .execute(&mut *tx)
        .await?;
    }

    // Objets perdus : on décrémente, puis on supprime la ligne si la quantité tombe à zéro.
    for o in &sortie.objets_retires {
        let qte = o.quantite.unwrap_or(1).max(1);
        if let Some((id_objet,)) =
            sqlx::query_as::<_, (i32,)>("SELECT id_objet FROM objet WHERE nom = $1")
                .bind(&o.nom)
                .fetch_optional(&mut *tx)
                .await?
        {
            sqlx::query(
                "UPDATE inventaire SET quantite = quantite - $1
                 WHERE id_personnage = $2 AND id_objet = $3",
            )
            .bind(qte)
            .bind(id_personnage)
            .bind(id_objet)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "DELETE FROM inventaire WHERE id_personnage = $1 AND id_objet = $2 AND quantite <= 0",
            )
            .bind(id_personnage)
            .bind(id_objet)
            .execute(&mut *tx)
            .await?;
        }
    }

    sqlx::query("UPDATE partie SET updated_at = now() WHERE id_partie = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(TourResponse {
        numero,
        narration: sortie.narration,
        jets_de_des: sortie.jets_de_des,
        personnage: PersoSnapshot {
            niveau,
            xp,
            pv_actuels,
            pv_max,
            or_pieces,
        },
    }))
}

/// Assemble le contexte texte envoyé au Maître du Jeu.
fn construire_contexte(
    perso: &PersoEtat,
    quetes: &[(String,)],
    derniers: &[(i32, Option<String>, String)],
    action: &str,
) -> String {
    let mut c = String::new();
    c.push_str("FICHE DU PERSONNAGE\n");
    c.push_str(&format!(
        "Nom : {} — Classe : {} — Niveau {}\n",
        perso.nom, perso.classe_nom, perso.niveau
    ));
    c.push_str(&format!(
        "PV : {}/{} — Or : {}\n",
        perso.pv_actuels, perso.pv_max, perso.or_pieces
    ));
    c.push_str(&format!(
        "Histoire : {}\n\n",
        perso.histoire.as_deref().unwrap_or("—")
    ));

    c.push_str("QUÊTES ACTIVES : ");
    if quetes.is_empty() {
        c.push_str("aucune\n\n");
    } else {
        let titres: Vec<&str> = quetes.iter().map(|q| q.0.as_str()).collect();
        c.push_str(&titres.join(", "));
        c.push_str("\n\n");
    }

    c.push_str("HISTORIQUE RÉCENT (du plus ancien au plus récent) :\n");
    if derniers.is_empty() {
        c.push_str("(début de l'aventure)\n");
    } else {
        for (_, act, narr) in derniers.iter().rev() {
            if let Some(a) = act {
                c.push_str(&format!("[Joueur] {a}\n"));
            }
            c.push_str(&format!("[MJ] {narr}\n"));
        }
    }

    c.push_str("\nACTION DU JOUEUR : ");
    if action.is_empty() {
        c.push_str("Commence l'aventure.");
    } else {
        c.push_str(action);
    }
    c
}

/// Routes parties, montées sous `/parties`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create).get(list))
        .route("/{id}", get(detail))
        .route("/{id}/tours", post(jouer))
}
