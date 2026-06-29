use async_openai::{config::OpenAIConfig, Client};
use sqlx::PgPool;

use crate::config::Config;

/// État partagé injecté dans chaque handler Axum.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub openai: Client<OpenAIConfig>,
}
