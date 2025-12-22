# api-response

A Rust library for standardized API error handling with automatic module-based error code management and message formatting.

## Features

- **Automatic Module Prefixing**: Error messages automatically include module paths in `[module] message` format
- **Auto Error Code Numbering**: Sequential error code allocation based on module and base code
- **Transparent Error Delegation**: Proper error propagation through `transparent` variants
- **HTTP Status Code Support**: Attach HTTP status codes to error variants
- **Web Framework Integration**: Built-in support for Axum, Actix-web, and Poem
- **Type-safe**: Compile-time error code and module validation
- **Zero Runtime Cost**: All formatting logic uses efficient trait methods

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
apiresponse = "0.1"
thiserror = "1.0"  # For error definitions
```

For web framework integration:

```toml
[dependencies]
apiresponse = { version = "0.1", features = ["axum"] }  # or "actix", "poem"
```

## Quick Start

### 1. Define Your Error Type

```rust
use api_response::Response;
use thiserror::Error;

#[derive(Debug, Error, Response)]
#[response(module = "auth", base = 1000)]
pub enum AuthError {
    #[error("User not found")]
    #[response(status = 404)]
    UserNotFound,

    #[error("Invalid password")]
    #[response(status = 401)]
    InvalidPassword,
}
```

### 2. Use the Error

```rust
let error = AuthError::UserNotFound;

// Automatic module prefix
println!("{}", error.message());
// Output: [auth] User not found

// Error code (auto-numbered)
println!("{}", error.error_code());
// Output: 1000

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
//   "message": "[auth] User not found",
//   "data": null
// }
```

## Core Features

### 🎯 Automatic Module Prefixing

Error messages automatically include module paths:

```rust
#[derive(Debug, Error, Response)]
#[response(module = "payment.stripe", base = 2000)]
pub enum PaymentError {
    #[error("Insufficient balance")]
    InsufficientBalance,  // Error code: 2000
}

let err = PaymentError::InsufficientBalance;
err.message()      // → "[payment.stripe] Insufficient balance"
err.module_path()  // → "payment.stripe"
err.raw_message()  // → "Insufficient balance"
```

### 🔄 Transparent Error Delegation

Proper error propagation when wrapping errors:

```rust
#[derive(Debug, Error, Response)]
#[response(module = "api")]
pub enum ApiError {
    #[error(transparent)]
    #[response(transparent)]
    Auth(#[from] AuthError),  // Delegates to inner error
}

// When ApiError::Auth(AuthError::UserNotFound) occurs:
let error = ApiError::Auth(AuthError::UserNotFound);
error.message()      // → "[auth] User not found" (from AuthError)
error.module_path()  // → "auth" (from AuthError)
error.error_code()   // → 1000 (from AuthError)
```

### 🔢 Error Code Management

#### Automatic Numbering with `base`

```rust
#[derive(Debug, Error, Response)]
#[response(module = "user", base = 1000)]
pub enum UserError {
    #[error("Not found")]
    NotFound,        // Code: 1000

    #[error("Unauthorized")]
    Unauthorized,    // Code: 1001

    #[error("Forbidden")]
    Forbidden,       // Code: 1002
}
```

#### Manual Code Assignment (No `base`)

```rust
#[derive(Debug, Error, Response)]
#[response(module = "validation")]
pub enum ValidationError {
    #[error("Invalid email")]
    #[response(code = 4001, status = 400)]
    InvalidEmail,

    #[error("Weak password")]
    #[response(code = 4002, status = 400)]
    WeakPassword,
}
```

#### Override Auto-numbering

```rust
#[derive(Debug, Error, Response)]
#[response(module = "auth", base = 1000)]
pub enum AuthError {
    #[error("User not found")]
    NotFound,        // Code: 1000 (auto)

    #[error("Locked")]
    #[response(code = 1099)]  // Override: 1099 instead of 1001
    Locked,
}
```

#### No Module Attribute (Optional)

```rust
#[derive(Debug, Error, Response)]
#[response(base = 6000)]
pub enum SimpleError {
    #[error("Generic error")]
    GenericError,    // Code: 6000, no module prefix

    #[error("Another error")]
    AnotherError,    // Code: 6001, no module prefix
}

// Usage
let err = SimpleError::GenericError;
err.message()      // → "Generic error" (no module prefix)
err.module_path()  // → ""
```

## Common Usage Patterns

### RESTful API Error Handling

```rust
#[derive(Debug, Error, Response)]
#[response(module = "user", base = 1000)]
pub enum UserError {
    #[error("User not found")]
    #[response(status = 404)]
    NotFound,

    #[error("Permission denied")]
    #[response(status = 403)]
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
#[response(module = "app")]
pub enum AppError {
    #[error(transparent)]
    #[response(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    #[response(transparent)]
    Database(#[from] DbError),

    #[error("Internal error")]
    #[response(code = 5000, status = 500)]
    Internal,
}
```

## Web Framework Integration

### Axum

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

### Actix-web

```rust
use actix_web::{get, HttpResponse};
use apiresponse::ApiResponse;

#[get("/user")]
async fn handler() -> ApiResponse {
    // ApiResponse implements Responder
    ApiResponse::from_error(AuthError::UserNotFound)
}
```

### Poem

```rust
use poem::{get, handler, Route};
use apiresponse::ApiResponse;

#[handler]
async fn handler() -> ApiResponse {
    // ApiResponse implements IntoResponse
    ApiResponse::from_error(AuthError::UserNotFound)
}

let app = Route::new().at("/user", get(handler));
```

## Examples

We provide comprehensive examples:

```bash
# Complete feature demonstration
cargo run --example complete_example

# Web framework integration
cargo run --example web_framework_example
```


## API Reference

### Response Trait

```rust
pub trait Response: Display {
    /// Returns the error code
    fn error_code(&self) -> u64;

    /// Returns the module path (e.g., "auth.login")
    fn module_path(&self) -> &'static str;

    /// Returns the raw error message without module prefix
    fn raw_message(&self) -> String;

    /// Returns the formatted message with module prefix: "[module] message"
    fn message(&self) -> String;

    /// Returns the HTTP status code
    fn http_status_code(&self) -> u16;
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
- `ApiResponse::success(data)` - Create success response with data
- `ApiResponse::success_with_message(data, message)` - Success with custom message
- `ApiResponse::ok()` - Create empty success response

### Attributes

#### Enum-level Attributes

```rust
#[response(module = "module_name", base = base_code)]
//        ^^^^^^^^^^^^^^^^^^^       ^^^^^^^^^^^^
//        Optional (default "")     Optional (if omitted, variants must specify code)
```

#### Variant-level Attributes

```rust
#[response(code = error_code, status = http_status, message = "custom_message")]
//        ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^^^^^^^^^
//        Optional (override) Optional (default 200) Optional (override Display)

#[response(transparent)]  // Delegate to inner error
```

## Advanced Features

### Access Metadata

```rust
// Get module path
let module = error.module_path();  // "auth.login"

// Raw vs Formatted messages
error.raw_message()  // → "User not found"
error.message()      // → "[auth.login] User not found"
```

### Multi-level Module Names

```rust
#[response(module = "api.v1.auth")]
pub enum AuthError { /* ... */ }

// Messages will be prefixed with [api.v1.auth]
```

## Design Philosophy

- **Simple by Default**: Minimal boilerplate, maximum functionality
- **Type Safety**: Compile-time validation of error codes and modules
- **Flexibility**: Support both auto-numbering and manual code assignment
- **Zero Cost**: All abstractions compile away to efficient code
- **Framework Agnostic**: Core library has no web framework dependencies

## Requirements

- Rust 1.70 or later
- `thiserror` for error definitions (recommended)
- `serde` and `serde_json` for serialization

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Related Projects

- [thiserror](https://github.com/dtolnay/thiserror) - Excellent error derive macros
- [anyhow](https://github.com/dtolnay/anyhow) - Flexible error handling
