# apiresponse

A Rust library for standardized API error handling with derive macro support.

## Features

- **Derive Macro**: Implement `Response` trait automatically via `#[derive(Response)]`
- **Explicit Error Codes**: Every error variant requires an explicit code for API stability
- **Transparent Error Delegation**: Proper error propagation through `transparent` variants
- **HTTP Status Code Support**: Attach HTTP status codes to error variants
- **Axum Integration**: Built-in support for Axum web framework
- **Type-safe**: Compile-time validation of error codes

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
apiresponse = "0.2.1"
thiserror = "1.0"  # For error definitions
```

For Axum integration:

```toml
[dependencies]
apiresponse = { version = "0.2.1", features = ["axum"] }
```

## Quick Start

### 1. Define Your Error Type

```rust
use apiresponse::Response;
use thiserror::Error;

#[derive(Debug, Error, Response)]
pub enum AuthError {
    #[error("User not found")]
    #[response(code = 1000, status = 404)]
    UserNotFound,

    #[error("Invalid password")]
    #[response(code = 1001, status = 401)]
    InvalidPassword,
}
```

### 2. Use the Error

```rust
let error = AuthError::UserNotFound;

// Error code (from #[response(code = ...)])
println!("{}", error.error_code());
// Output: 1000

// Error message (from #[error(...)]) / Display
println!("{}", error.message());
// Output: User not found

// HTTP status code
println!("{}", error.http_status_code());
// Output: 404
```

### 3. Convert to API Response

```rust
use apiresponse::ApiResponse;

// Method 1: From error
let response = ApiResponse::from_error(error);

// Method 2: From Result
let result: Result<String, AuthError> = Err(AuthError::UserNotFound);
let response: ApiResponse = result.into();

// Serialize to JSON
let json = serde_json::to_string(&response).unwrap();
// {
//   "code": 1000,
//   "message": "User not found",
//   "data": null
// }
```

## Core Features

### 🔢 Explicit Error Codes

Every variant must specify an error code. No implicit numbering means API contracts stay stable:

```rust
#[derive(Debug, Error, Response)]
pub enum PaymentError {
    #[error("Insufficient balance")]
    #[response(code = 2000)]
    InsufficientBalance,

    #[error("Payment failed: {0}")]
    #[response(code = 2001)]
    PaymentFailed(String),

    #[error("Order not found")]
    #[response(code = 2002, status = 404)]
    OrderNotFound,
}
```

### 🔄 Transparent Error Delegation

Delegates `error_code()`, `message()`, and `http_status_code()` to the wrapped inner error:

```rust
#[derive(Debug, Error, Response)]
pub enum AppError {
    #[error("Internal server error")]
    #[response(code = 5000, status = 500)]
    Internal,

    #[error(transparent)]
    #[response(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    #[response(transparent)]
    Database(#[from] DbError),
}

// When AppError::Auth(AuthError::UserNotFound) occurs:
let error = AppError::Auth(AuthError::UserNotFound);
error.error_code()      // → 1000 (from AuthError)
error.message()         // → "User not found" (from AuthError)
error.http_status_code() // → 404 (from AuthError)
```

### 🌐 HTTP Status Codes

Optionally attach HTTP status codes to variants. Defaults to `200`:

```rust
#[derive(Debug, Error, Response)]
pub enum DbError {
    #[error("Connection failed")]
    #[response(code = 3000, status = 503)]
    ConnectionFailed,

    #[error("Data not found")]
    #[response(code = 3001, status = 404)]
    RecordNotFound,
}
```

## Common Usage Patterns

### RESTful API Error Handling

```rust
#[derive(Debug, Error, Response)]
pub enum UserError {
    #[error("User not found")]
    #[response(code = 1000, status = 404)]
    NotFound,

    #[error("Permission denied")]
    #[response(code = 1001, status = 403)]
    PermissionDenied,
}

// In API handler
async fn get_user(id: u64) -> Result<Json<User>, UserError> {
    let user = find_user(id).ok_or(UserError::NotFound)?;
    check_permission(&user).ok_or(UserError::PermissionDenied)?;
    Ok(Json(user))
}
```

### Error Aggregation

```rust
#[derive(Debug, Error, Response)]
pub enum AppError {
    #[error(transparent)]
    #[response(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    #[response(transparent)]
    Database(#[from] DbError),

    #[error("Internal error")]
    #[response(code = 9000, status = 500)]
    Internal,
}
```

## Axum Integration

```rust
use axum::{routing::get, Router};
use apiresponse::ApiResponse;

async fn handler() -> ApiResponse {
    // ApiResponse automatically implements IntoResponse
    // HTTP status code is set from the status_code field
    ApiResponse::from_error(AuthError::UserNotFound)
}

let app = Router::new().route("/user", get(handler));
```

## API Reference

### Response Trait

```rust
pub trait Response: Display {
    /// Returns the error code
    fn error_code(&self) -> u64;

    /// Returns the error message via the Display implementation
    fn message(&self) -> String {
        self.to_string()
    }

    /// Returns the HTTP status code (defaults to 200)
    fn http_status_code(&self) -> u16 {
        200
    }
}
```

### ApiResponse Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub code: u64,
    pub message: String,
    pub data: serde_json::Value,
    #[serde(skip)]
    pub status_code: u16,
}
```

Methods:
- `ApiResponse::success(data)` — Create success response with data
- `ApiResponse::ok()` — Create empty success response
- `ApiResponse::from_error(error)` — Create error response from a `Response` implementor

### Attributes

#### Variant-level Attributes

```rust
#[response(code = error_code, status = http_status)]
//        ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
//        Required             Optional (default 200)

#[response(transparent)]  // Delegate to inner error (code and status not needed)
```

## Design Philosophy

- **Explicit over Implicit**: Error codes must be explicitly assigned for API stability
- **Minimal by Default**: Only three trait methods — `error_code()`, `message()`, `http_status_code()`
- **Type Safety**: Compile-time validation ensures every variant has an error code
- **Framework Agnostic**: Core library has no web framework dependencies; Axum is optional

## Requirements

- Rust 1.70 or later
- `thiserror` for error definitions (recommended)
- `serde` and `serde_json` for serialization

## License

MIT OR Apache-2.0

## Related Projects

- [thiserror](https://github.com/dtolnay/thiserror) — Error derive macros
- [anyhow](https://github.com/dtolnay/anyhow) — Flexible error handling
