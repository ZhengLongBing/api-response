use serde::{Deserialize, Serialize};
use std::fmt::Display;
use typed_builder::TypedBuilder;
// Re-export the derive macro
pub use apiresponse_macro::Response;

pub trait Response: Display {
    /// Returns the error code for this error type.
    fn error_code(&self) -> u64;

    /// Returns the module path (e.g., "auth.login").
    /// Defaults to empty string if no module is defined.
    fn module_path(&self) -> &'static str {
        ""
    }

    /// Returns the raw error message without module prefix.
    /// Defaults to the Display implementation.
    fn raw_message(&self) -> String {
        self.to_string()
    }

    /// Returns the formatted error message with module prefix.
    /// Format: "[module] message"
    /// If no module path, returns raw message.
    fn message(&self) -> String {
        let module = self.module_path();
        if module.is_empty() {
            self.raw_message()
        } else {
            format!("[{}] {}", module, self.raw_message())
        }
    }

    /// Returns the HTTP status code for this error type.
    /// Defaults to 200 (OK).
    fn http_status_code(&self) -> u16 {
        200
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct ApiResponse {
    #[builder(default = 0)]
    pub code: u64,

    #[builder(default = "OK".to_string())]
    pub message: String,

    #[builder(default = serde_json::Value::default())]
    pub data: serde_json::Value,

    #[serde(skip)]
    #[builder(default = 200)]
    pub status_code: u16,
}

impl Default for ApiResponse {
    fn default() -> Self {
        Self::builder().build()
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
}

impl<T, E> From<Result<T, E>> for ApiResponse
where
    T: Serialize,
    E: Response,
{
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(d) => ApiResponse::success(d),
            Err(e) => ApiResponse {
                code: e.error_code(),
                message: e.message(),
                data: serde_json::Value::default(),
                status_code: e.http_status_code(),
            },
        }
    }
}

// ============================================================================
// Framework-specific implementations
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

#[cfg(feature = "actix")]
mod actix_impl {
    use super::*;
    use actix_web::{HttpRequest, HttpResponse, Responder, body::BoxBody, http::StatusCode};

    impl Responder for ApiResponse {
        type Body = BoxBody;

        fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
            let status =
                StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            HttpResponse::build(status).json(self)
        }
    }
}

#[cfg(feature = "poem")]
mod poem_impl {
    use super::*;
    use poem::http::StatusCode;
    use poem::{IntoResponse, Response};

    impl IntoResponse for ApiResponse {
        fn into_response(self) -> Response {
            let status =
                StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            Response::builder()
                .status(status)
                .body(serde_json::to_string(&self).unwrap_or_default())
        }
    }
}
