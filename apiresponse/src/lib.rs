use serde::{Deserialize, Serialize};
use std::fmt::Display;

// Re-export the derive macro
pub use apiresponse_macro::Response;

pub trait Response: Display {
    /// Returns the error code for this error type.
    fn error_code(&self) -> u64;

    /// Returns the error message via the Display implementation.
    fn message(&self) -> String {
        self.to_string()
    }

    /// Returns the HTTP status code for this error type.
    /// Defaults to 200 (OK).
    fn http_status_code(&self) -> u16 {
        200
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub code: u64,
    pub message: String,
    pub data: serde_json::Value,

    #[serde(skip)]
    pub status_code: u16,
}

impl Default for ApiResponse {
    fn default() -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: serde_json::Value::Null,
            status_code: 200,
        }
    }
}

impl ApiResponse {
    /// Create a successful response with data.
    pub fn success(data: impl Serialize) -> Self {
        Self {
            code: 0,
            message: "OK".to_string(),
            data: serde_json::json!(data),
            status_code: 200,
        }
    }

    /// Create a success response without data.
    pub fn ok() -> Self {
        Self::default()
    }

    /// Create an error response from a `Response` implementor.
    pub fn from_error(error: impl Response) -> Self {
        Self {
            code: error.error_code(),
            message: error.message(),
            data: serde_json::Value::Null,
            status_code: error.http_status_code(),
        }
    }
}

impl<T, E> From<Result<T, E>> for ApiResponse
where
    T: Serialize,
    E: Response,
{
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(d) => ApiResponse::success(d),
            Err(e) => ApiResponse::from_error(e),
        }
    }
}

// ============================================================================
// Axum integration
// ============================================================================

#[cfg(feature = "axum")]
mod axum_impl {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};

    impl IntoResponse for ApiResponse {
        fn into_response(self) -> Response {
            let status =
                StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (status, axum::Json(self)).into_response()
        }
    }
}
