use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitBool, LitStr, Path, Result, Token,
};

use super::{
    optional_parameter::{OptionalParameter, OptionalParameters, ParameterError, Unknown},
    GraphQLSchema, OverrideParameters,
};

pub(super) struct DeriveAddition {
    type_name: Ident,
    trait_derive_macro: Path,
}
impl Parse for DeriveAddition {
    fn parse(input: ParseStream) -> Result<Self> {
        let type_name = input.call(Ident::parse_any)?;
        _ = input.parse::<syn::Token![:]>()?;
        let trait_derive_macro = input
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "Expected a trait Derive macro"))?;
        Ok(Self {
            type_name,
            trait_derive_macro,
        })
    }
}

pub(super) struct NoDefaultDeriveModifier {
    type_name: Ident,
    default_traits: bool,
}
impl Parse for NoDefaultDeriveModifier {
    fn parse(input: ParseStream) -> Result<Self> {
        let type_name = input.call(Ident::parse_any)?;
        _ = input.parse::<syn::Token![:]>()?;
        let default_traits = input.parse::<LitBool>()?.value;
        Ok(Self {
            type_name,
            default_traits,
        })
    }
}

/// A single parsed parameter for the `make_types!` macro.
///
/// Currently empty — reserved for future parameters.
pub(super) enum MakeTypesParameter {
    Derive(DeriveAddition),
    DefaultDerivations(NoDefaultDeriveModifier),
}
impl OptionalParameter for MakeTypesParameter {
    fn try_parse_parameter(input: ParseStream) -> core::result::Result<Self, ParameterError> {
        let ident = Self::parse_ident(input)?;
        #[allow(clippy::match_single_binding)]
        match ident.to_string().as_str() {
            "derive" => Ok(Self::Derive(input.parse()?)),
            "default_traits" => Ok(Self::DefaultDerivations(input.parse()?)),
            // Unknown option
            _ => ident.unknown(),
        }
    }
}

/// Accumulated parameters for the `make_types!` macro after parsing.
#[derive(Default)]
pub(super) struct MakeTypesParameters {
    pub(super) add_derivations: HashMap<String, Vec<Path>>,
    pub(super) no_default_traits: HashSet<String>,
}

impl OptionalParameters<MakeTypesParameter> for MakeTypesParameters {
    fn set_param(&mut self, parameter: MakeTypesParameter) {
        match parameter {
            MakeTypesParameter::Derive(DeriveAddition {
                type_name,
                trait_derive_macro,
            }) => {
                self.add_derivations
                    .entry(type_name.to_string())
                    .or_default()
                    .push(trait_derive_macro);
            }
            MakeTypesParameter::DefaultDerivations(NoDefaultDeriveModifier {
                type_name,
                default_traits,
            }) => {
                if !default_traits {
                    self.no_default_traits.insert(type_name.to_string());
                }
            }
        }
    }
}

/// Parsed input for the `make_types!` macro, generating only the GraphQL struct and enum types.
struct MakeTypes {
    graphql_schema: GraphQLSchema,
}

impl Parse for MakeTypes {
    fn parse(input: ParseStream) -> Result<Self> {
        let graphql_schema_path = input.parse::<LitStr>()?;

        let mut override_parameters = OverrideParameters::default();
        let mut parameters = MakeTypesParameters::default();

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
            parameters,
            Default::default(),
        )?;

        Ok(Self { graphql_schema })
    }
}

impl ToTokens for MakeTypes {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.graphql_schema.appsync_types_to_tokens(tokens);
    }
}

/// Entry point for the `make_types!` proc-macro implementation.
pub(crate) fn make_types_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let make_types = parse_macro_input!(input as MakeTypes);
    make_types.into_token_stream().into()
}
