use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ApiError {
    #[error("resource not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("internal server error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", self.to_string()),
            ApiError::Database(e) => {
                tracing::error!("database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "a database error occurred".to_string(),
                )
            }
            ApiError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                self.to_string(),
            ),
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    fn status_of(e: ApiError) -> StatusCode {
        let response = e.into_response();
        response.status()
    }

    #[test]
    fn not_found_returns_404() {
        assert_eq!(status_of(ApiError::NotFound), StatusCode::NOT_FOUND);
    }

    #[test]
    fn unauthorized_returns_401() {
        assert_eq!(status_of(ApiError::Unauthorized), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn bad_request_returns_400() {
        assert_eq!(
            status_of(ApiError::BadRequest("missing field".to_string())),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn internal_returns_500() {
        assert_eq!(
            status_of(ApiError::Internal),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
