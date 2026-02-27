mod graphql;
mod make_handlers;
mod make_operation;
mod make_types;
mod optional_parameter;
mod overrides;

#[cfg(feature = "compat")]
pub(crate) mod legacy;

use graphql::GraphQLSchema;
use make_handlers::{MakeHandlers, MakeHandlersParameters};
use make_operation::MakeOperationParameters;
use make_types::MakeTypesParameters;
use optional_parameter::{OptionalParameters, ParameterError};
use overrides::OverrideParameters;

pub(crate) use make_handlers::make_handlers_impl;
pub(crate) use make_operation::make_operation_impl;
pub(crate) use make_types::make_types_impl;

use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitStr, Result, Token,
};

struct MakeAppsync {
    graphql_schema: GraphQLSchema,
    make_handlers: MakeHandlers,
}

impl Parse for MakeAppsync {
    fn parse(input: ParseStream) -> Result<Self> {
        let graphql_schema_path = input.parse::<LitStr>()?;

        let mut override_parameters = OverrideParameters::default();
        let mut make_types_parameters = MakeTypesParameters::default();
        let mut make_operation_parameters = MakeOperationParameters::default();
        let mut make_handlers_parameters = MakeHandlersParameters::default();

        while input.peek(Token![,]) {
            _ = input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            match override_parameters.try_parse_parameter(input) {
                // Matched, so go on to the next one
                Ok(()) => continue,
                Err(pe) => match pe {
                    // Inexistant error, try the next parameter types
                    ParameterError::InexistantParameter(_) => {}
                    // Hard errors, return
                    ParameterError::NotParameter(error) | ParameterError::ArgumentError(error) => {
                        return Err(error)
                    }
                },
            }
            match make_types_parameters.try_parse_parameter(input) {
                // Matched, so go on to the next one
                Ok(()) => continue,
                Err(pe) => match pe {
                    // Inexistant error, try the next parameter types
                    ParameterError::InexistantParameter(_) => {}
                    // Hard errors, return
                    ParameterError::NotParameter(error) | ParameterError::ArgumentError(error) => {
                        return Err(error)
                    }
                },
            }
            match make_operation_parameters.try_parse_parameter(input) {
                // Matched, so go on to the next one
                Ok(()) => continue,
                Err(pe) => match pe {
                    // Inexistant error, try the next parameter types
                    ParameterError::InexistantParameter(_) => {}
                    // Hard errors, return
                    ParameterError::NotParameter(error) | ParameterError::ArgumentError(error) => {
                        return Err(error)
                    }
                },
            }
            // Last try, just forward error if any
            make_handlers_parameters.try_parse_parameter(input)?;
        }

        let graphql_schema = GraphQLSchema::new(graphql_schema_path, override_parameters, None)?;

        Ok(Self {
            graphql_schema,
            make_handlers: make_handlers_parameters.into(),
        })
    }
}

impl ToTokens for MakeAppsync {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.graphql_schema.appsync_types_to_tokens(tokens);

        self.graphql_schema.appsync_operations_to_tokens(tokens);

        self.make_handlers.to_tokens(tokens);
    }
}

pub(crate) fn make_appsync_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let make_appsync = parse_macro_input!(input as MakeAppsync);
    make_appsync.into_token_stream().into()
}
