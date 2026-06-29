use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Données portées par le JWT.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // id_utilisateur
    pub pseudo: String,
    pub role: String,
    pub exp: usize,
}

/// Génère un token signé valable 7 jours.
pub fn create_token(
    secret: &str,
    user_id: i32,
    pseudo: &str,
    role: &str,
) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::days(7)).timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        pseudo: pseudo.to_string(),
        role: role.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal)
}

/// Vérifie la signature et l'expiration d'un token, puis renvoie ses claims.
pub fn decode_token(secret: &str, token: &str) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}
