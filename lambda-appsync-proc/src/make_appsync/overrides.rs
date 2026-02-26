use std::collections::HashMap;

use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    Ident, Type,
};

// TODO TMP crate vis, should be super
pub(crate) struct TypeOverride {
    type_name: Ident,
    field_name: Ident,
    arg_name: Option<Ident>,
    type_ident: Type,
}
impl TypeOverride {
    pub(crate) fn type_name(&self) -> &Ident {
        &self.type_name
    }
    pub(crate) fn field_name(&self) -> &Ident {
        &self.field_name
    }
    pub(crate) fn arg_name(&self) -> Option<&Ident> {
        self.arg_name.as_ref()
    }
    pub(super) fn type_ident(self) -> Type {
        self.type_ident
    }
}
impl Parse for TypeOverride {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let type_name = input.call(Ident::parse_any)?;
        _ = input.parse::<syn::Token![.]>()?;
        let field_name = input.call(Ident::parse_any)?;
        let arg_name = if input.peek(syn::Token![.]) {
            _ = input.parse::<syn::Token![.]>()?;
            Some(input.call(Ident::parse_any)?)
        } else {
            None
        };
        _ = input.parse::<syn::Token![:]>()?;
        let type_ident = input
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "Expected a Type (struct, enum, etc...)"))?;
        Ok(Self {
            type_name,
            field_name,
            arg_name,
            type_ident,
        })
    }
}

// TODO TMP crate vis, should be super
pub(crate) struct NameOverride {
    type_name: Ident,
    field_name: Option<Ident>,
    new_name: String,
}
impl NameOverride {
    pub(crate) fn type_name(&self) -> &Ident {
        &self.type_name
    }
    pub(crate) fn field_name(&self) -> Option<&Ident> {
        self.field_name.as_ref()
    }
    pub(super) fn new_name(self) -> String {
        self.new_name
    }
}
impl Parse for NameOverride {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let type_name = input.call(Ident::parse_any)?;
        let field_name = if input.peek(syn::Token![.]) {
            _ = input.parse::<syn::Token![.]>()?;
            Some(input.call(Ident::parse_any)?)
        } else {
            None
        };
        _ = input.parse::<syn::Token![:]>()?;
        let new_name = input.call(Ident::parse_any)?;
        let new_name = new_name.to_string();
        Ok(Self {
            type_name,
            field_name,
            new_name,
        })
    }
}

// Captures type_override = Type.field: CustomType and Type.field.param: CustomType options
// using a HashMap hierarchy of TypeName -> FieldName -> (Optional field override, Map of arg overrides)
// Top level mapping from GraphQL type names to their field overrides
pub(crate) type TypeOverrides = HashMap<TypeName, FieldTypeOverrides>;

// For each type, maps field names to their overrides
pub(crate) type FieldTypeOverrides = HashMap<FieldName, FieldTypeOverride>;

// A field can have both a direct type override and argument type overrides
// - First element: Optional field type override (Type.field: CustomType)
// - Second element: Map of argument overrides (Type.field.arg: CustomType)
pub(crate) type FieldTypeOverride = (Option<TypeOverride>, ArgTypeOverrides);

// Maps argument names to their type overrides for a field
pub(crate) type ArgTypeOverrides = HashMap<ArgName, TypeOverride>;

// Captures name_override = Type: CustomName and Type.field: custom_name options
// using a HashMap hierarchy of TypeName -> (Optional type override, Map of field overrides)
// This works the same for name_override = Enum: CustomEnumName and Enum.VARIANT: CustomVariant
// Top level mapping from GraphQL type names to their field overrides
pub(crate) type NameOverrides = HashMap<TypeName, TypeNameOverride>;

// A type can have both a direct name override and field name overrides
// - First element: Optional type name override (Type: CustomName)
// - Second element: Map of field overrides (Type.field: custom_name)
pub(crate) type TypeNameOverride = (Option<NameOverride>, FieldNameOverrides);

// Maps field names to their name overrides for a field
pub(crate) type FieldNameOverrides = HashMap<FieldName, NameOverride>;

// [Type|Field|Arg]Name are just String
pub(crate) type TypeName = String;
pub(crate) type FieldName = String;
pub(crate) type ArgName = String;

use super::optional_parameter::{OptionalParameter, OptionalParameters, ParameterError, Unknown};
pub(super) enum OverrideParameter {
    TypeOverride(TypeOverride),
    NameOverride(NameOverride),
}

impl OptionalParameter for OverrideParameter {
    fn try_parse_parameter(input: ParseStream) -> Result<Self, ParameterError> {
        let ident = Self::parse_ident(input)?;
        match ident.to_string().as_str() {
            "type_override" => Ok(Self::TypeOverride(input.parse()?)),
            "name_override" => Ok(Self::NameOverride(input.parse()?)),
            // Unknown option
            _ => ident.unknown(),
        }
    }
}

#[derive(Default)]
pub(super) struct OverrideParameters {
    pub type_overrides: TypeOverrides,
    pub name_overrides: NameOverrides,
}
impl OptionalParameters<OverrideParameter> for OverrideParameters {
    fn set_param(&mut self, p: OverrideParameter) {
        match p {
            OverrideParameter::TypeOverride(to) => {
                // Retrieve the entry corresponding to `Type.field`
                let to_field_entry = self
                    .type_overrides
                    .entry(to.type_name().to_string())
                    .or_default()
                    .entry(to.field_name().to_string())
                    .or_default();
                if let Some(arg_name) = to.arg_name() {
                    // There is a `.param`
                    // This is a parameter override
                    to_field_entry.1.insert(arg_name.to_string(), to);
                } else {
                    // no `.param`
                    // This is just a field override
                    to_field_entry.0.replace(to);
                }
            }
            OverrideParameter::NameOverride(no) => {
                // Retrieve the entry corresponding to `Type`
                let no_type_entry = self
                    .name_overrides
                    .entry(no.type_name().to_string())
                    .or_default();
                if let Some(field_name) = no.field_name() {
                    // There is a `.field`
                    // This is a field override
                    no_type_entry.1.insert(field_name.to_string(), no);
                } else {
                    // no `.field`
                    // This is just a type override
                    no_type_entry.0.replace(no);
                }
            }
        }
    }
}
