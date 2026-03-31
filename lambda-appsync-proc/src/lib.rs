//! Procedural macros for the [`lambda-appsync`](https://docs.rs/lambda-appsync) type-safe AWS
//! AppSync resolver framework.
//!
//! **This crate is not intended for direct use.** Depend on [`lambda-appsync`](https://docs.rs/lambda-appsync)
//! instead — it re-exports all macros from this crate under a stable, unified API.
//!
//! # Provided Macros
//!
//! | Macro | Kind | Purpose |
//! |---|---|---|
//! | [`make_appsync!`] | function-like | Generate the full AppSync resolver scaffolding from a GraphQL schema |
//! | [`make_handlers!`] | function-like | Generate handler dispatch glue for a set of operations |
//! | [`make_operation!`] | function-like | Generate a single typed operation from a GraphQL field definition |
//! | [`make_types!`] | function-like | Generate Rust types from GraphQL type definitions |
//! | [`macro@appsync_operation`] | attribute | Annotate an async handler function as an AppSync operation resolver |
//! | [`appsync_lambda_main!`] | function-like | *(feature: `compat`)* Generate a `main` entry point compatible with the legacy single-resolver pattern |
//!
//! Each macro has detailed documentation on its own page, including syntax, options, and examples.
//!
//! # Feature Flags
//!
//! - **`compat`** — Enables [`appsync_lambda_main!`] for backwards-compatible single-resolver Lambda entry points
//! - **`log`** — Enables `log`-based logging support in generated code
//! - **`env_logger`** — Enables `env_logger` initialisation in generated entry points
//! - **`tracing`** — Enables `tracing`-based instrumentation in generated code

mod internal;

use proc_macro::TokenStream;

#[doc = include_str!("../doc/make_appsync.md")]
#[proc_macro]
pub fn make_appsync(input: TokenStream) -> TokenStream {
    internal::make_appsync::make_appsync_impl(input)
}

#[doc = include_str!("../doc/make_handlers.md")]
#[proc_macro]
pub fn make_handlers(input: TokenStream) -> TokenStream {
    internal::make_appsync::make_handlers_impl(input)
}

#[doc = include_str!("../doc/make_operation.md")]
#[proc_macro]
pub fn make_operation(input: TokenStream) -> TokenStream {
    internal::make_appsync::make_operation_impl(input)
}

#[doc = include_str!("../doc/make_types.md")]
#[proc_macro]
pub fn make_types(input: TokenStream) -> TokenStream {
    internal::make_appsync::make_types_impl(input)
}

#[doc = include_str!("../doc/appsync_operation.md")]
#[proc_macro_attribute]
pub fn appsync_operation(args: TokenStream, input: TokenStream) -> TokenStream {
    internal::appsync_operation::appsync_operation_impl(args, input)
}

#[cfg(feature = "compat")]
#[cfg_attr(feature = "compat", doc = include_str!("../doc/appsync_lambda_main.md"))]
#[proc_macro]
pub fn appsync_lambda_main(input: TokenStream) -> TokenStream {
    internal::make_appsync::legacy::appsync_lambda_main::appsync_lambda_main_impl(input)
}
