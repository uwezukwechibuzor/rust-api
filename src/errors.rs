use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => match e {
                sqlx::Error::RowNotFound => (
                    StatusCode::NOT_FOUND,
                    "Record not found".to_string(),
                ),
                sqlx::Error::Database(db_err) => {
                    if db_err.constraint().is_some() {
                        (StatusCode::CONFLICT, "Email already exists".to_string())
                    } else {
                        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                    }
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            },
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}