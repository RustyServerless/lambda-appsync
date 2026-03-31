use super::*;

/// A GraphQL scalar type, including both standard and AWS AppSync-specific scalars.
pub(super) enum Scalar {
    String,
    ID,
    Int,
    Float,
    Boolean,
    AWSEmail,
    AWSPhone,
    AWSTimestamp,
    AWSDate,
    AWSTime,
    AWSDateTime,
    #[allow(clippy::upper_case_acronyms)]
    AWSJSON,
    #[allow(clippy::upper_case_acronyms)]
    AWSURL,
    AWSIPAddress,
}
impl TryFrom<&str> for Scalar {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "String" => Ok(Self::String),
            "ID" => Ok(Self::ID),
            "Int" => Ok(Self::Int),
            "Float" => Ok(Self::Float),
            "Boolean" => Ok(Self::Boolean),
            "AWSEmail" => Ok(Self::AWSEmail),
            "AWSPhone" => Ok(Self::AWSPhone),
            "AWSTimestamp" => Ok(Self::AWSTimestamp),
            "AWSDate" => Ok(Self::AWSDate),
            "AWSTime" => Ok(Self::AWSTime),
            "AWSDateTime" => Ok(Self::AWSDateTime),
            "AWSJSON" => Ok(Self::AWSJSON),
            "AWSURL" => Ok(Self::AWSURL),
            "AWSIPAddress" => Ok(Self::AWSIPAddress),
            _ => Err(()),
        }
    }
}
impl ToTokens for Scalar {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Scalar::String => quote! {String},
            Scalar::ID => quote! {::lambda_appsync::ID},
            Scalar::Int => quote! {i32},
            Scalar::Float => quote! {f64},
            Scalar::Boolean => quote! {bool},
            Scalar::AWSEmail => quote! {::lambda_appsync::AWSEmail},
            Scalar::AWSPhone => quote! {::lambda_appsync::AWSPhone},
            Scalar::AWSTimestamp => quote! {::lambda_appsync::AWSTimestamp},
            Scalar::AWSDate => quote! {::lambda_appsync::AWSDate},
            Scalar::AWSTime => quote! {::lambda_appsync::AWSTime},
            Scalar::AWSDateTime => quote! {::lambda_appsync::AWSDateTime},
            Scalar::AWSJSON => quote! {::lambda_appsync::serde_json::Value},
            Scalar::AWSURL => quote! {::lambda_appsync::AWSUrl},
            Scalar::AWSIPAddress => quote! {::core::net::IpAddr},
        })
    }
}

/// The resolved Rust type for a GraphQL field, including nullability and list wrapping.
pub(super) enum FieldType {
    /// A user-supplied type override replacing the inferred type.
    Overriden(syn::Type),
    /// A custom GraphQL object or enum type, optionally qualified with a module path.
    Custom { name: Name, path: Option<Path> },
    /// A built-in GraphQL scalar.
    Scalar(Scalar),
    /// A GraphQL list type (`[T]`), mapped to `Vec<T>`.
    List(Box<FieldType>),
    /// A nullable GraphQL type, mapped to `Option<T>`.
    Optionnal(Box<FieldType>),
}
impl FieldType {
    fn from_string(name: String) -> Self {
        if let Ok(scalar) = Scalar::try_from(name.as_str()) {
            Self::Scalar(scalar)
        } else {
            let name = Name::from(name);
            Self::Custom { name, path: None }
        }
    }
    fn is_optionnal(&self) -> bool {
        matches!(self, FieldType::Optionnal(_))
    }
    pub(super) fn override_type(&mut self, type_override: TypeOverride) {
        match self {
            FieldType::Overriden(_) | FieldType::Custom { .. } | FieldType::Scalar(_) => {
                *self = FieldType::Overriden(type_override.type_ident())
            }
            FieldType::List(field_type) => field_type.override_type(type_override),
            FieldType::Optionnal(field_type) => field_type.override_type(type_override),
        }
    }
    pub(super) fn add_path(&mut self, p: &Path) {
        match self {
            FieldType::Custom { path, .. } => {
                path.replace(p.clone());
            }
            FieldType::Overriden(_) | FieldType::Scalar(_) => {}
            FieldType::List(field_type) => field_type.add_path(p),
            FieldType::Optionnal(field_type) => field_type.add_path(p),
        }
    }
}
impl From<graphql_parser::schema::Type<'_, String>> for FieldType {
    fn from(value: graphql_parser::schema::Type<'_, String>) -> Self {
        match value {
            graphql_parser::query::Type::NamedType(name) => {
                Self::Optionnal(Box::new(FieldType::from_string(name)))
            }
            graphql_parser::query::Type::ListType(inner) => {
                Self::Optionnal(Box::new(Self::List(Box::new(FieldType::from(*inner)))))
            }
            graphql_parser::query::Type::NonNullType(inner) => {
                let inner = *inner;
                match inner {
                    graphql_parser::query::Type::NamedType(name) => FieldType::from_string(name),
                    graphql_parser::query::Type::ListType(inner) => {
                        Self::List(Box::new(FieldType::from(*inner)))
                    }
                    graphql_parser::query::Type::NonNullType(_) => {
                        unreachable!("Double NonNullType is not supported")
                    }
                }
            }
        }
    }
}
impl ToTokens for FieldType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldType::Custom { name, path } => {
                if let Some(path) = path {
                    tokens.extend(quote! {#path::})
                }
                let name = name.to_type_ident();
                tokens.extend(quote! {#name})
            }
            FieldType::Scalar(scalar) => tokens.extend(quote! {#scalar}),
            FieldType::List(field_type) => tokens.extend(quote! {::std::vec::Vec<#field_type>}),
            FieldType::Optionnal(field_type) => {
                tokens.extend(quote! {::core::option::Option<#field_type>})
            }
            FieldType::Overriden(ty) => tokens.extend(quote! {#ty}),
        }
    }
}

/// A single field or input value from a GraphQL object or input type.
pub(super) struct Field {
    pub(super) name: Name,
    pub(super) field_type: FieldType,
}
impl From<graphql_parser::schema::Field<'_, String>> for Field {
    fn from(value: graphql_parser::schema::Field<'_, String>) -> Self {
        let name = Name::from(value.name);
        let field_type = FieldType::from(value.field_type);
        Self { name, field_type }
    }
}
impl From<graphql_parser::schema::InputValue<'_, String>> for Field {
    fn from(value: graphql_parser::schema::InputValue<'_, String>) -> Self {
        let name = Name::from(value.name);
        let field_type = FieldType::from(value.value_type);
        Self { name, field_type }
    }
}

/// A wrapper around a [`Field`] that generates the struct field tokens, including serde attributes.
pub(super) struct FieldContext<'a> {
    field: &'a Field,
    with_serde: bool,
}
impl<'a> FieldContext<'a> {
    pub(super) fn new(field: &'a Field, with_serde: bool) -> Self {
        Self { field, with_serde }
    }
}
impl ToTokens for FieldContext<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field = self.field;
        let name = field.name.to_var_ident();
        // If the `name` identifier is different from the original name, we must serde_rename the type
        let orig_name = field.name.orig();

        let field_type = &field.field_type;
        let mut serde_options = vec![];

        if name != orig_name {
            serde_options.push(quote! {
                rename = #orig_name
            });
        }
        if field_type.is_optionnal() {
            serde_options.push(quote! {
                default, skip_serializing_if = "Option::is_none"
            });
        }
        if !serde_options.is_empty() && self.with_serde {
            tokens.extend(quote! {
                #[serde(#(#serde_options),*)]
            })
        }
        tokens.extend(quote! {
            pub #name: #field_type
        });
    }
}
