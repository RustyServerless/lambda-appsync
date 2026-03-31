use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::{braced, parenthesized, parse::Parse, parse_macro_input, Ident, Token, Type, Visibility};

use super::common::{Name, OperationKind};

/// Optional flags accepted by the `appsync_operation` macro attribute.
enum ArgsOption {
    /// Preserve the original function name in addition to generating the operation method
    #[cfg(feature = "compat")]
    KeepOriginalFunctionName,
    /// Pass the full `AppsyncEvent` as an extra argument to the handler.
    WithAppsyncEvent,
    /// Remove the function and inline its body inside the implemented method on Operation
    InlineAndRemove,
}
impl Parse for ArgsOption {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            #[cfg(feature = "compat")]
            "keep_original_function_name" => Ok(Self::KeepOriginalFunctionName),
            "with_appsync_event" => Ok(Self::WithAppsyncEvent),
            "inline_and_remove" => Ok(Self::InlineAndRemove),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unknown option `{ident}`",),
            )),
        }
    }
}

/// Parsed macro attribute arguments for `appsync_operation`.
struct Args {
    op_kind: OperationKind,
    op_name: Name,
    keep_original_function_name: bool,
    with_appsync_event: bool,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op_kind = input.parse::<Ident>()?;
        let op_kind_s = op_kind.to_string();
        let op_kind = match op_kind_s.as_str() {
            "query" => OperationKind::Query,
            "mutation" => OperationKind::Mutation,
            "subscription" => OperationKind::Subscription,
            _ => {
                return Err(syn::Error::new(
                    op_kind.span(),
                    format!(
                        "Expected one of `query`, `mutation` or `subscription`, got `{op_kind_s}`."
                    ),
                ));
            }
        };
        let op_name;
        _ = parenthesized!(op_name in input);
        let op_name = op_name.parse::<Ident>()?;
        let op_name = Name::from((op_name.to_string(), op_name.span()));

        let mut args = Self {
            op_kind,
            op_name,
            keep_original_function_name: !cfg!(feature = "compat"),
            with_appsync_event: false,
        };

        while input.peek(Token![,]) {
            _ = input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            // We got an option
            let option = input.parse::<ArgsOption>()?;
            match option {
                #[cfg(feature = "compat")]
                ArgsOption::KeepOriginalFunctionName => args.keep_original_function_name = true,
                ArgsOption::WithAppsyncEvent => args.with_appsync_event = true,
                ArgsOption::InlineAndRemove => args.keep_original_function_name = false,
            }
        }
        Ok(args)
    }
}

/// A single parsed function argument, including its optional `mut` binding.
struct FctArg {
    is_mut: bool,
    name: Ident,
    ty: Type,
}
impl Parse for FctArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_mut = if input.peek(Token![mut]) {
            _ = input.parse::<Token![mut]>()?;
            true
        } else {
            false
        };
        let name = input.parse()?;
        _ = input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self { is_mut, name, ty })
    }
}
impl ToTokens for FctArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.is_mut {
            tokens.extend(quote! {mut});
        }
        let name = &self.name;
        let ty = &self.ty;
        tokens.extend(quote! {
            #name: #ty
        });
    }
}
/// A parsed async function that the `appsync_operation` macro is applied to.
struct Fct {
    vis: Option<Visibility>,
    fct_name: Ident,
    args: Vec<FctArg>,
    return_type: Type,
    body: TokenStream2,
}
impl Fct {
    /// Generates a non-async stub of the function used for compile-time signature checking.
    fn dummy_function(&self) -> TokenStream2 {
        let fct_name = &self.fct_name;
        let args = self.args.iter();
        let return_type = &self.return_type;
        quote! {
            #[allow(unused_variables)]
            fn #fct_name(#(#args),*) -> #return_type {
                todo!()
            };
        }
    }
}
impl Parse for Fct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = if input.peek(Token![pub]) {
            Some(input.parse()?)
        } else {
            None
        };

        if !(input.peek(Token![async]) && input.peek2(Token![fn])) {
            return Err(syn::Error::new(
                input.span(),
                "appsync_operation macro must be use on an async function",
            ));
        }
        _ = input.parse::<Token![async]>()?;
        _ = input.parse::<Token![fn]>()?;
        let fct_name = input.parse()?;

        let args_input;
        _ = parenthesized!(args_input in input);
        let mut args = vec![];
        while let Ok(arg) = args_input.parse::<FctArg>() {
            args.push(arg);
            if args_input.peek(Token![,]) {
                _ = args_input.parse::<Token![,]>()?;
            }
        }

        _ = input.parse::<Token![->]>()?;

        let return_type = input.parse()?;

        let body_input;
        _ = braced!(body_input in input);
        let body = body_input.parse()?;

        Ok(Self {
            vis,
            fct_name,
            args,
            return_type,
            body,
        })
    }
}
impl ToTokens for Fct {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let vis = if let Some(ref vis) = self.vis {
            vis.into_token_stream()
        } else {
            TokenStream2::new()
        };

        let fct_name = &self.fct_name;
        let args = self.args.iter();
        let orig_fct_body = &self.body;
        let return_type = &self.return_type;

        tokens.extend(quote! {
            #vis async fn #fct_name(
                #(#args),*
            ) -> #return_type {
                #orig_fct_body
            }
        });
    }
}

/// The fully parsed input to the `appsync_operation` macro, combining its attribute args and the annotated function.
struct AppsyncOperation {
    args: Args,
    fct: Fct,
}
impl AppsyncOperation {
    /// Returns the token stream for the path to the generated operation submodule in `__operations`.
    fn op_module_path(&self) -> TokenStream2 {
        let op_module_name = self.args.op_name.to_var_ident();
        let span = op_module_name.span();
        let op_type_module = Ident::new(self.args.op_kind.module_name(), span);
        let op_submodule_name = if self.args.with_appsync_event {
            Ident::new("with_event", span)
        } else {
            Ident::new("without_event", span)
        };
        quote_spanned! {span=>
            crate::__operations::#op_type_module::#op_module_name::#op_submodule_name
        }
    }
    /// Generates a compile-time assertion that verifies the user function matches the expected operation signature.
    fn check_signature_to_tokens(&self) -> TokenStream2 {
        let op_module_path = self.op_module_path();

        let fct_name = &self.fct.fct_name;
        let dymmy_fct = self.fct.dummy_function();
        quote! {
            const _: fn() = || {
                // Compile-time assertion only – never calls the user fn.
                #dymmy_fct
                #op_module_path::check_signature(#fct_name);
            };
        }
    }

    /// Generates the `impl Operation` method that extracts arguments and dispatches to the user function.
    fn impl_operation_to_tokens(&self) -> TokenStream2 {
        let vis = if let Some(ref vis) = self.fct.vis {
            vis.into_token_stream()
        } else {
            TokenStream2::new()
        };

        let operation_body = if self.args.keep_original_function_name {
            // Call the original fct
            let fct_name = &self.fct.fct_name;
            let arg_names = self.fct.args.iter().map(|a| &a.name);
            &quote! {
                #fct_name(#(#arg_names),*).await
            }
        } else {
            // Inline the original fct body
            &self.fct.body
        };

        let op_module_path = self.op_module_path();

        let op_fct_name = self
            .args
            .op_name
            .to_prefixed_fct_ident(self.args.op_kind.fct_prefix());
        let arg_names = self.fct.args.iter().map(|a| &a.name);
        let return_type = &self.fct.return_type;
        quote! {
            impl crate::Operation {
                #vis async fn #op_fct_name(
                    mut event: ::lambda_appsync::AppsyncEvent<Self>
                ) -> #return_type {
                    let (#(#arg_names,)*) = #op_module_path::operation_arguments(&mut event)?;
                    #operation_body
                }
            }
        }
    }
}
impl TryFrom<(Args, Fct)> for AppsyncOperation {
    type Error = syn::Error;

    fn try_from((args, fct): (Args, Fct)) -> Result<Self, Self::Error> {
        Ok(Self { args, fct })
    }
}
impl ToTokens for AppsyncOperation {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(self.check_signature_to_tokens());
        tokens.extend(self.impl_operation_to_tokens());
        if self.args.keep_original_function_name {
            self.fct.to_tokens(tokens);
        }
    }
}

/// Entry point for the `appsync_operation` proc-macro implementation.
pub(crate) fn appsync_operation_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let fct = parse_macro_input!(input as Fct);
    let appsync_operation = match AppsyncOperation::try_from((args, fct)) {
        Ok(ao) => ao,
        Err(e) => return e.into_compile_error().into(),
    };

    appsync_operation.into_token_stream().into()
}
