use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Integration failure with {service}: {message}")]
    Integration { service: String, message: String },

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub timestamp: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::Integration { service, message } => (
                StatusCode::BAD_GATEWAY,
                "INTEGRATION_ERROR",
                format!("{service} error: {message}"),
            ),
            AppError::Database(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                err.to_string(),
            ),
            AppError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                err.to_string(),
            ),
        };

        let body = Json(ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        });

        (status, body).into_response()
    }
}
