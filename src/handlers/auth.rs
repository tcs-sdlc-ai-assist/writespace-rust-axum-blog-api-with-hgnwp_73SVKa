use axum::{http::StatusCode, Json};
use sqlx::PgPool;

use crate::errors::AppError;
use crate::middleware::auth::encode_jwt;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User, UserInfo};

pub async fn login_handler(
    axum::extract::Extension(pool): axum::extract::Extension<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(AppError::BadRequest(
            "Username and password are required".to_string(),
        ));
    }

    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, display_name, username, password_hash, role, is_default_admin, created_at FROM users WHERE username = $1",
    )
    .bind(&payload.username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Database error: {}", e)))?
    .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    let password_valid = bcrypt::verify(&payload.password, &user.password_hash)
        .map_err(|e| AppError::InternalError(format!("Password verification error: {}", e)))?;

    if !password_valid {
        return Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    let token = encode_jwt(user.id, &user.username, &user.display_name, &user.role)?;

    let response = AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn register_handler(
    axum::extract::Extension(pool): axum::extract::Extension<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    if payload.username.is_empty() {
        return Err(AppError::BadRequest("Username is required".to_string()));
    }

    if payload.display_name.is_empty() {
        return Err(AppError::BadRequest(
            "Display name is required".to_string(),
        ));
    }

    if payload.password.is_empty() {
        return Err(AppError::BadRequest("Password is required".to_string()));
    }

    if payload.username.len() < 3 || payload.username.len() > 50 {
        return Err(AppError::BadRequest(
            "Username must be between 3 and 50 characters".to_string(),
        ));
    }

    if !payload
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err(AppError::BadRequest(
            "Username must contain only alphanumeric characters and underscores".to_string(),
        ));
    }

    if payload.display_name.len() > 100 {
        return Err(AppError::BadRequest(
            "Display name must be at most 100 characters".to_string(),
        ));
    }

    if payload.password.len() < 8 || payload.password.len() > 72 {
        return Err(AppError::BadRequest(
            "Password must be between 8 and 72 characters".to_string(),
        ));
    }

    if payload.password.chars().any(|c| c.is_whitespace()) {
        return Err(AppError::BadRequest(
            "Password must not contain whitespace".to_string(),
        ));
    }

    let existing: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = $1")
            .bind(&payload.username)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::InternalError(format!("Database error: {}", e)))?;

    if existing.is_some() {
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    let password_hash = bcrypt::hash(&payload.password, 12)
        .map_err(|e| AppError::InternalError(format!("Password hashing error: {}", e)))?;

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (display_name, username, password_hash, role, is_default_admin) VALUES ($1, $2, $3, 'user', false) RETURNING id, display_name, username, password_hash, role, is_default_admin, created_at",
    )
    .bind(&payload.display_name)
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return AppError::Conflict("Username already exists".to_string());
                }
            }
        }
        AppError::InternalError(format!("Failed to create user: {}", e))
    })?;

    let token = encode_jwt(user.id, &user.username, &user.display_name, &user.role)?;

    let response = AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}