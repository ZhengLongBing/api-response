//! Derive macros for the `apiresponse` crate.
//!
//! This crate provides the `#[derive(Response)]` macro which implements
//! the `Response` trait for your error types.
//!
//! ## Usage with thiserror
//!
//! ```rust,ignore
//! use apiresponse::Response;
//! use thiserror::Error;
//!
//! #[derive(Debug, Error, Response)]
//! pub enum MyError {
//!     #[error("not found")]
//!     #[response(code = 1001, status = 404)]
//!     NotFound,
//!
//!     #[error("unauthorized: {0}")]
//!     #[response(code = 1002, status = 401)]
//!     Unauthorized(String),
//!
//!     #[error(transparent)]
//!     #[response(transparent)]
//!     Other(#[from] anyhow::Error),
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, parse_macro_input};

/// Derive macro for implementing the `Response` trait.
///
/// ## Variant-level Attributes
///
/// - `#[response(code = N)]` - Set the error code (required for non-transparent variants)
/// - `#[response(status = N)]` - Set the HTTP status code (optional, defaults to 200)
/// - `#[response(transparent)]` - Delegate to the inner error's `Response` implementation
///
/// ## Example
///
/// ```rust,ignore
/// #[derive(Debug, thiserror::Error, Response)]
/// pub enum AuthError {
///     #[error("User not found")]
///     #[response(code = 1000, status = 404)]
///     UserNotFound,
///
///     #[error("Invalid password")]
///     #[response(code = 1001, status = 401)]
///     InvalidPassword,
///
///     #[error(transparent)]
///     #[response(transparent)]
///     Internal(#[from] anyhow::Error),
/// }
/// ```
#[proc_macro_derive(Response, attributes(response))]
pub fn response_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_response_derive(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_response_derive(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let data = match &input.data {
        Data::Enum(data) => data,
        Data::Struct(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Response can only be derived for enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Response cannot be derived for unions",
            ));
        }
    };

    // Single pass: parse attrs and build pattern once per variant, produce both arms
    let arms: Vec<_> = data
        .variants
        .iter()
        .map(|v| {
            let attrs = parse_response_attrs(&v.attrs)?;
            let pattern = build_pattern(name, &v.ident, &v.fields, attrs.transparent);

            let code_expr = if attrs.transparent {
                quote! { inner.error_code() }
            } else if let Some(code) = attrs.code {
                quote! { #code }
            } else {
                return Err(syn::Error::new_spanned(
                    v,
                    "Variant must specify an error code with `#[response(code = N)]`.\n\
                     Example: #[response(code = 1001)]",
                ));
            };

            let status_expr = if attrs.transparent {
                quote! { inner.http_status_code() }
            } else {
                let status = attrs.status.unwrap_or(200);
                quote! { #status }
            };

            Ok((
                quote! { #pattern => #code_expr },
                quote! { #pattern => #status_expr },
            ))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let (code_arms, status_arms): (Vec<_>, Vec<_>) = arms.into_iter().unzip();

    Ok(quote! {
        impl #impl_generics apiresponse::Response for #name #ty_generics #where_clause {
            fn error_code(&self) -> u64 {
                match self {
                    #(#code_arms),*
                }
            }

            fn message(&self) -> String {
                self.to_string()
            }

            fn http_status_code(&self) -> u16 {
                match self {
                    #(#status_arms),*
                }
            }
        }
    })
}

/// Parse `#[response(...)]` attributes from a variant.
fn parse_response_attrs(attrs: &[Attribute]) -> syn::Result<ResponseAttrs> {
    let mut result = ResponseAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("response") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("code") {
                let value: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit {
                    lit: Lit::Int(lit), ..
                }) = value
                {
                    result.code = Some(lit.base10_parse()?);
                } else {
                    return Err(meta.error("code must be an integer"));
                }
            } else if meta.path.is_ident("status") {
                let value: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit {
                    lit: Lit::Int(lit), ..
                }) = value
                {
                    result.status = Some(lit.base10_parse()?);
                } else {
                    return Err(meta.error("status must be an integer"));
                }
            } else if meta.path.is_ident("transparent") {
                result.transparent = true;
            }
            Ok(())
        })?;
    }

    Ok(result)
}

#[derive(Default)]
struct ResponseAttrs {
    code: Option<u64>,
    status: Option<u16>,
    transparent: bool,
}

/// Build the match arm pattern for a variant.
fn build_pattern(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &Fields,
    transparent: bool,
) -> TokenStream2 {
    match fields {
        Fields::Unit => quote! { #enum_name::#variant_name },
        Fields::Unnamed(fields) => {
            if transparent && fields.unnamed.len() == 1 {
                quote! { #enum_name::#variant_name(inner) }
            } else {
                quote! { #enum_name::#variant_name(..) }
            }
        }
        Fields::Named(_) => {
            if transparent {
                quote! { #enum_name::#variant_name { ref inner, .. } }
            } else {
                quote! { #enum_name::#variant_name { .. } }
            }
        }
    }
}
