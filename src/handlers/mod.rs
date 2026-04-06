use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{CreateUserRequest, UpdateUserRequest, User, UserListResponse},
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}
fn default_page() -> i64 { 1 }
fn default_limit() -> i64 { 20 }

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".into()));
    }
    if !body.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".into()));
    }
    if body.age < 0 || body.age > 150 {
        return Err(AppError::BadRequest("Age must be between 0 and 150".into()));
    }

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (name, email, age)
        VALUES ($1, $2, $3)
        RETURNING id, name, email, age, created_at, updated_at
        "#,
        body.name.trim(),
        body.email.trim().to_lowercase(),
        body.age,
    )
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<UserListResponse>, AppError> {
    let limit = params.limit.clamp(1, 100);
    let offset = (params.page.max(1) - 1) * limit;

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?
        .unwrap_or(0);

    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, email, age, created_at, updated_at
        FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(UserListResponse { total, users }))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, email, age, created_at, updated_at
        FROM users WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    let existing = sqlx::query_as!(
        User,
        "SELECT id, name, email, age, created_at, updated_at FROM users WHERE id = $1",
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    let new_name  = body.name.unwrap_or(existing.name);
    let new_email = body.email.unwrap_or(existing.email);
    let new_age   = body.age.unwrap_or(existing.age);

    if new_name.trim().is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".into()));
    }
    if !new_email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".into()));
    }

    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET name = $1, email = $2, age = $3
        WHERE id = $4
        RETURNING id, name, email, age, created_at, updated_at
        "#,
        new_name.trim(),
        new_email.trim().to_lowercase(),
        new_age,
        id,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("User {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}