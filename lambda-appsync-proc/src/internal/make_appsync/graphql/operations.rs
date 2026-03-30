use super::*;

/// A single GraphQL query, mutation, or subscription field with its arguments and return type.
pub(super) struct Operation {
    name: Name,
    args: Vec<Field>,
    return_type: FieldType,
}
impl Operation {
    fn variant(&self) -> Ident {
        self.name.to_type_ident()
    }
    fn default_op(&self, kind: OperationKind) -> proc_macro2::TokenStream {
        let fct_name = self.name.to_prefixed_fct_ident(kind.fct_prefix());
        let span = graphql_path_span();
        let return_type = match kind {
            OperationKind::Query | OperationKind::Mutation => {
                let return_type = &self.return_type;
                quote_spanned! {span=>#return_type}
            }
            OperationKind::Subscription => quote_spanned! {span=>
                ::core::option::Option<::lambda_appsync::subscription_filters::FilterGroup>
            },
        };
        let default_body = match kind {
            OperationKind::Query | OperationKind::Mutation => {
                let unimplemented_message =
                    format!("{kind} `{}` is unimplemented", self.name.orig());
                quote_spanned! {span=>
                    ::core::result::Result::Err(::lambda_appsync::AppsyncError::new(
                        "Unimplemented",
                        #unimplemented_message,
                    ))
                }
            }
            OperationKind::Subscription => quote_spanned! {span=>
                ::core::result::Result::Ok(None)
            },
        };
        quote_spanned! {span=>
            async fn #fct_name(_event: ::lambda_appsync::AppsyncEvent<Operation>) -> ::core::result::Result<#return_type, ::lambda_appsync::AppsyncError> {
                #default_body
            }
        }
    }
    fn execute_match_arm(&self, kind: OperationKind) -> proc_macro2::TokenStream {
        let span = graphql_path_span();
        let operation_enum_name = kind.operation_enum_name(span);
        let variant = self.name.to_type_ident();
        let fct_name = self.name.to_prefixed_fct_ident(kind.fct_prefix());
        quote_spanned! {span=>
            #operation_enum_name::#variant => Operation::#fct_name(event)
            .await
            .map(::lambda_appsync::res_to_json)
        }
    }
    fn argument_extractor(&self, with_event: bool) -> proc_macro2::TokenStream {
        let span = graphql_path_span();
        let params_types = self.args.iter().map(|arg| &arg.field_type);
        let param_strs = self.args.iter().map(|arg| arg.name.orig());

        let return_type = if with_event {
            quote! {
                (#(#params_types,)* &::lambda_appsync::AppsyncEvent<Operation>,)
            }
        } else {
            quote! {
                (#(#params_types,)*)
            }
        };
        let returned_tuple = if with_event {
            quote! {
                (#(::lambda_appsync::arg_from_json(&mut args, #param_strs)?,)* event,)
            }
        } else {
            quote! {
                (#(::lambda_appsync::arg_from_json(&mut args, #param_strs)?,)*)
            }
        };

        let extract_args = if self.args.is_empty() {
            quote! {
                _ = event.args.take();
            }
        } else {
            quote! {
                let mut args = event.args.take();
            }
        };
        quote_spanned! {span=>
            pub(crate) fn operation_arguments(event: &mut ::lambda_appsync::AppsyncEvent<Operation>) -> ::core::result::Result<#return_type, ::lambda_appsync::AppsyncError> {
                #extract_args
                 Ok(#returned_tuple)
            }
        }
    }
    fn operation_module(&self, kind: OperationKind) -> proc_macro2::TokenStream {
        let module_name = self.name.to_var_ident();
        let params_types = self
            .args
            .iter()
            .map(|arg| &arg.field_type)
            .collect::<Vec<_>>();
        let arument_extractor_without_event = self.argument_extractor(false);
        let arument_extractor_with_event = self.argument_extractor(true);
        let return_type = match kind {
            OperationKind::Query | OperationKind::Mutation => {
                let return_type = &self.return_type;
                quote_spanned! {return_type.span()=>
                    ::core::result::Result<#return_type, ::lambda_appsync::AppsyncError>
                }
            }
            OperationKind::Subscription => quote_spanned! {graphql_path_span()=>
                ::core::result::Result<::core::option::Option<::lambda_appsync::subscription_filters::FilterGroup>, ::lambda_appsync::AppsyncError>
            },
        };

        quote! {
            pub(crate) mod #module_name {
                pub(crate) mod without_event {
                    use super::super::super::*;
                    pub(crate) fn check_signature<F: Fn(#(#params_types),*) -> #return_type>(_f: F) {}
                    #arument_extractor_without_event
                }
                pub(crate) mod with_event {
                    use super::super::super::*;
                    pub(crate) fn check_signature<F: Fn(#(#params_types,)* &::lambda_appsync::AppsyncEvent<Operation>) -> #return_type>(_f: F) {}
                    #arument_extractor_with_event
                }
            }
        }
    }
    fn apply_type_overrides(
        &mut self,
        (field_type_override, mut arg_type_overrides): FieldTypeOverride,
    ) -> Result<(), syn::Error> {
        let mut errors = vec![];
        if let Some(field_type_override) = field_type_override {
            self.return_type.override_type(field_type_override);
        }
        for arg in self.args.iter_mut() {
            let arg_name = arg.name.orig();
            if let Some(arg_type_override) = arg_type_overrides.remove(arg_name) {
                arg.field_type.override_type(arg_type_override);
            }
        }
        if !arg_type_overrides.is_empty() {
            errors.extend(arg_type_overrides.into_values().map(|to| {
                syn::Error::new(
                    to.arg_name()
                        .expect("always set in arg_type_overrides")
                        .span(),
                    format!(
                        "No argument `{}` in operation `{}::{}`",
                        to.arg_name().expect("always set in arg_type_overrides"),
                        to.type_name(),
                        to.field_name(),
                    ),
                )
            }));
        }
        if errors.is_empty() {
            Ok(())
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
    fn apply_type_module_path(&mut self, path: &Path) {
        for arg in self.args.iter_mut() {
            arg.field_type.add_path(path);
        }
        self.return_type.add_path(path);
    }
}
impl From<graphql_parser::schema::Field<'_, String>> for Operation {
    fn from(value: graphql_parser::schema::Field<'_, String>) -> Self {
        let name = Name::from(value.name);
        let args = value.arguments.into_iter().map(Field::from).collect();
        let return_type = FieldType::from(value.field_type);
        Self {
            name,
            args,
            return_type,
        }
    }
}

/// A collection of [`Operation`]s belonging to a single GraphQL operation type (Query, Mutation, or Subscription).
#[derive(Default)]
pub(super) struct Operations(Vec<Operation>);
impl From<graphql_parser::schema::ObjectType<'_, String>> for Operations {
    fn from(value: graphql_parser::schema::ObjectType<'_, String>) -> Self {
        Self(value.fields.into_iter().map(Operation::from).collect())
    }
}
impl Operations {
    pub(super) fn variants_iter(&self) -> impl Iterator<Item = Ident> + '_ {
        self.0.iter().map(Operation::variant)
    }
    pub(super) fn default_op_iter(
        &self,
        kind: OperationKind,
    ) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        self.0.iter().map(move |op| op.default_op(kind))
    }
    pub(super) fn execute_match_arm_iter(
        &self,
        kind: OperationKind,
    ) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        self.0.iter().map(move |op| op.execute_match_arm(kind))
    }
    pub(super) fn operation_module_iter(
        &self,
        kind: OperationKind,
    ) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        self.0.iter().map(move |op| op.operation_module(kind))
    }
    pub(super) fn apply_type_overrides(
        &mut self,
        mut type_overrides: FieldTypeOverrides,
    ) -> Result<(), syn::Error> {
        let mut errors = vec![];
        for op in self.0.iter_mut() {
            let op_name = op.name.orig();
            if let Some(type_override) = type_overrides.remove(op_name) {
                match op.apply_type_overrides(type_override) {
                    Ok(_) => (),
                    Err(e) => errors.push(e),
                };
            }
        }
        if !type_overrides.is_empty() {
            errors.extend(
                type_overrides
                    .into_values()
                    .flat_map(|fo| fo.0.into_iter().chain(fo.1.into_values()))
                    .map(|to| {
                        syn::Error::new(
                            to.field_name().span(),
                            format!("No operation `{}` in `{}`", to.field_name(), to.type_name()),
                        )
                    }),
            );
        }
        if errors.is_empty() {
            Ok(())
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

    pub(super) fn apply_type_module_path(&mut self, path: &Path) {
        for op in self.0.iter_mut() {
            op.apply_type_module_path(path);
        }
    }
}
