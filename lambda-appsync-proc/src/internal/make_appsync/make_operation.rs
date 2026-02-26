use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitStr, Result, Token,
};

use super::{
    optional_parameter::{OptionalParameter, OptionalParameters, ParameterError, Unknown},
    GraphQLSchema, OverrideParameters,
};

pub(super) enum MakeOperationParameter {}
impl OptionalParameter for MakeOperationParameter {
    fn try_parse_parameter(input: ParseStream) -> core::result::Result<Self, ParameterError> {
        let ident = Self::parse_ident(input)?;
        #[allow(clippy::match_single_binding)]
        match ident.to_string().as_str() {
            // Unknown option
            _ => ident.unknown(),
        }
    }
}

#[derive(Default)]
pub(super) struct MakeOperationParameters {}
impl OptionalParameters<MakeOperationParameter> for MakeOperationParameters {
    fn set_param(&mut self, p: MakeOperationParameter) {
        match p {}
    }
}

struct MakeOperation {
    graphql_schema: GraphQLSchema,
    #[allow(dead_code)]
    parameters: MakeOperationParameters,
}

impl Parse for MakeOperation {
    fn parse(input: ParseStream) -> Result<Self> {
        let graphql_schema_path = input.parse::<LitStr>()?;

        let mut override_parameters = OverrideParameters::default();
        let mut parameters = MakeOperationParameters::default();

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
            // Last try, just forward error if any
            parameters.try_parse_parameter(input)?;
        }

        let graphql_schema = GraphQLSchema::new(graphql_schema_path, override_parameters)?;

        Ok(Self {
            graphql_schema,
            parameters,
        })
    }
}

impl ToTokens for MakeOperation {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.graphql_schema.appsync_operations_to_tokens(tokens);
    }
}

pub(crate) fn make_operation_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let make_operation = parse_macro_input!(input as MakeOperation);
    make_operation.into_token_stream().into()
}
