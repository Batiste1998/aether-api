use std::env;

/// Configuration de l'application, chargée depuis l'environnement (`.env`).
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub jwt_secret: String,
    pub openai_model: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL manquant dans l'environnement".to_string())?,
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-a-changer".to_string()),
            openai_model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
        })
    }
}
