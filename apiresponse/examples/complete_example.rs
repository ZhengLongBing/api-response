//! Complete usage example for apiresponse.

use apiresponse::{ApiResponse, Response};
use thiserror::Error;

// ============================================================================
// Example 1: Basic usage with explicit error codes
// ============================================================================

#[derive(Debug, Error, Response)]
pub enum AuthError {
    #[error("User does not exist")]
    #[response(code = 1000)]
    UserNotFound,

    #[error("Incorrect password")]
    #[response(code = 1001)]
    PasswordWrong,

    #[error("Token expired")]
    #[response(code = 1002, status = 401)]
    TokenExpired,

    #[error("Account is locked")]
    #[response(code = 1010, status = 403)]
    AccountLocked,
}

// ============================================================================
// Example 2: Variants with parameters
// ============================================================================

#[derive(Debug, Error, Response)]
pub enum PaymentError {
    #[error("Insufficient balance")]
    #[response(code = 2000)]
    InsufficientBalance,

    #[error("Payment failed: {0}")]
    #[response(code = 2001)]
    PaymentFailed(String),

    #[error("Order does not exist")]
    #[response(code = 2002, status = 404)]
    OrderNotFound,
}

// ============================================================================
// Example 3: Database error
// ============================================================================

#[derive(Debug, Error, Response)]
pub enum DbError {
    #[error("Connection failed")]
    #[response(code = 3000, status = 503)]
    ConnectionFailed,

    #[error("Query timeout")]
    #[response(code = 3001, status = 504)]
    QueryTimeout,

    #[error("Data does not exist")]
    #[response(code = 3002, status = 404)]
    RecordNotFound,
}

// ============================================================================
// Example 4: Transparent error wrapping
// ============================================================================

#[derive(Debug, Error, Response)]
pub enum ApiError {
    #[error("Unknown internal error")]
    #[response(code = 9000, status = 500)]
    Internal,

    // transparent: delegates error_code, message, and http_status_code to inner error
    #[error(transparent)]
    #[response(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    #[response(transparent)]
    Payment(#[from] PaymentError),

    #[error(transparent)]
    #[response(transparent)]
    Database(#[from] DbError),
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("========================================");
    println!("Example 1: Basic error usage");
    println!("========================================\n");

    let error = AuthError::UserNotFound;
    println!("Error: {:?}", error);
    println!("Error code: {}", error.error_code());
    println!("Message: {}", error.message());
    println!("HTTP status code: {}", error.http_status_code());
    println!();

    let locked = AuthError::AccountLocked;
    println!("AccountLocked:");
    println!("  Error code: {}", locked.error_code());
    println!("  Message: {}", locked.message());
    println!("  HTTP status code: {}", locked.http_status_code());
    println!();

    println!("========================================");
    println!("Example 2: Variants with parameters");
    println!("========================================\n");

    let payment_failed = PaymentError::PaymentFailed("Network timeout".to_string());
    println!("Payment failed:");
    println!("  Error code: {}", payment_failed.error_code());
    println!("  Message: {}", payment_failed.message());
    println!();

    println!("========================================");
    println!("Example 3: Transparent error delegation");
    println!("========================================\n");

    // Direct API error
    let api_error1 = ApiError::Internal;
    println!("API error (direct):");
    println!("  Error code: {}", api_error1.error_code());
    println!("  Message: {}", api_error1.message());
    println!();

    // Transparent: wrapping auth error
    let api_error2 = ApiError::Auth(AuthError::PasswordWrong);
    println!("API error (wrapped auth error):");
    println!(
        "  Error code: {} (delegated from AuthError)",
        api_error2.error_code()
    );
    println!(
        "  Message: {} (delegated from AuthError)",
        api_error2.message()
    );
    println!(
        "  HTTP status code: {} (delegated from AuthError)",
        api_error2.http_status_code()
    );
    println!();

    // Transparent: wrapping database error
    let api_error3 = ApiError::Database(DbError::ConnectionFailed);
    println!("API error (wrapped database error):");
    println!("  Error code: {}", api_error3.error_code());
    println!("  Message: {}", api_error3.message());
    println!("  HTTP status code: {}", api_error3.http_status_code());
    println!();

    println!("========================================");
    println!("Example 4: ApiResponse conversion");
    println!("========================================\n");

    // Success response
    let success: ApiResponse = ApiResponse::success("hello world");
    println!("Success:");
    println!("  {}", serde_json::to_string_pretty(&success).unwrap());
    println!();

    // Error response from Result
    let result: Result<String, AuthError> = Err(AuthError::UserNotFound);
    let response: ApiResponse = result.into();
    println!("Error from Result:");
    println!("  {}", serde_json::to_string_pretty(&response).unwrap());
    println!();

    // Error response directly
    let response = ApiResponse::from_error(AuthError::TokenExpired);
    println!("Error directly:");
    println!("  {}", serde_json::to_string_pretty(&response).unwrap());
    println!();

    println!("========================================");
    println!("All examples completed!");
    println!("========================================");
}
