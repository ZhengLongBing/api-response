# apiresponse-macro

Derive macro for the `apiresponse` crate. Implements the `Response` trait automatically for your error enums.

## Usage

```rust
use apiresponse::Response;
use thiserror::Error;

#[derive(Debug, Error, Response)]
pub enum MyError {
    #[error("not found")]
    #[response(code = 1001, status = 404)]
    NotFound,

    #[error("unauthorized: {0}")]
    #[response(code = 1002, status = 401)]
    Unauthorized(String),

    #[error(transparent)]
    #[response(transparent)]
    Other(#[from] anyhow::Error),
}
```

## Variant-level Attributes

| Attribute | Required | Description |
|-----------|:--------:|-------------|
| `#[response(code = N)]` | Yes* | Error code for this variant |
| `#[response(status = N)]` | No | HTTP status code (default: 200) |
| `#[response(transparent)]` | — | Delegate to inner error's `Response` impl |

\* Not needed on `transparent` variants — they delegate to the inner error.

## Generated Implementation

The macro generates a `Response` impl with three methods:

- `error_code()` — returns the `code` value (or delegates to inner error for `transparent`)
- `message()` — delegates to `Display` (i.e., your `#[error("...")]` message)
- `http_status_code()` — returns the `status` value (or delegates to inner error for `transparent`)

## Requirements

`apiresponse` >= 0.2.1
- `syn` + `quote` + `proc-macro2` (for proc-macro compilation)

## License

MIT OR Apache-2.0
