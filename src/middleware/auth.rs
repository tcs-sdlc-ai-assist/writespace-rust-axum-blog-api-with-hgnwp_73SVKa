use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::Claims;

pub fn encode_jwt(
    user_id: Uuid,
    username: &str,
    display_name: &str,
    role: &str,
) -> Result<String, AppError> {
    let jwt_secret = env::var("JWT_SECRET")
        .map_err(|_| AppError::InternalError("JWT_SECRET not configured".to_string()))?;

    let now = chrono::Utc::now();
    let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        display_name: display_name.to_string(),
        role: role.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn decode_jwt(token: &str) -> Result<Claims, AppError> {
    let jwt_secret = env::var("JWT_SECRET")
        .map_err(|_| AppError::InternalError("JWT_SECRET not configured".to_string()))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::Unauthorized("Token has expired".to_string())
        }
        jsonwebtoken::errors::ErrorKind::InvalidToken => {
            AppError::Unauthorized("Invalid token".to_string())
        }
        _ => AppError::Unauthorized(format!("Invalid token: {}", e)),
    })?;

    Ok(token_data.claims)
}

fn extract_token_from_header(req: &Request) -> Option<String> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            if value.starts_with("Bearer ") {
                Some(value[7..].to_string())
            } else {
                None
            }
        })
}

pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, AppError> {
    let token = extract_token_from_header(&req).ok_or_else(|| {
        AppError::Unauthorized("Missing or invalid Authorization header".to_string())
    })?;

    let claims = decode_jwt(&token)?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

pub async fn require_admin(req: Request, next: Next) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    if claims.role != "admin" {
        return Err(AppError::Forbidden(
            "Admin access required".to_string(),
        ));
    }

    Ok(next.run(req).await)
}

pub async fn optional_auth(mut req: Request, next: Next) -> Response {
    if let Some(token) = extract_token_from_header(&req) {
        if let Ok(claims) = decode_jwt(&token) {
            req.extensions_mut().insert(claims);
        }
    }

    next.run(req).await
}