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

/// Une question générée par l'IA.
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionGeneree {
    pub intitule: String,
    pub choix: Vec<String>,
    /// Index (0-3) de la bonne réponse dans `choix`.
    pub bonne_reponse: usize,
    #[serde(default)]
    pub explication: Option<String>,
}

/// Quiz complet généré par l'IA.
#[derive(Debug, Serialize, Deserialize)]
pub struct QuizGenere {
    pub questions: Vec<QuestionGeneree>,
}

const SYSTEME: &str = r#"Tu es un générateur de quiz pour un jeu de culture générale en français, dans l'esprit de Kahoot.

On te donne un thème, une difficulté et un nombre de questions. Tu produis des questions à choix multiples de qualité.

Règles :
- Rédige tout en français.
- Chaque question a EXACTEMENT 4 propositions, dont une seule correcte.
- "bonne_reponse" est l'index (0, 1, 2 ou 3) de la bonne proposition dans "choix".
- Mélange la position de la bonne réponse d'une question à l'autre.
- Les propositions fausses doivent être plausibles mais clairement incorrectes.
- Adapte la difficulté : "facile" = grand public, "moyen" = amateur éclairé, "difficile" = expert.
- Ajoute une "explication" courte (1 phrase) justifiant la bonne réponse.
- Pas de question d'opinion ni d'ambiguïté : une seule réponse défendable.

Réponds UNIQUEMENT par un objet JSON valide de cette forme exacte :
{
  "questions": [
    {
      "intitule": "string",
      "choix": ["string", "string", "string", "string"],
      "bonne_reponse": 0,
      "explication": "string"
    }
  ]
}"#;

/// Demande à GPT-4o de générer un quiz et renvoie sa sortie structurée.
pub async fn generer_quiz(
    client: &Client<OpenAIConfig>,
    model: &str,
    theme: &str,
    difficulte: &str,
    nb_questions: i32,
) -> Result<QuizGenere, AppError> {
    let build_err = |e: async_openai::error::OpenAIError| {
        tracing::error!("construction de la requête OpenAI: {e}");
        AppError::Ai
    };

    let consigne =
        format!("Thème : {theme}\nDifficulté : {difficulte}\nNombre de questions : {nb_questions}");

    let system = ChatCompletionRequestSystemMessageArgs::default()
        .content(SYSTEME)
        .build()
        .map_err(build_err)?;
    let user = ChatCompletionRequestUserMessageArgs::default()
        .content(consigne)
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

    let quiz: QuizGenere = serde_json::from_str(&content).map_err(|e| {
        tracing::error!("JSON du quiz invalide ({e}) : {content}");
        AppError::Ai
    })?;

    // Validation : 4 choix par question, index de bonne réponse cohérent.
    if quiz.questions.is_empty()
        || quiz
            .questions
            .iter()
            .any(|q| q.choix.len() != 4 || q.bonne_reponse > 3)
    {
        tracing::error!("structure de quiz invalide renvoyée par l'IA");
        return Err(AppError::Ai);
    }

    Ok(quiz)
}
