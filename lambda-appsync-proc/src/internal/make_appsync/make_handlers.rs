use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitBool, Result, Token, Type,
};

use super::optional_parameter::{OptionalParameter, OptionalParameters, ParameterError, Unknown};

pub(super) enum MakeHandlersParameter {
    Batch(bool),
    OperationType(Type),
}
impl OptionalParameter for MakeHandlersParameter {
    fn try_parse_parameter(
        input: syn::parse::ParseStream,
    ) -> core::result::Result<Self, ParameterError> {
        let ident = Self::parse_ident(input)?;
        match ident.to_string().as_str() {
            "batch" => Ok(Self::Batch(input.parse::<LitBool>()?.value())),
            "operation_type" => Ok(Self::OperationType(input.parse()?)),
            // Unknown option
            _ => ident.unknown(),
        }
    }
}

pub(super) struct MakeHandlersParameters {
    batch: bool,
    operation_type: Option<Type>,
}
impl Default for MakeHandlersParameters {
    fn default() -> Self {
        Self {
            batch: true,
            operation_type: None,
        }
    }
}
impl OptionalParameters<MakeHandlersParameter> for MakeHandlersParameters {
    fn set_param(&mut self, p: MakeHandlersParameter) {
        match p {
            MakeHandlersParameter::Batch(batch) => self.batch = batch,
            MakeHandlersParameter::OperationType(t) => self.operation_type = Some(t),
        }
    }
}

pub(super) struct MakeHandlers {
    batch: bool,
    operation_type: TokenStream2,
}

impl From<MakeHandlersParameters> for MakeHandlers {
    fn from(value: MakeHandlersParameters) -> Self {
        let MakeHandlersParameters {
            batch,
            operation_type,
        } = value;
        let operation_type = operation_type
            .map(|t| t.into_token_stream())
            .unwrap_or_else(|| Ident::new("Operation", Span::call_site()).into_token_stream());
        Self {
            batch,
            operation_type,
        }
    }
}

impl Parse for MakeHandlers {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut parameters = MakeHandlersParameters::default();

        while input.peek(Token![,]) {
            _ = input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            parameters.try_parse_parameter(input)?
        }

        Ok(parameters.into())
    }
}

impl ToTokens for MakeHandlers {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let operation = &self.operation_type;
        let mut batch_handler = TokenStream2::new();
        if self.batch {
            batch_handler.extend(quote! {
                #[doc = "Handles a batch of [lambda_appsync::AppsyncEvent<Operation>] concurrently."]
                async fn appsync_batch_handler(
                    events: Vec<::lambda_appsync::AppsyncEvent<#operation>>
                ) -> Vec<::lambda_appsync::AppsyncResponse> {
                    let handles = events
                        .into_iter()
                        .map(|e| ::lambda_appsync::tokio::spawn(Self::appsync_handler(e)))
                        .collect::<Vec<_>>();

                    let mut results = vec![];
                    for h in handles {
                        results.push(h.await.unwrap())
                    }
                    results
                }
            });
        }

        let (service_fn_call, ret_type) = if self.batch {
            (
                format_ident!("appsync_batch_handler"),
                quote! {Vec<::lambda_appsync::AppsyncResponse>},
            )
        } else {
            (
                format_ident!("appsync_handler"),
                quote! {::lambda_appsync::AppsyncResponse},
            )
        };

        tokens.extend(quote! {

            #[deny(dead_code)]
            trait Handlers {
                #[doc = "Handles a single deserialized [lambda_appsync::AppsyncEvent<Operation>]."]
                async fn appsync_handler(
                    event: ::lambda_appsync::AppsyncEvent<#operation>
                ) -> ::lambda_appsync::AppsyncResponse {
                    event.info.operation.execute(event).await
                }

                #batch_handler

                #[doc = "Top-level Lambda handler used as the service_fn of the Runtime."]
                async fn service_fn(
                    event: ::lambda_appsync::lambda_runtime::LambdaEvent<::lambda_appsync::serde_json::Value>
                ) -> ::core::result::Result<#ret_type, ::lambda_appsync::lambda_runtime::Error> {
                    Ok(Self::#service_fn_call(::lambda_appsync::serde_json::from_value(event.payload)?).await)
                }
            }

            struct DefaultHandlers;
            impl Handlers for DefaultHandlers {}

        });
    }
}

pub(crate) fn make_handlers_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let make_handlers = parse_macro_input!(input as MakeHandlers);
    make_handlers.into_token_stream().into()
}
