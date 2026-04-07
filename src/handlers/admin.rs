use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    AdminUserResponse, Claims, CreateUserRequest, RecentPost, StatsResponse, User,
};

pub async fn stats_handler(
    Extension(pool): Extension<PgPool>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<StatsResponse>, AppError> {
    let total_posts: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to count posts: {}", e)))?;

    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to count users: {}", e)))?;

    let total_admins: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = 'admin'")
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to count admins: {}", e)))?;

    let recent_posts: Vec<RecentPost> = sqlx::query_as(
        "SELECT id, title, created_at FROM posts ORDER BY created_at DESC LIMIT 5",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to fetch recent posts: {}", e)))?;

    Ok(Json(StatsResponse {
        total_posts: total_posts.0,
        total_users: total_users.0,
        total_admins: total_admins.0,
        recent_posts,
    }))
}

pub async fn list_users_handler(
    Extension(pool): Extension<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<AdminUserResponse>>, AppError> {
    let users: Vec<User> = sqlx::query_as(
        "SELECT id, display_name, username, password_hash, role, is_default_admin, created_at FROM users ORDER BY created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to fetch users: {}", e)))?;

    let response: Vec<AdminUserResponse> = users
        .iter()
        .map(|user| {
            let is_deletable = !user.is_default_admin && user.id != claims.sub;
            AdminUserResponse {
                id: user.id,
                username: user.username.clone(),
                display_name: user.display_name.clone(),
                role: user.role.clone(),
                is_deletable,
                created_at: user.created_at,
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_user_handler(
    Extension(pool): Extension<PgPool>,
    Extension(_claims): Extension<Claims>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<AdminUserResponse>), AppError> {
    let username = payload.username.trim().to_string();
    let display_name = payload.display_name.trim().to_string();
    let password = payload.password.clone();
    let role = payload.role.trim().to_lowercase();

    if username.is_empty() || display_name.is_empty() || password.is_empty() {
        return Err(AppError::BadRequest(
            "Username, display name, and password are required".to_string(),
        ));
    }

    if username.len() < 3 || username.len() > 50 {
        return Err(AppError::BadRequest(
            "Username must be between 3 and 50 characters".to_string(),
        ));
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err(AppError::BadRequest(
            "Username must contain only alphanumeric characters and underscores".to_string(),
        ));
    }

    if display_name.len() > 100 {
        return Err(AppError::BadRequest(
            "Display name must be at most 100 characters".to_string(),
        ));
    }

    if password.len() < 8 || password.len() > 72 {
        return Err(AppError::BadRequest(
            "Password must be between 8 and 72 characters".to_string(),
        ));
    }

    if password.chars().any(|c| c.is_whitespace()) {
        return Err(AppError::BadRequest(
            "Password must not contain whitespace".to_string(),
        ));
    }

    if role != "admin" && role != "user" {
        return Err(AppError::BadRequest(
            "Role must be 'admin' or 'user'".to_string(),
        ));
    }

    let existing: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = $1")
            .bind(&username)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Failed to check existing user: {}", e))
            })?;

    if existing.is_some() {
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    let password_hash = bcrypt::hash(&password, 12)
        .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?;

    let user: User = sqlx::query_as(
        "INSERT INTO users (display_name, username, password_hash, role) VALUES ($1, $2, $3, $4) RETURNING id, display_name, username, password_hash, role, is_default_admin, created_at",
    )
    .bind(&display_name)
    .bind(&username)
    .bind(&password_hash)
    .bind(&role)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to create user: {}", e)))?;

    let response = AdminUserResponse {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
        is_deletable: !user.is_default_admin,
        created_at: user.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn delete_user_handler(
    Extension(pool): Extension<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if user_id == claims.sub {
        return Err(AppError::Forbidden(
            "Cannot delete your own account".to_string(),
        ));
    }

    let user: User = sqlx::query_as(
        "SELECT id, display_name, username, password_hash, role, is_default_admin, created_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to fetch user: {}", e)))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if user.is_default_admin {
        return Err(AppError::Forbidden(
            "Cannot delete the default admin account".to_string(),
        ));
    }

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to delete user: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}