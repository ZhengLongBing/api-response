//! Derive macros for the `api-response` crate.
//!
//! This crate provides the `#[derive(Response)]` macro which implements
//! the `Response` trait for your error types.
//!
//! ## Usage with thiserror
//!
//! ```rust,ignore
//! use api_response::Response;
//! use thiserror::Error;
//!
//! #[derive(Debug, Error, Response)]
//! pub enum MyError {
//!     #[error("not found")]
//!     #[response(code = 404)]
//!     NotFound,
//!
//!     #[error("unauthorized: {0}")]
//!     #[response(code = 401)]
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
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, Variant, parse_macro_input,
};

/// Derive macro for implementing the `Response` trait.
///
/// ## Enum-level Attributes
///
/// - `#[response(module = "...")]` - Set the module name (optional, defaults to empty string)
/// - `#[response(base = N)]` - Set the base error code for auto-numbering (optional)
///
/// ## Variant-level Attributes
///
/// - `#[response(code = N)]` - Set the error code for this variant (required if no base)
/// - `#[response(status = N)]` - Set the HTTP status code for this variant (optional, defaults to 200)
/// - `#[response(message = "...")]` - Override the error message (optional, defaults to Display)
/// - `#[response(transparent)]` - Delegate to the inner error's `Response` implementation
///
/// ## Example
///
/// ```rust,ignore
/// #[derive(Debug, thiserror::Error, Response)]
/// #[response(module = "auth", base = 1000)]
/// pub enum AuthError {
///     #[error("User not found")]
///     UserNotFound,  // Code: 1000
///
///     #[error("Invalid password")]
///     #[response(status = 401)]
///     InvalidPassword,  // Code: 1001
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

    let enum_attrs = parse_enum_attrs(&input.attrs)?;

    let code_arms: Vec<_> = data
        .variants
        .iter()
        .enumerate()
        .map(|(index, v)| generate_code_arm(name, v, &enum_attrs, index))
        .collect::<syn::Result<_>>()?;

    let module_str = enum_attrs.module.as_deref().unwrap_or("");

    let raw_message_arms: Vec<_> = data
        .variants
        .iter()
        .map(|v| generate_raw_message_arm(name, v))
        .collect::<syn::Result<_>>()?;

    let module_path_arms: Vec<_> = data
        .variants
        .iter()
        .map(|v| generate_module_path_arm(name, v, module_str))
        .collect::<syn::Result<_>>()?;

    let status_arms: Vec<_> = data
        .variants
        .iter()
        .map(|v| generate_status_arm(name, v))
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        impl #impl_generics apiresponse::Response for #name #ty_generics #where_clause {
            fn error_code(&self) -> u64 {
                match self {
                    #(#code_arms),*
                }
            }

            fn module_path(&self) -> &'static str {
                match self {
                    #(#module_path_arms),*
                }
            }

            fn raw_message(&self) -> String {
                match self {
                    #(#raw_message_arms),*
                }
            }

            fn http_status_code(&self) -> u16 {
                match self {
                    #(#status_arms),*
                }
            }
        }
    })
}

/// Parse #[response(...)] attributes from a variant
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
            } else if meta.path.is_ident("message") {
                let value: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit), ..
                }) = value
                {
                    result.message = Some(lit.value());
                } else {
                    return Err(meta.error("message must be a string"));
                }
            } else if meta.path.is_ident("transparent") {
                result.transparent = true;
            }
            Ok(())
        })?;
    }

    Ok(result)
}

/// Parse #[response(...)] attributes from the enum itself
fn parse_enum_attrs(attrs: &[Attribute]) -> syn::Result<EnumAttrs> {
    let mut result = EnumAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("response") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("module") {
                let value: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit), ..
                }) = value
                {
                    result.module = Some(lit.value());
                } else {
                    return Err(meta.error("module must be a string"));
                }
            } else if meta.path.is_ident("base") {
                let value: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit {
                    lit: Lit::Int(lit), ..
                }) = value
                {
                    result.base = Some(lit.base10_parse()?);
                } else {
                    return Err(meta.error("base must be an integer"));
                }
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
    message: Option<String>,
    transparent: bool,
}

/// Enum-level attributes parsed from #[response(...)] on the enum itself
#[derive(Default)]
struct EnumAttrs {
    /// Module name (e.g., "auth" or "auth.login")
    module: Option<String>,
    /// Base error code for auto-numbering
    base: Option<u64>,
}

fn generate_code_arm(
    enum_name: &Ident,
    variant: &Variant,
    enum_attrs: &EnumAttrs,
    index: usize,
) -> syn::Result<TokenStream2> {
    let variant_name = &variant.ident;
    let attrs = parse_response_attrs(&variant.attrs)?;

    let pattern = match &variant.fields {
        Fields::Unit => quote! { #enum_name::#variant_name },
        Fields::Unnamed(fields) => {
            if attrs.transparent && fields.unnamed.len() == 1 {
                quote! { #enum_name::#variant_name(inner) }
            } else {
                quote! { #enum_name::#variant_name(..) }
            }
        }
        Fields::Named(_) => {
            if attrs.transparent {
                // Find the first field for transparent delegation
                quote! { #enum_name::#variant_name { ref inner, .. } }
            } else {
                quote! { #enum_name::#variant_name { .. } }
            }
        }
    };

    let code_expr = if attrs.transparent {
        quote! { inner.error_code() }
    } else {
        // 优先级: 变体 code > enum base + index
        let code = if let Some(explicit_code) = attrs.code {
            explicit_code
        } else if let Some(base) = enum_attrs.base {
            base + index as u64
        } else {
            // 没有 base 且变体没有显式 code，报错
            return Err(syn::Error::new_spanned(
                variant,
                "Variant must explicitly specify 'code' when enum does not have 'base' attribute.\n\
                 Example: #[response(code = 1001)]",
            ));
        };
        quote! { #code }
    };

    Ok(quote! { #pattern => #code_expr })
}

fn generate_raw_message_arm(enum_name: &Ident, variant: &Variant) -> syn::Result<TokenStream2> {
    let variant_name = &variant.ident;
    let attrs = parse_response_attrs(&variant.attrs)?;

    let pattern = match &variant.fields {
        Fields::Unit => quote! { #enum_name::#variant_name },
        Fields::Unnamed(fields) => {
            if attrs.transparent && fields.unnamed.len() == 1 {
                quote! { #enum_name::#variant_name(inner) }
            } else {
                quote! { #enum_name::#variant_name(..) }
            }
        }
        Fields::Named(_) => {
            if attrs.transparent {
                quote! { #enum_name::#variant_name { ref inner, .. } }
            } else {
                quote! { #enum_name::#variant_name { .. } }
            }
        }
    };

    let message_expr = if attrs.transparent {
        quote! { inner.raw_message() }
    } else if let Some(msg) = &attrs.message {
        quote! { #msg.to_string() }
    } else {
        // Use Display implementation (from thiserror)
        quote! { self.to_string() }
    };

    Ok(quote! { #pattern => #message_expr })
}

fn generate_status_arm(enum_name: &Ident, variant: &Variant) -> syn::Result<TokenStream2> {
    let variant_name = &variant.ident;
    let attrs = parse_response_attrs(&variant.attrs)?;

    let pattern = match &variant.fields {
        Fields::Unit => quote! { #enum_name::#variant_name },
        Fields::Unnamed(fields) => {
            if attrs.transparent && fields.unnamed.len() == 1 {
                quote! { #enum_name::#variant_name(inner) }
            } else {
                quote! { #enum_name::#variant_name(..) }
            }
        }
        Fields::Named(_) => {
            if attrs.transparent {
                quote! { #enum_name::#variant_name { ref inner, .. } }
            } else {
                quote! { #enum_name::#variant_name { .. } }
            }
        }
    };

    let status_expr = if attrs.transparent {
        quote! { inner.http_status_code() }
    } else {
        let status = attrs.status.unwrap_or(200);
        quote! { #status }
    };

    Ok(quote! { #pattern => #status_expr })
}

fn generate_module_path_arm(
    enum_name: &Ident,
    variant: &Variant,
    module_str: &str,
) -> syn::Result<TokenStream2> {
    let variant_name = &variant.ident;
    let attrs = parse_response_attrs(&variant.attrs)?;

    let pattern = match &variant.fields {
        Fields::Unit => quote! { #enum_name::#variant_name },
        Fields::Unnamed(fields) => {
            if attrs.transparent && fields.unnamed.len() == 1 {
                quote! { #enum_name::#variant_name(inner) }
            } else {
                quote! { #enum_name::#variant_name(..) }
            }
        }
        Fields::Named(_) => {
            if attrs.transparent {
                quote! { #enum_name::#variant_name { ref inner, .. } }
            } else {
                quote! { #enum_name::#variant_name { .. } }
            }
        }
    };

    let module_expr = if attrs.transparent {
        quote! { inner.module_path() }
    } else {
        quote! { #module_str }
    };

    Ok(quote! { #pattern => #module_expr })
}
