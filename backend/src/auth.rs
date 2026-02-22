// Authentication: password hashing, JWT tokens, and middleware.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::Database;

// ── JWT ──────────────────────────────────────────────────────────────

/// JWT secret – in production this should come from an env var.
fn jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "infon-dev-secret-change-in-production".to_string())
        .into_bytes()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64, // user id
    pub username: String,
    pub role: String,
    pub exp: usize, // expiry (unix timestamp)
}

pub fn create_token(user_id: i64, username: &str, role: &str) -> Result<String, String> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&jwt_secret()),
    )
    .map_err(|e| format!("Failed to create token: {e}"))
}

pub fn verify_token(token: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(&jwt_secret()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Invalid token: {e}"))
}

// ── Password hashing ─────────────────────────────────────────────────

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("Failed to hash password: {e}"))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| format!("Invalid password hash: {e}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// ── Axum extractor: AuthUser ─────────────────────────────────────────

/// Extracts the authenticated user from the Authorization header.
/// Usage: `AuthUser(claims)` in handler parameters.
#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "Missing Authorization header"})),
                )
            })?;

        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid Authorization header format"})),
            )
        })?;

        let claims = verify_token(token).map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": e})),
            )
        })?;

        Ok(AuthUser(claims))
    }
}

/// Optional auth extractor – does not reject if no token is present.
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<Claims>);

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok());

        let Some(header) = auth_header else {
            return Ok(OptionalAuthUser(None));
        };

        let Some(token) = header.strip_prefix("Bearer ") else {
            return Ok(OptionalAuthUser(None));
        };

        match verify_token(token) {
            Ok(claims) => Ok(OptionalAuthUser(Some(claims))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}

// ── Auth API handlers ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Serialize)]
pub struct UserPublic {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub created_at: String,
}

pub async fn register(
    State(db): State<Arc<Database>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    if req.username.is_empty() || req.password.is_empty() || req.email.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "username, email, and password are required"})),
        )
            .into_response();
    }

    if req.username.len() < 3 || req.username.len() > 30 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "username must be 3-30 characters"})),
        )
            .into_response();
    }

    if req.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "password must be at least 8 characters"})),
        )
            .into_response();
    }

    let password_hash = match hash_password(&req.password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Password hash error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal error"})),
            )
                .into_response();
        }
    };

    let display_name = req.display_name.unwrap_or_else(|| req.username.clone());

    match db
        .create_user(&req.username, &req.email, &password_hash, &display_name)
        .await
    {
        Ok(user) => {
            let token = match create_token(user.id, &user.username, &user.role) {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("Token creation error: {e}");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Internal error"})),
                    )
                        .into_response();
                }
            };
            (
                StatusCode::CREATED,
                Json(serde_json::json!(AuthResponse {
                    token,
                    user: UserPublic {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        display_name: user.display_name,
                        role: user.role,
                        created_at: user.created_at,
                    },
                })),
            )
                .into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({"error": "Username or email already taken"})),
                )
                    .into_response()
            } else {
                tracing::error!("DB error in register: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Internal error"})),
                )
                    .into_response()
            }
        }
    }
}

pub async fn login(
    State(db): State<Arc<Database>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match db.get_user_by_username(&req.username).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid username or password"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("DB error in login: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal error"})),
            )
                .into_response();
        }
    };

    let Some(ref password_hash) = user.password_hash else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "This account uses OAuth login"})),
        )
            .into_response();
    };

    match verify_password(&req.password, password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid username or password"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Password verify error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal error"})),
            )
                .into_response();
        }
    }

    let token = match create_token(user.id, &user.username, &user.role) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Token creation error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal error"})),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(serde_json::json!(AuthResponse {
            token,
            user: UserPublic {
                id: user.id,
                username: user.username,
                email: user.email,
                display_name: user.display_name,
                role: user.role,
                created_at: user.created_at,
            },
        })),
    )
        .into_response()
}

pub async fn me(AuthUser(claims): AuthUser, State(db): State<Arc<Database>>) -> impl IntoResponse {
    match db.get_user(claims.sub).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::json!(UserPublic {
                id: user.id,
                username: user.username,
                email: user.email,
                display_name: user.display_name,
                role: user.role,
                created_at: user.created_at,
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("DB error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal error"})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "testpassword123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrongpassword", &hash).unwrap());
    }

    #[test]
    fn test_jwt_create_and_verify() {
        let token = create_token(1, "testuser", "user").unwrap();
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn test_jwt_invalid_token() {
        assert!(verify_token("invalid.token.here").is_err());
    }
}
