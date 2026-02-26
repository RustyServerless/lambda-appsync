//! Crate not intended for direct use.

mod appsync_operation;
mod common;
mod make_appsync;

use proc_macro::TokenStream;

#[doc = include_str!("../doc/make_appsync.md")]
#[proc_macro]
pub fn make_appsync(input: TokenStream) -> TokenStream {
    make_appsync::make_appsync_impl(input)
}

#[doc = include_str!("../doc/make_handlers.md")]
#[proc_macro]
pub fn make_handlers(input: TokenStream) -> TokenStream {
    make_appsync::make_handlers_impl(input)
}

#[doc = include_str!("../doc/make_operation.md")]
#[proc_macro]
pub fn make_operation(input: TokenStream) -> TokenStream {
    make_appsync::make_operation_impl(input)
}

#[doc = include_str!("../doc/make_types.md")]
#[proc_macro]
pub fn make_types(input: TokenStream) -> TokenStream {
    make_appsync::make_types_impl(input)
}

#[doc = include_str!("../doc/appsync_operation.md")]
#[proc_macro_attribute]
pub fn appsync_operation(args: TokenStream, input: TokenStream) -> TokenStream {
    appsync_operation::appsync_operation_impl(args, input)
}

#[cfg(feature = "compat")]
#[doc = include_str!("../doc/appsync_lambda_main.md")]
#[proc_macro]
pub fn appsync_lambda_main(input: TokenStream) -> TokenStream {
    make_appsync::legacy::appsync_lambda_main::appsync_lambda_main_impl(input)
}
