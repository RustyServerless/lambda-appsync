mod fields;
mod operations;
mod types;
use fields::*;
use operations::*;
use types::*;

use graphql_parser::schema::{Definition, TypeDefinition};

use quote::{quote, quote_spanned, ToTokens};
use syn::Path;
use syn::{spanned::Spanned, Ident, LitStr};

use crate::internal::make_appsync::{MakeOperationParameters, MakeTypesParameters};

use super::super::common::{Name, OperationKind};
use super::overrides::{FieldTypeOverride, FieldTypeOverrides, OverrideParameters, TypeOverride};

/// The root type names for each operation kind, parsed from the GraphQL `schema { ... }` block.
///
/// Defaults to `Query`, `Mutation`, and `Subscription` when no explicit schema definition is present.
#[derive(Debug)]
struct SchemaDefinition {
    query: String,
    mutation: String,
    subscription: String,
}
impl SchemaDefinition {
    fn schema_definition(&self, name: &str) -> Option<OperationKind> {
        if name == self.query {
            Some(OperationKind::Query)
        } else if name == self.mutation {
            Some(OperationKind::Mutation)
        } else if name == self.subscription {
            Some(OperationKind::Subscription)
        } else {
            None
        }
    }
}
impl Default for SchemaDefinition {
    fn default() -> Self {
        Self {
            query: "Query".to_owned(),
            mutation: "Mutation".to_owned(),
            subscription: "Subscription".to_owned(),
        }
    }
}
impl From<graphql_parser::schema::SchemaDefinition<'_, String>> for SchemaDefinition {
    fn from(value: graphql_parser::schema::SchemaDefinition<'_, String>) -> Self {
        let mut sd = Self::default();
        if let Some(query) = value.query {
            sd.query = query
        }
        if let Some(mutation) = value.mutation {
            sd.mutation = mutation
        }
        if let Some(subscription) = value.subscription {
            sd.subscription = subscription
        }
        sd
    }
}

/// The fully parsed and override-applied GraphQL schema, ready for code generation.
pub(super) struct GraphQLSchema {
    queries: Operations,
    mutations: Operations,
    subscriptions: Operations,
    structures: Vec<Structure>,
    enums: Vec<Enum>,
    make_operation_parameters: MakeOperationParameters,
}
impl GraphQLSchema {
    pub(super) fn new(
        graphql_schema_path: LitStr,
        override_parameters: OverrideParameters,
        mut make_types_parameters: MakeTypesParameters,
        make_operation_parameters: MakeOperationParameters,
    ) -> Result<Self, syn::Error> {
        let mut queries = None;
        let mut mutations = None;
        let mut subscriptions = None;
        let mut structures = vec![];
        let mut enums = vec![];

        let path_value = graphql_schema_path.value();
        let span = graphql_schema_path.span();
        let full_path = if std::path::Path::new(&path_value).is_relative() {
            std::env::current_dir()
                .map_err(|e| {
                    syn::Error::new(span, format!("Could not get current directory: {e}"))
                })?
                .join(&path_value)
        } else {
            std::path::PathBuf::from(path_value)
        };
        let schema_str = std::fs::read_to_string(&full_path).map_err(|e| {
            syn::Error::new(
                span,
                format!(
                    "Could not open GraphQL schema file at '{}' ({e})",
                    full_path.display()
                ),
            )
        })?;

        let mut graphql_schema = graphql_parser::parse_schema(&schema_str)
            .map_err(|e| {
                syn::Error::new(span, format!("Could not parse GraphQL schema file ({e})",))
            })?
            .into_static();

        let sd = if let Some(index) = graphql_schema
            .definitions
            .iter()
            .position(|def| matches!(def, Definition::SchemaDefinition(_)))
        {
            let Definition::SchemaDefinition(def) = graphql_schema.definitions.swap_remove(index)
            else {
                unreachable!("just verified it is a schema def")
            };
            SchemaDefinition::from(def)
        } else {
            SchemaDefinition::default()
        };

        let OverrideParameters {
            mut type_overrides,
            mut name_overrides,
        } = override_parameters;

        let mut errors = vec![];
        for definition in graphql_schema.definitions {
            match definition {
                Definition::TypeDefinition(type_definition) => {
                    match type_definition {
                        TypeDefinition::Object(object_type) => {
                            if let Some(op_kind) = sd.schema_definition(&object_type.name) {
                                // This is an Object defining operations
                                let type_overrides = type_overrides.remove(&object_type.name);
                                let mut ops = Operations::from(object_type);
                                if let MakeOperationParameters {
                                    type_module: Some(ref custom_type_module),
                                    ..
                                } = make_operation_parameters
                                {
                                    ops.apply_type_module_path(custom_type_module);
                                }
                                if let Some(type_overrides) = type_overrides {
                                    match ops.apply_type_overrides(type_overrides) {
                                        Ok(_) => (),
                                        Err(e) => errors.push(e),
                                    };
                                }
                                match op_kind {
                                    OperationKind::Query => {
                                        queries.replace(ops);
                                    }
                                    OperationKind::Mutation => {
                                        mutations.replace(ops);
                                    }
                                    OperationKind::Subscription => {
                                        subscriptions.replace(ops);
                                    }
                                }
                            } else {
                                // This is an object defining a structure-style type
                                let mut structure = Structure::from(object_type);
                                match structure.apply_type_overrides(&mut type_overrides) {
                                    Ok(_) => (),
                                    Err(e) => errors.push(e),
                                };
                                match structure.apply_name_overrides(&mut name_overrides) {
                                    Ok(_) => (),
                                    Err(e) => errors.push(e),
                                };
                                structure.configure_trait_derivation(&mut make_types_parameters);

                                structures.push(structure);
                            }
                        }
                        TypeDefinition::Enum(enum_type) => {
                            let mut r_enum = Enum::from(enum_type);
                            match r_enum.apply_name_overrides(&mut name_overrides) {
                                Ok(_) => (),
                                Err(e) => errors.push(e),
                            };
                            r_enum.configure_trait_derivation(&mut make_types_parameters);
                            enums.push(r_enum);
                        }
                        TypeDefinition::InputObject(input_object_type) => {
                            let mut structure = Structure::from(input_object_type);
                            match structure.apply_type_overrides(&mut type_overrides) {
                                Ok(_) => (),
                                Err(e) => errors.push(e),
                            };
                            match structure.apply_name_overrides(&mut name_overrides) {
                                Ok(_) => (),
                                Err(e) => errors.push(e),
                            };
                            structure.configure_trait_derivation(&mut make_types_parameters);

                            structures.push(structure);
                        }
                        // Not yet implemented, ignored for now
                        TypeDefinition::Scalar(_) => {}
                        TypeDefinition::Interface(_) => (),
                        TypeDefinition::Union(_) => (),
                    }
                }
                // Already processed
                Definition::SchemaDefinition(_) => {
                    return Err(syn::Error::new(
                        span,
                        "GraphQL schema file has two `schema` definition",
                    ));
                }
                // Ignored for now
                Definition::TypeExtension(_) => (),
                Definition::DirectiveDefinition(_) => (),
            }
        }

        // Add an error for each un-matched type_override
        errors.extend(
            type_overrides
                .into_values()
                .flat_map(|field_type_overrides| field_type_overrides.into_values())
                .flat_map(|field_type_override| {
                    field_type_override
                        .0
                        .into_iter()
                        .chain(field_type_override.1.into_values())
                })
                .map(|type_override| {
                    syn::Error::new(
                        type_override.type_name().span(),
                        format!("No type or input named `{}`", type_override.type_name()),
                    )
                }),
        );

        // Add an error for each un-matched name_override
        errors.extend(
            name_overrides
                .into_values()
                .flat_map(|name_overrides| {
                    name_overrides
                        .0
                        .into_iter()
                        .chain(name_overrides.1.into_values())
                })
                .map(|name_override| {
                    syn::Error::new(
                        name_override.type_name().span(),
                        format!(
                            "No type, enum or input named `{}`",
                            name_override.type_name()
                        ),
                    )
                }),
        );

        if errors.is_empty() {
            Ok(Self {
                queries: queries.unwrap_or_default(),
                mutations: mutations.unwrap_or_default(),
                subscriptions: subscriptions.unwrap_or_default(),
                structures,
                enums,
                make_operation_parameters,
            })
        } else {
            Err(errors
                .into_iter()
                .reduce(|mut acc, e| {
                    acc.combine(e);
                    acc
                })
                .expect("not empty"))
        }
    }
    fn enums_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enums = self.enums.iter();

        tokens.extend(quote! {
            #(#enums)*
        });
    }
    fn structs_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let structures = self.structures.iter();

        tokens.extend(quote! {
            #(#structures)*
        });
    }
    fn operation_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let query_field_name = OperationKind::Query.operation_enum_name();
        let query_field_variants = self.queries.variants_iter();
        let mutation_field_name = OperationKind::Mutation.operation_enum_name();
        let mutation_field_variants = self.mutations.variants_iter();
        let subscription_field_name = OperationKind::Subscription.operation_enum_name();
        let subscription_field_variants = self.subscriptions.variants_iter();
        tokens.extend(quote! {
            #[derive(Debug, Clone, Copy, ::lambda_appsync::serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub enum #query_field_name {
                #(#query_field_variants,)*
            }
            #[derive(Debug, Clone, Copy, ::lambda_appsync::serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub enum #mutation_field_name {
                #(#mutation_field_variants,)*
            }
            #[derive(Debug, Clone, Copy, ::lambda_appsync::serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub enum #subscription_field_name {
                #(#subscription_field_variants,)*
            }
            #[doc = "Represent all the possible operations (Query, Mutation, Subscription) defined by the GraphQL schema."]
            #[doc = "The operations are implemented by using the [`macro@appsync_operation`](crate::appsync_operation) attribute macro on your functions."]
            #[derive(Debug, Clone, Copy, ::lambda_appsync::serde::Deserialize)]
            #[serde(tag = "parentTypeName", content = "fieldName")]
            pub enum Operation {
                Query(#query_field_name),
                Mutation(#mutation_field_name),
                Subscription(#subscription_field_name),
            }
            use __operations::DefaultOperations;
            impl DefaultOperations for Operation {}
        });
    }
    fn default_operations_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let query_field_default_ops = self.queries.default_op_iter(OperationKind::Query);
        let mutation_field_default_ops = self.mutations.default_op_iter(OperationKind::Mutation);
        let subscription_field_default_ops = self
            .subscriptions
            .default_op_iter(OperationKind::Subscription);
        tokens.extend(quote! {
            pub(super) trait DefaultOperations {
                #(#query_field_default_ops)*
                #(#mutation_field_default_ops)*
                #(#subscription_field_default_ops)*
            }
        });
    }

    fn impl_operation_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let query_field_execute_match_arm =
            self.queries.execute_match_arm_iter(OperationKind::Query);
        let mutation_field_execute_match_arm = self
            .mutations
            .execute_match_arm_iter(OperationKind::Mutation);
        let subscription_field_execute_match_arm = self
            .subscriptions
            .execute_match_arm_iter(OperationKind::Subscription);

        #[allow(unused_mut)]
        let mut log_lines = proc_macro2::TokenStream::new();
        #[cfg(feature = "log")]
        if self.make_operation_parameters.error_logging {
            log_lines.extend(quote! {
                ::lambda_appsync::log::error!("{e}");
            });
        }

        tokens.extend(quote! {
            impl Operation {
                #[doc = "Will retrieve the [`Operation`] for the [`AppsyncEvent`](::lambda_appsync::AppsyncEvent)"]
                #[doc = "and extract the API call parameters to call the user code marked with the correspoding [`macro@appsync_operation`](crate::appsync_operation)"]
                pub async fn execute(self,
                    event: ::lambda_appsync::AppsyncEvent<Self>
                ) -> ::lambda_appsync::AppsyncResponse {
                    match self._execute(event).await {
                        ::core::result::Result::Ok(v) => v.into(),
                        ::core::result::Result::Err(e) => {
                            #log_lines
                            e.into()
                        }
                    }
                }
                async fn _execute(
                    self,
                    event: ::lambda_appsync::AppsyncEvent<Self>
                ) -> ::core::result::Result<::lambda_appsync::serde_json::Value, ::lambda_appsync::AppsyncError> {
                    match self {
                        Operation::Query(query_field) => match query_field {
                            #(#query_field_execute_match_arm,)*
                        },
                        Operation::Mutation(mutation_field) => match mutation_field {
                            #(#mutation_field_execute_match_arm,)*
                        },
                        Operation::Subscription(subscription_field) => match subscription_field {
                            #(#subscription_field_execute_match_arm,)*
                        },
                    }
                }
            }
        });
    }
    fn operations_module_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut default_operations_trait = proc_macro2::TokenStream::new();
        self.default_operations_to_tokens(&mut default_operations_trait);
        let query_operation_module_iter = self.queries.operation_module_iter(OperationKind::Query);
        let mutation_operation_module_iter = self
            .mutations
            .operation_module_iter(OperationKind::Mutation);
        let subscription_operation_module_iter = self
            .subscriptions
            .operation_module_iter(OperationKind::Subscription);
        tokens.extend(quote! {
            #[allow(dead_code)]
            mod __operations {
                use super::*;
                #default_operations_trait
                pub(crate) mod queries {
                    #(#query_operation_module_iter)*
                }
                pub(crate) mod mutations {
                    #(#mutation_operation_module_iter)*
                }
                pub(crate) mod subscriptions {
                    #(#subscription_operation_module_iter)*
                }
            }
        });
    }
    pub(crate) fn appsync_types_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.enums_to_tokens(tokens);
        self.structs_to_tokens(tokens);
    }
    pub(crate) fn appsync_operations_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.operations_module_to_tokens(tokens);
        self.operation_to_tokens(tokens);
        self.impl_operation_to_tokens(tokens);
    }
}
