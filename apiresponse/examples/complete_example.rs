// Complete usage example: Demonstrating module path formatting and documentation generation

use apiresponse::{ApiResponse, Response};
use thiserror::Error;

// ============================================================================
// Example 1: Basic usage - automatic module prefix
// ============================================================================

#[derive(Debug, Error, Response)]
#[response(module = "auth.login", base = 1000)]
pub enum AuthError {
    #[error("User does not exist")]
    UserNotFound,

    #[error("Incorrect password")]
    PasswordWrong,

    #[error("Token expired")]
    #[response(status = 401)]
    TokenExpired,

    #[error("Account is locked")]
    #[response(code = 1010, status = 403)] // Explicitly specified error code
    AccountLocked,
}

// ============================================================================
// Example 2: Two-level module name
// ============================================================================

#[derive(Debug, Error, Response)]
#[response(module = "payment.alipay", base = 2000)]
pub enum PaymentError {
    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Order does not exist")]
    #[response(status = 404)]
    OrderNotFound,
}

// ============================================================================
// Example 3: Database error
// ============================================================================

#[derive(Debug, Error, Response)]
#[response(module = "database", base = 3000)]
pub enum DbError {
    #[error("Connection failed")]
    #[response(status = 503)]
    ConnectionFailed,

    #[error("Query timeout")]
    #[response(status = 504)]
    QueryTimeout,

    #[error("Data does not exist")]
    #[response(status = 404)]
    RecordNotFound,
}

// ============================================================================
// Example 4: Transparent error wrapping - demonstrating recursive handling
// ============================================================================

#[derive(Debug, Error, Response)]
#[response(module = "api")]
pub enum ApiError {
    #[error("Authentication failed")]
    #[response(code = 4001, status = 401)]
    AuthenticationFailed,

    // transparent variant: delegates to inner error
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
// Example 5: No base attribute - code must be explicitly specified
// ============================================================================

#[derive(Debug, Error, Response)]
#[response(module = "validation")]
pub enum ValidationError {
    #[error("Invalid email format")]
    #[response(code = 5001, status = 400)]
    InvalidEmail,

    #[error("Insufficient password strength")]
    #[response(code = 5002, status = 400)]
    WeakPassword,

    #[error("Invalid phone number format")]
    #[response(code = 5003, status = 400)]
    InvalidPhone,
}

// ============================================================================
// Example 6: No module attribute - module path defaults to empty string
// ============================================================================

#[derive(Debug, Error, Response)]
#[response(base = 6000)]
pub enum SimpleError {
    #[error("Generic error")]
    GenericError,

    #[error("Another error")]
    #[response(status = 500)]
    AnotherError,
}

// ============================================================================
// Main function: Demonstrates all features
// ============================================================================

fn main() {
    println!("========================================");
    println!("Example 1: Basic module prefix formatting");
    println!("========================================\n");

    let error = AuthError::UserNotFound;
    println!("Error: {:?}", error);
    println!("Error code: {}", error.error_code());
    println!("Module path: {}", error.module_path());
    println!("Raw message: {}", error.raw_message());
    println!("Formatted message: {}", error.message());
    println!("HTTP status code: {}", error.http_status_code());
    println!();

    // Variant with explicitly specified error code
    let locked = AuthError::AccountLocked;
    println!("Locked account error:");
    println!(
        "  Error code: {} (explicitly specified)",
        locked.error_code()
    );
    println!("  Message: {}", locked.message());
    println!("  HTTP status code: {}", locked.http_status_code());
    println!();

    println!("========================================");
    println!("Example 2: Two-level module name");
    println!("========================================\n");

    let payment_error = PaymentError::InsufficientBalance;
    println!("Payment error:");
    println!("  Module: {}", payment_error.module_path());
    println!("  Message: {}", payment_error.message());
    println!();

    let payment_failed = PaymentError::PaymentFailed("Network timeout".to_string());
    println!("Payment failed (with parameter):");
    println!("  Message: {}", payment_failed.message());
    println!();

    println!("========================================");
    println!("Example 3: Transparent recursive handling");
    println!("========================================\n");

    // Direct API error
    let api_error1 = ApiError::AuthenticationFailed;
    println!("API error (direct):");
    println!("  Module: {}", api_error1.module_path());
    println!("  Message: {}", api_error1.message());
    println!("  Error code: {}", api_error1.error_code());
    println!();

    // Transparent: wrapping authentication error
    let api_error2 = ApiError::Auth(AuthError::PasswordWrong);
    println!("API error (wrapped auth error):");
    println!("  Module: {} (recursed to inner)", api_error2.module_path());
    println!("  Message: {} (recursed to inner)", api_error2.message());
    println!("  Error code: {} (from AuthError)", api_error2.error_code());
    println!();

    // Transparent: wrapping database error
    let api_error3 = ApiError::Database(DbError::ConnectionFailed);
    println!("API error (wrapped database error):");
    println!("  Module: {} (recursed to inner)", api_error3.module_path());
    println!("  Message: {} (recursed to inner)", api_error3.message());
    println!("  Error code: {}", api_error3.error_code());
    println!("  HTTP status code: {}", api_error3.http_status_code());
    println!();

    println!("========================================");
    println!("Example 4: Integration with ApiResponse");
    println!("========================================\n");

    // Automatic conversion using Result
    let result: Result<String, AuthError> = Err(AuthError::UserNotFound);
    let response2: ApiResponse = result.into();
    println!("Converting ApiResponse from Result:");
    println!("  {}", serde_json::to_string_pretty(&response2).unwrap());
    println!();

    println!("========================================");
    println!("Example 5: Usage without base attribute");
    println!("========================================\n");

    let validation_error = ValidationError::InvalidEmail;
    println!("Validation error (no base, code explicitly specified):");
    println!("  Error code: {}", validation_error.error_code());
    println!("  Message: {}", validation_error.message());
    println!(
        "  HTTP status code: {}",
        validation_error.http_status_code()
    );
    println!();

    println!("========================================");
    println!("Example 6: Usage without module attribute");
    println!("========================================\n");

    let simple_error = SimpleError::GenericError;
    println!("Simple error (no module, empty prefix):");
    println!("  Error code: {}", simple_error.error_code());
    println!("  Module path: \"{}\"", simple_error.module_path());
    println!("  Raw message: {}", simple_error.raw_message());
    println!("  Formatted message: {}", simple_error.message());
    println!("  Note: No module prefix since module is empty");
    println!();

    println!("========================================");
    println!("All examples completed!");
    println!("========================================");
}
