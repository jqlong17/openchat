//! HTTP 处理器。

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::error::ApiError;
use crate::jwt;
use crate::password;
use crate::state::AppState;
use crate::HealthResponse;

// --- health ---

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

// --- auth DTO ---

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshBody {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

fn validate_register(body: &RegisterBody) -> Result<(), ApiError> {
    if body.username.is_empty() || body.username.len() > 64 {
        return Err(ApiError::bad_request(
            "invalid_username",
            "username length must be 1–64",
        ));
    }
    if body.password.len() < 8 {
        return Err(ApiError::bad_request(
            "invalid_password",
            "password must be at least 8 characters",
        ));
    }
    if body.display_name.is_empty() || body.display_name.len() > 128 {
        return Err(ApiError::bad_request(
            "invalid_display_name",
            "display_name length must be 1–128",
        ));
    }
    Ok(())
}

pub async fn register_handler(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<AuthResponse>, ApiError> {
    validate_register(&body)?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let hash = password::hash_password(&body.password)
        .map_err(|_| ApiError::internal("password hash failed"))?;

    let r = sqlx::query(
        r#"INSERT INTO users (id, username, display_name, password_hash, status, created_at)
           VALUES (?, ?, ?, ?, 'active', ?)"#,
    )
    .bind(&id)
    .bind(&body.username)
    .bind(&body.display_name)
    .bind(&hash)
    .bind(&now)
    .execute(&state.pool)
    .await;

    if let Err(e) = r {
        if let sqlx::Error::Database(db) = &e {
            if db.is_unique_violation() {
                return Err(ApiError::conflict("username already taken"));
            }
        }
        return Err(e.into());
    }

    let access = jwt::encode_access_token(&id, &state.jwt_secret)?;
    let refresh = jwt::encode_refresh_token(&id, &state.jwt_secret)?;
    Ok(Json(AuthResponse {
        access_token: access,
        refresh_token: refresh,
        expires_in: jwt::ACCESS_TTL_SECS,
    }))
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Json<AuthResponse>, ApiError> {
    let row = sqlx::query("SELECT id, password_hash FROM users WHERE username = ? COLLATE NOCASE")
        .bind(&body.username)
        .fetch_optional(&state.pool)
        .await?;

    let row = row.ok_or_else(|| ApiError::unauthorized("invalid credentials"))?;
    let user_id: String = row.try_get("id")?;
    let phc: String = row.try_get("password_hash")?;

    if !password::verify_password(&body.password, &phc) {
        return Err(ApiError::unauthorized("invalid credentials"));
    }

    let access = jwt::encode_access_token(&user_id, &state.jwt_secret)?;
    let refresh = jwt::encode_refresh_token(&user_id, &state.jwt_secret)?;
    Ok(Json(AuthResponse {
        access_token: access,
        refresh_token: refresh,
        expires_in: jwt::ACCESS_TTL_SECS,
    }))
}

pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(body): Json<RefreshBody>,
) -> Result<Json<AuthResponse>, ApiError> {
    let claims = jwt::decode_token(&body.refresh_token, &state.jwt_secret)?;
    if claims.typ != "refresh" {
        return Err(ApiError::unauthorized("not a refresh token"));
    }
    let user_id = claims.sub;
    let access = jwt::encode_access_token(&user_id, &state.jwt_secret)?;
    let refresh = jwt::encode_refresh_token(&user_id, &state.jwt_secret)?;
    Ok(Json(AuthResponse {
        access_token: access,
        refresh_token: refresh,
        expires_in: jwt::ACCESS_TTL_SECS,
    }))
}

fn bearer_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    let h = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or_else(|| ApiError::unauthorized("missing Authorization"))?;
    let s = h
        .to_str()
        .map_err(|_| ApiError::unauthorized("invalid Authorization"))?;
    s.strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("expected Bearer token"))
}

pub async fn me_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, ApiError> {
    let token = bearer_token(&headers)?;
    let claims = jwt::decode_token(token, &state.jwt_secret)?;
    if claims.typ != "access" {
        return Err(ApiError::unauthorized("not an access token"));
    }
    let row = sqlx::query(
        "SELECT id, username, display_name FROM users WHERE id = ? AND status = 'active'",
    )
    .bind(&claims.sub)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| ApiError::unauthorized("user not found"))?;
    let id: String = row.try_get("id")?;
    let username: String = row.try_get("username")?;
    let display_name: String = row.try_get("display_name")?;
    Ok(Json(UserResponse {
        id,
        username,
        display_name,
        avatar_url: None,
    }))
}
