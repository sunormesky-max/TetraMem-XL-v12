use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("memory error: {0}")]
    Memory(#[from] crate::universe::memory::MemoryError),

    #[error("energy error: {0}")]
    Energy(#[from] crate::universe::energy::EnergyError),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("too many requests")]
    TooManyRequests,

    #[error("internal error: {0}")]
    Internal(String),

    #[error("config error: {0}")]
    Config(#[from] crate::universe::config::ConfigError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    success: bool,
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Memory(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Energy(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::TooManyRequests => {
                (StatusCode::TOO_MANY_REQUESTS, "too many requests".to_string())
            }
            AppError::Internal(msg) => {
                tracing::error!("internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            }
            AppError::Config(e) => {
                tracing::error!("config error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            }
            AppError::Io(e) => {
                tracing::error!("io error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            }
            AppError::Serialize(e) => {
                tracing::error!("serialization error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            }
        };

        let body = ErrorBody {
            success: false,
            error: message,
            detail: None,
        };

        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_error_conversion() {
        let me = crate::universe::memory::MemoryError::EmptyData;
        let app: AppError = me.into();
        assert!(matches!(app, AppError::Memory(_)));
    }

    #[test]
    fn not_found_display() {
        let e = AppError::NotFound("test".to_string());
        assert!(e.to_string().contains("test"));
    }
}
