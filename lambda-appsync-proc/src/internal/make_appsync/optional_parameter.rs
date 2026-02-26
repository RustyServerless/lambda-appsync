use syn::{
    parse::{discouraged::Speculative, ParseStream},
    Ident, Token,
};

pub(crate) enum ParameterError {
    InexistantParameter(syn::Error),
    NotParameter(syn::Error),
    ArgumentError(syn::Error),
}
impl From<syn::Error> for ParameterError {
    fn from(value: syn::Error) -> Self {
        ParameterError::ArgumentError(value)
    }
}
impl From<ParameterError> for syn::Error {
    fn from(value: ParameterError) -> Self {
        match value {
            ParameterError::InexistantParameter(error)
            | ParameterError::NotParameter(error)
            | ParameterError::ArgumentError(error) => error,
        }
    }
}

pub(super) trait OptionalParameter: Sized {
    fn try_parse_parameter(input: ParseStream) -> Result<Self, ParameterError>;
    fn parse_ident(input: ParseStream) -> Result<Ident, ParameterError> {
        let ident = input
            .parse::<Ident>()
            .map_err(ParameterError::NotParameter)?;
        _ = input
            .parse::<Token![=]>()
            .map_err(ParameterError::NotParameter)?;
        Ok(ident)
    }
}
pub(super) trait Unknown {
    fn unknown<T>(&self) -> Result<T, ParameterError>;
}
impl Unknown for Ident {
    fn unknown<T>(&self) -> Result<T, ParameterError> {
        // Unknown option
        Err(ParameterError::InexistantParameter(syn::Error::new(
            self.span(),
            format!("Unknown parameter `{self}`",),
        )))
    }
}

pub(super) trait OptionalParameters<P: OptionalParameter>: Default {
    fn try_parse_parameter(&mut self, input: ParseStream) -> Result<(), ParameterError> {
        let forked_input = input.fork();
        P::try_parse_parameter(&forked_input).map(|parameter| {
            input.advance_to(&forked_input);
            self.set_param(parameter);
        })
    }
    fn set_param(&mut self, parameter: P);
}
