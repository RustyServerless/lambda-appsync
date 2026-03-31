use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
#[cfg(feature = "log")]
use syn::LitBool;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitStr, Path, Result, Token,
};

use super::{
    optional_parameter::{OptionalParameter, OptionalParameters, ParameterError, Unknown},
    GraphQLSchema, OverrideParameters,
};

/// A single parsed parameter for the `make_operation!` macro.
pub(super) enum MakeOperationParameter {
    /// A module path prefix applied to all custom types referenced in operations.
    TypeModule(Path),
    /// Whether to log errors via `log::error!` (requires `log` feature, default: `true`).
    #[cfg(feature = "log")]
    ErrorLogging(bool),
}
impl OptionalParameter for MakeOperationParameter {
    fn try_parse_parameter(input: ParseStream) -> core::result::Result<Self, ParameterError> {
        let ident = Self::parse_ident(input)?;
        #[allow(clippy::match_single_binding)]
        match ident.to_string().as_str() {
            "type_module" => Ok(Self::TypeModule(input.parse()?)),
            #[cfg(feature = "log")]
            "error_logging" => Ok(Self::ErrorLogging(input.parse::<LitBool>()?.value())),
            // Unknown option
            _ => ident.unknown(),
        }
    }
}

/// Accumulated parameters for the `make_operation!` macro after parsing.
pub(super) struct MakeOperationParameters {
    pub(super) type_module: Option<Path>,
    #[cfg(feature = "log")]
    pub(super) error_logging: bool,
}
impl Default for MakeOperationParameters {
    fn default() -> Self {
        Self {
            type_module: None,
            #[cfg(feature = "log")]
            error_logging: true,
        }
    }
}
impl OptionalParameters<MakeOperationParameter> for MakeOperationParameters {
    fn set_param(&mut self, p: MakeOperationParameter) {
        match p {
            MakeOperationParameter::TypeModule(path) => self.type_module = Some(path),
            #[cfg(feature = "log")]
            MakeOperationParameter::ErrorLogging(error_logging) => {
                self.error_logging = error_logging
            }
        }
    }
}

/// Parsed input for the `make_operation!` macro, generating only the operation dispatch code.
struct MakeOperation {
    graphql_schema: GraphQLSchema,
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

        let graphql_schema = GraphQLSchema::new(
            graphql_schema_path,
            override_parameters,
            Default::default(),
            parameters,
        )?;

        Ok(Self { graphql_schema })
    }
}

impl ToTokens for MakeOperation {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.graphql_schema.appsync_operations_to_tokens(tokens);
    }
}

/// Entry point for the `make_operation!` proc-macro implementation.
pub(crate) fn make_operation_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let make_operation = parse_macro_input!(input as MakeOperation);
    make_operation.into_token_stream().into()
}
