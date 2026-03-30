use proc_macro2::TokenStream;

use crate::internal::make_appsync::overrides::{NameOverrides, TypeOverrides};

use super::*;

#[derive(Debug)]
pub(super) struct TypeTraitDerives {
    additional_trait_derivations: Vec<Path>,
    default_traits: bool,
}

impl Default for TypeTraitDerives {
    fn default() -> Self {
        Self {
            additional_trait_derivations: vec![],
            default_traits: true,
        }
    }
}
impl TypeTraitDerives {
    fn derive_from_default(
        &self,
        default_traits: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let mut traits = TokenStream::new();
        if self.default_traits {
            traits.extend(default_traits);
        }
        if !self.additional_trait_derivations.is_empty() {
            let add_traits = self.additional_trait_derivations.iter();
            traits.extend(quote! {
                #(#add_traits),*
            });
        }
        if self.default_traits || !self.additional_trait_derivations.is_empty() {
            quote! {
                #[derive(#traits)]
            }
        } else {
            TokenStream::new()
        }
    }
    pub(super) fn no_default(&mut self) {
        self.default_traits = false;
    }
    pub(super) fn set_additional_derivations(&mut self, additional_derivations: Vec<Path>) {
        self.additional_trait_derivations = additional_derivations;
    }
    fn includes_default(&self) -> bool {
        self.additional_trait_derivations.iter().any(|p| {
            p.get_ident().is_some_and(|i| {
                let trait_name = i.to_string();
                trait_name == "Default"
            })
        })
    }
    fn includes_serde(&self) -> bool {
        self.default_traits
            || self.additional_trait_derivations.iter().any(|p| {
                p.get_ident().is_some_and(|i| {
                    let trait_name = i.to_string();
                    trait_name == "Serialize" || trait_name == "Deserialize"
                })
            })
    }
}

pub(super) trait Named {
    fn name(&self) -> &str;
}

pub(super) trait ConfigureTraitDerivation: Named {
    fn derives(&mut self) -> &mut TypeTraitDerives;
    fn configure_trait_derivation(&mut self, make_types_parameters: &mut MakeTypesParameters) {
        if make_types_parameters.no_default_traits.remove(self.name()) {
            self.derives().no_default();
        }
        if let Some(add_derivations) = make_types_parameters.add_derivations.remove(self.name()) {
            self.derives().set_additional_derivations(add_derivations);
        }
    }
}

/// A GraphQL object or input object type, generating a Rust `struct` with serde derives.
pub struct Structure {
    name: Name,
    fields: Vec<Field>,
    derives: TypeTraitDerives,
}

impl Named for Structure {
    fn name(&self) -> &str {
        self.name.orig()
    }
}

impl ConfigureTraitDerivation for Structure {
    fn derives(&mut self) -> &mut TypeTraitDerives {
        &mut self.derives
    }
}

impl Structure {
    pub(super) fn apply_type_overrides(
        &mut self,
        type_overrides: &mut TypeOverrides,
    ) -> Result<(), syn::Error> {
        let Some(mut type_overrides) = type_overrides.remove(self.name.orig()) else {
            return Ok(());
        };
        let mut errors = vec![];
        for field in self.fields.iter_mut() {
            let field_name = field.name.orig();
            if let Some((field_override, args_override)) = type_overrides.remove(field_name) {
                if !args_override.is_empty() {
                    errors.extend(args_override.into_values().flat_map(|to| {
                        syn::Error::new(
                            to.arg_name().expect("always set in args_override").span(),
                            "Using args overrides is only supported on operations",
                        )
                    }));
                }
                if let Some(field_override) = field_override {
                    field.field_type.override_type(field_override);
                }
            }
        }
        if !type_overrides.is_empty() {
            errors.extend(
                type_overrides
                    .into_values()
                    .flat_map(|to| to.0.into_iter().chain(to.1.into_values()))
                    .map(|to| {
                        syn::Error::new(
                            to.field_name().span(),
                            format!("No field `{}` in `{}`", to.field_name(), to.type_name()),
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
    pub(super) fn apply_name_overrides(
        &mut self,
        name_overrides: &mut NameOverrides,
    ) -> Result<(), syn::Error> {
        let Some((type_override, mut field_overrides)) = name_overrides.remove(self.name.orig())
        else {
            return Ok(());
        };
        let mut errors = vec![];
        if let Some(type_override) = type_override {
            self.name.override_name(type_override.new_name());
        }
        for field in self.fields.iter_mut() {
            let field_name = field.name.orig();
            if let Some(field_override) = field_overrides.remove(field_name) {
                field.name.override_name(field_override.new_name());
            }
        }
        if !field_overrides.is_empty() {
            errors.extend(field_overrides.into_values().map(|no| {
                syn::Error::new(
                    no.field_name()
                        .expect("always set in field_overrides")
                        .span(),
                    format!(
                        "No field `{}` in `{}`",
                        no.field_name().expect("always set in field_overrides"),
                        no.type_name()
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
}
impl From<graphql_parser::schema::ObjectType<'_, String>> for Structure {
    fn from(value: graphql_parser::schema::ObjectType<'_, String>) -> Self {
        let name = Name::from(value.name);
        let fields = value.fields.into_iter().map(Field::from).collect();
        Self {
            name,
            fields,
            derives: Default::default(),
        }
    }
}
impl From<graphql_parser::schema::InputObjectType<'_, String>> for Structure {
    fn from(value: graphql_parser::schema::InputObjectType<'_, String>) -> Self {
        let name = Name::from(value.name);
        let fields = value.fields.into_iter().map(Field::from).collect();
        Self {
            name,
            fields,
            derives: Default::default(),
        }
    }
}
impl ToTokens for Structure {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let derive = self.derives.derive_from_default(quote! {
            Debug, Clone, ::lambda_appsync::serde::Serialize, ::lambda_appsync::serde::Deserialize,
        });
        let struct_name = self.name.to_type_ident();
        let fields = self
            .fields
            .iter()
            .map(|f| FieldContext::new(f, self.derives.includes_serde()));
        tokens.extend(quote! {
            #derive
            pub struct #struct_name {
                #(#fields,)*
            }
        });
    }
}

/// A GraphQL enum type, generating a Rust `enum` with serde, `Display`, and `FromStr` impls.
#[derive(Debug)]
pub(super) struct Enum {
    name: Name,
    variants: Vec<Name>,
    derives: TypeTraitDerives,
}
impl Named for Enum {
    fn name(&self) -> &str {
        self.name.orig()
    }
}
impl ConfigureTraitDerivation for Enum {
    fn derives(&mut self) -> &mut TypeTraitDerives {
        &mut self.derives
    }
}

impl Enum {
    pub(super) fn apply_name_overrides(
        &mut self,
        name_overrides: &mut NameOverrides,
    ) -> Result<(), syn::Error> {
        let Some((type_override, mut field_overrides)) = name_overrides.remove(self.name.orig())
        else {
            return Ok(());
        };
        let mut errors = vec![];
        if let Some(type_override) = type_override {
            self.name.override_name(type_override.new_name());
        }
        for variant in self.variants.iter_mut() {
            let variant_name = variant.orig();
            if let Some(field_override) = field_overrides.remove(variant_name) {
                variant.override_name(field_override.new_name());
            }
        }
        if !field_overrides.is_empty() {
            errors.extend(field_overrides.into_values().map(|no| {
                syn::Error::new(
                    no.field_name()
                        .expect("always set in field_overrides")
                        .span(),
                    format!(
                        "No variant `{}` in `{}`",
                        no.field_name().expect("always set in field_overrides"),
                        no.type_name()
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
}
impl From<graphql_parser::schema::EnumType<'_, String>> for Enum {
    fn from(value: graphql_parser::schema::EnumType<'_, String>) -> Self {
        let name = Name::from(value.name);
        let variants = value
            .values
            .into_iter()
            .map(|v| Name::from(v.name))
            .collect();
        Self {
            name,
            variants,
            derives: Default::default(),
        }
    }
}
impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let derive = self.derives.derive_from_default(quote! {
            Debug, Clone, Copy, ::lambda_appsync::serde::Serialize, ::lambda_appsync::serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
        });
        let enum_name = self.name.to_type_ident();
        let count = proc_macro2::Literal::usize_unsuffixed(self.variants.len());
        let variant_orig_iter = self.variants.iter().map(|n| n.orig()).collect::<Vec<_>>();
        let variants = self
            .variants
            .iter()
            .map(|n| n.to_type_ident())
            .collect::<Vec<_>>();
        let error_message = format!("`{{}}` is an invalid value for enum {}", enum_name);
        let index_fct_arms = variants
            .iter()
            .enumerate()
            .map(|(idx, var)| quote! {Self::#var => #idx});

        let rendered_variants = variants
            .iter()
            .zip(variant_orig_iter.iter())
            .enumerate()
            .map(|(idx, (var, variant_orig))| {
                let default = (self.derives.includes_default() && idx == 0).then(|| {
                    quote! {
                        #[default]
                    }
                });
                let serde_rename = self.derives.includes_serde().then(|| {
                    quote! {
                        #[serde(rename = #variant_orig)]
                    }
                });
                quote! {
                    #default
                    #serde_rename
                    #var
                }
            });

        tokens.extend(quote! {
            #derive
            pub enum #enum_name {
                #(#rendered_variants,)*
            }
            impl #enum_name {
                pub const COUNT: usize = #count;
                pub const fn all() -> [Self; Self::COUNT] {
                    [#(Self::#variants,)*]
                }
                pub const fn index(self) -> usize {
                    match self {
                        #(#index_fct_arms,)*
                    }
                }
            }
        });
        if self.derives.default_traits {
            tokens.extend(quote! {
                impl ::core::fmt::Display for #enum_name {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        match self {
                            #(Self::#variants => write!(f, #variant_orig_iter),)*
                        }
                    }
                }
                impl ::core::str::FromStr for #enum_name {
                    type Err = ::lambda_appsync::AppsyncError;

                    fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                        match s {
                            #(#variant_orig_iter => ::core::result::Result::Ok(Self::#variants),)*
                            _ => ::core::result::Result::Err(::lambda_appsync::AppsyncError::new(
                                "InvalidStr",
                                format!(#error_message, s),
                            ))
                        }
                    }
                }
            });
        }
    }
}
