use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing;

use crate::config::AppConfig;
use crate::errors::AppError;

pub async fn create_pool(database_url: &str) -> Result<PgPool, AppError> {
    tracing::info!("Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to connect to database: {}", e)))?;

    tracing::info!("Database connection established");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    tracing::info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to run migrations: {}", e)))?;

    tracing::info!("Database migrations completed");
    Ok(())
}

pub async fn seed_admin(pool: &PgPool, config: &AppConfig) -> Result<(), AppError> {
    tracing::info!("Checking for default admin user...");

    let existing_admin: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE is_default_admin = true LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to check for default admin: {}", e)))?;

    if existing_admin.is_some() {
        tracing::info!("Default admin user already exists, skipping seed");
        return Ok(());
    }

    tracing::info!("Creating default admin user: {}", config.default_admin_username);

    let password_hash = bcrypt::hash(&config.default_admin_password, 12)
        .map_err(|e| AppError::InternalError(format!("Failed to hash admin password: {}", e)))?;

    sqlx::query(
        "INSERT INTO users (display_name, username, password_hash, role, is_default_admin) VALUES ($1, $2, $3, 'admin', true)"
    )
    .bind(&config.default_admin_username)
    .bind(&config.default_admin_username)
    .bind(&password_hash)
    .execute(pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to create default admin: {}", e)))?;

    tracing::info!("Default admin user created successfully");
    Ok(())
}