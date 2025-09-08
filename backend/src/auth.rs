use anyhow::Result;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
}

pub struct AuthService;

impl AuthService {
    pub fn hash_password(password: &str) -> Result<String> {
        let hash = hash(password, DEFAULT_COST)?;
        Ok(hash)
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let is_valid = verify(password, hash)?;
        Ok(is_valid)
    }

    pub fn generate_token(user_id: Uuid, secret: &str) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(24);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}

pub struct AuthUser {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authorization = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(AuthError::MissingToken)?;

        let authorization = authorization
            .to_str()
            .map_err(|_| AuthError::InvalidToken)?;

        if !authorization.starts_with("Bearer ") {
            return Err(AuthError::InvalidToken);
        }

        let token = authorization.trim_start_matches("Bearer ");

        let claims = AuthService::verify_token(token, &state.config.jwt_secret)
            .map_err(|_| AuthError::InvalidToken)?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

        // Verify user still exists
        let user = state
            .db
            .find_user_by_id(user_id)
            .await
            .map_err(|_| AuthError::DatabaseError)?;

        if user.is_none() {
            return Err(AuthError::InvalidToken);
        }

        Ok(AuthUser { user_id })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authorization token")]
    MissingToken,
    #[error("Invalid authorization token")]
    InvalidToken,
    #[error("Database error")]
    DatabaseError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            AuthError::MissingToken | AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string()
        }));

        (status, body).into_response()
    }
}