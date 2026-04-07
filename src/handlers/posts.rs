use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    AuthorInfo, Claims, CreatePostRequest, Post, PostResponse, PostSummary, UpdatePostRequest,
};

pub async fn list_posts(
    Extension(pool): Extension<PgPool>,
    claims: Option<Extension<Claims>>,
) -> Result<Json<Vec<PostSummary>>, AppError> {
    let rows: Vec<(Uuid, String, chrono::DateTime<chrono::Utc>, Uuid, String, String)> =
        if claims.is_some() {
            sqlx::query_as(
                "SELECT p.id, p.title, p.created_at, u.id as author_id, u.display_name, u.role \
                 FROM posts p \
                 JOIN users u ON p.author_id = u.id \
                 ORDER BY p.created_at DESC",
            )
            .fetch_all(&pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT p.id, p.title, p.created_at, u.id as author_id, u.display_name, u.role \
                 FROM posts p \
                 JOIN users u ON p.author_id = u.id \
                 ORDER BY p.created_at DESC \
                 LIMIT 3",
            )
            .fetch_all(&pool)
            .await?
        };

    let posts: Vec<PostSummary> = rows
        .into_iter()
        .map(
            |(id, title, created_at, author_id, display_name, role)| PostSummary {
                id,
                title,
                created_at,
                author: AuthorInfo {
                    id: author_id,
                    display_name,
                    role,
                },
            },
        )
        .collect();

    Ok(Json(posts))
}

pub async fn get_post(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    claims: Option<Extension<Claims>>,
) -> Result<Json<PostResponse>, AppError> {
    let row: Option<(
        Uuid,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
        Uuid,
        String,
        String,
    )> = sqlx::query_as(
        "SELECT p.id, p.title, p.content, p.created_at, u.id as author_id, u.display_name, u.role \
         FROM posts p \
         JOIN users u ON p.author_id = u.id \
         WHERE p.id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?;

    let (post_id, title, content, created_at, author_id, display_name, role) =
        row.ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    let (can_edit, can_delete) = if let Some(Extension(ref c)) = claims {
        let is_owner = c.sub == author_id;
        let is_admin = c.role == "admin";
        (Some(is_owner || is_admin), Some(is_owner || is_admin))
    } else {
        (None, None)
    };

    let response = PostResponse {
        id: post_id,
        title,
        content,
        created_at,
        author: AuthorInfo {
            id: author_id,
            display_name,
            role,
        },
        can_edit,
        can_delete,
    };

    Ok(Json(response))
}

pub async fn create_post(
    Extension(pool): Extension<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostResponse>), AppError> {
    let title = payload.title.trim().to_string();
    let content = payload.content.trim().to_string();

    if title.is_empty() || content.is_empty() {
        return Err(AppError::BadRequest(
            "Title and content are required".to_string(),
        ));
    }

    if title.len() > 200 {
        return Err(AppError::BadRequest(
            "Title must be at most 200 characters".to_string(),
        ));
    }

    let post: Post = sqlx::query_as(
        "INSERT INTO posts (title, content, author_id) VALUES ($1, $2, $3) \
         RETURNING id, title, content, created_at, author_id",
    )
    .bind(&title)
    .bind(&content)
    .bind(claims.sub)
    .fetch_one(&pool)
    .await?;

    let response = PostResponse {
        id: post.id,
        title: post.title,
        content: post.content,
        created_at: post.created_at,
        author: AuthorInfo {
            id: claims.sub,
            display_name: claims.display_name,
            role: claims.role,
        },
        can_edit: None,
        can_delete: None,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_post(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<PostResponse>, AppError> {
    let title = payload.title.trim().to_string();
    let content = payload.content.trim().to_string();

    if title.is_empty() || content.is_empty() {
        return Err(AppError::BadRequest(
            "Title and content are required".to_string(),
        ));
    }

    if title.len() > 200 {
        return Err(AppError::BadRequest(
            "Title must be at most 200 characters".to_string(),
        ));
    }

    let existing: Post = sqlx::query_as(
        "SELECT id, title, content, created_at, author_id FROM posts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    let is_owner = claims.sub == existing.author_id;
    let is_admin = claims.role == "admin";

    if !is_owner && !is_admin {
        return Err(AppError::Forbidden(
            "Not authorized to edit this post".to_string(),
        ));
    }

    let updated: Post = sqlx::query_as(
        "UPDATE posts SET title = $1, content = $2 WHERE id = $3 \
         RETURNING id, title, content, created_at, author_id",
    )
    .bind(&title)
    .bind(&content)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    let author: (Uuid, String, String) = sqlx::query_as(
        "SELECT id, display_name, role FROM users WHERE id = $1",
    )
    .bind(updated.author_id)
    .fetch_one(&pool)
    .await?;

    let response = PostResponse {
        id: updated.id,
        title: updated.title,
        content: updated.content,
        created_at: updated.created_at,
        author: AuthorInfo {
            id: author.0,
            display_name: author.1,
            role: author.2,
        },
        can_edit: None,
        can_delete: None,
    };

    Ok(Json(response))
}

pub async fn delete_post(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<StatusCode, AppError> {
    let existing: Post = sqlx::query_as(
        "SELECT id, title, content, created_at, author_id FROM posts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    let is_owner = claims.sub == existing.author_id;
    let is_admin = claims.role == "admin";

    if !is_owner && !is_admin {
        return Err(AppError::Forbidden(
            "Not authorized to delete this post".to_string(),
        ));
    }

    sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}