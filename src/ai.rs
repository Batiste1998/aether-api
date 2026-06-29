use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ResponseFormat,
    },
    Client,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Un jet de dé décidé par le Maître du Jeu.
#[derive(Debug, Serialize, Deserialize)]
pub struct De {
    pub raison: String,
    pub de: String,
    pub resultat: i32,
    pub reussite: bool,
}

/// Variations d'état appliquées au personnage après l'action.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Changements {
    #[serde(default)]
    pub pv_delta: i32,
    #[serde(default)]
    pub xp_delta: i32,
    #[serde(default)]
    pub or_delta: i32,
}

/// Quête créée ou mise à jour par le MJ.
#[derive(Debug, Serialize, Deserialize)]
pub struct QueteOut {
    pub titre: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub statut: Option<String>,
    #[serde(default)]
    pub recompense_xp: i32,
    #[serde(default)]
    pub recompense_or: i32,
}

/// PNJ introduit dans la scène par le MJ.
#[derive(Debug, Serialize, Deserialize)]
pub struct PnjOut {
    pub nom: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub attitude: Option<String>,
}

/// Objet gagné ou perdu par le personnage.
#[derive(Debug, Serialize, Deserialize)]
pub struct ObjetOut {
    pub nom: String,
    #[serde(rename = "type", default)]
    pub type_objet: Option<String>,
    #[serde(default)]
    pub effet: Option<String>,
    #[serde(default)]
    pub quantite: Option<i32>,
}

/// Sortie complète et structurée du Maître du Jeu pour un tour.
#[derive(Debug, Serialize, Deserialize)]
pub struct MjOutput {
    pub narration: String,
    #[serde(default)]
    pub jets_de_des: Vec<De>,
    #[serde(default)]
    pub changements_etat: Changements,
    #[serde(default)]
    pub quetes: Vec<QueteOut>,
    #[serde(default)]
    pub pnj: Vec<PnjOut>,
    #[serde(default)]
    pub objets_ajoutes: Vec<ObjetOut>,
    #[serde(default)]
    pub objets_retires: Vec<ObjetOut>,
}

const SYSTEME: &str = r#"Tu es le Maître du Jeu (MJ) de « Chroniques d'Æther », un jeu de rôle textuel dans un univers médiéval-fantastique. Tu animes l'aventure d'un seul joueur.

À partir de la fiche du personnage, de l'historique récent et de l'action du joueur, tu fais avancer l'histoire de façon immersive et cohérente, et tu mets à jour l'état du jeu.

Règles :
- Écris la narration en français, à la 2e personne, en 2 à 5 phrases vivantes.
- Reste cohérent avec l'historique et la fiche du personnage.
- Les actions risquées ou les combats se résolvent par un jet de dé d20 (résultat entre 1 et 20) ; reporte-le dans "jets_de_des". Un résultat élevé favorise le joueur.
- Les dégâts subis sont des "pv_delta" négatifs, les soins positifs. Ne tue jamais le joueur en un seul tour sans qu'il ait pris un risque manifeste.
- Récompense la progression avec "xp_delta" (entre 5 et 50 selon l'enjeu) et parfois "or_delta".
- Crée une quête (dans "quetes") quand c'est narrativement pertinent ; passe son "statut" à "reussie" ou "echouee" quand elle se conclut.
- Introduis des PNJ (dans "pnj") quand la scène en fait apparaître.
- Quand le joueur trouve, reçoit ou ramasse un objet, ajoute-le dans "objets_ajoutes". Quand il en consomme, perd ou donne un, mets-le dans "objets_retires". Le champ "type" vaut "arme", "armure", "consommable" ou "cle".
- Si l'action est le tout premier tour, plante le décor et propose une accroche.

Réponds UNIQUEMENT par un objet JSON valide, sans texte autour, de cette forme exacte :
{
  "narration": "string",
  "jets_de_des": [{"raison": "string", "de": "d20", "resultat": 0, "reussite": true}],
  "changements_etat": {"pv_delta": 0, "xp_delta": 0, "or_delta": 0},
  "quetes": [{"titre": "string", "description": "string", "statut": "active", "recompense_xp": 0, "recompense_or": 0}],
  "pnj": [{"nom": "string", "description": "string", "attitude": "neutre"}],
  "objets_ajoutes": [{"nom": "string", "type": "consommable", "effet": "string", "quantite": 1}],
  "objets_retires": [{"nom": "string", "quantite": 1}]
}
Les tableaux peuvent être vides."#;

/// Appelle GPT-4o pour jouer un tour et renvoie sa sortie structurée.
pub async fn jouer_tour(
    client: &Client<OpenAIConfig>,
    model: &str,
    contexte: &str,
) -> Result<MjOutput, AppError> {
    let build_err = |e: async_openai::error::OpenAIError| {
        tracing::error!("construction de la requête OpenAI: {e}");
        AppError::Ai
    };

    let system = ChatCompletionRequestSystemMessageArgs::default()
        .content(SYSTEME)
        .build()
        .map_err(build_err)?;
    let user = ChatCompletionRequestUserMessageArgs::default()
        .content(contexte)
        .build()
        .map_err(build_err)?;

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .response_format(ResponseFormat::JsonObject)
        .messages(vec![system.into(), user.into()])
        .build()
        .map_err(build_err)?;

    let response = client.chat().create(request).await.map_err(|e| {
        tracing::error!("appel OpenAI échoué: {e}");
        AppError::Ai
    })?;

    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .ok_or(AppError::Ai)?;

    serde_json::from_str(&content).map_err(|e| {
        tracing::error!("JSON du MJ invalide ({e}) : {content}");
        AppError::Ai
    })
}
