use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub default_admin_username: String,
    pub default_admin_password: String,
}

pub fn load_config() -> AppConfig {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable is required. Set it to your PostgreSQL connection string.");

    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is required. Set it to a strong, random secret key.");

    let default_admin_username = env::var("DEFAULT_ADMIN_USERNAME")
        .unwrap_or_else(|_| "admin".to_string());

    let default_admin_password = env::var("DEFAULT_ADMIN_PASSWORD")
        .unwrap_or_else(|_| "admin123".to_string());

    if jwt_secret.len() < 16 {
        panic!("JWT_SECRET must be at least 16 characters long for security.");
    }

    if default_admin_username.is_empty() {
        panic!("DEFAULT_ADMIN_USERNAME must not be empty.");
    }

    if default_admin_password.len() < 8 {
        panic!("DEFAULT_ADMIN_PASSWORD must be at least 8 characters long.");
    }

    AppConfig {
        database_url,
        jwt_secret,
        default_admin_username,
        default_admin_password,
    }
}