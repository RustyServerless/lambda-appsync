use syn::{
    parse::{discouraged::Speculative, ParseStream},
    Ident, Token,
};

/// Error returned when attempting to parse an optional macro parameter.
pub(crate) enum ParameterError {
    /// The token stream does not match any known parameter name.
    InexistantParameter(syn::Error),
    /// The token stream does not look like a parameter at all (e.g. missing `=`).
    NotParameter(syn::Error),
    /// The parameter name matched but its value failed to parse.
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

/// Parses a single named optional parameter of the form `name = value` from a token stream.
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
/// Extension for [`struct@Ident`] to produce an [`ParameterError::InexistantParameter`] error.
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

/// Accumulates multiple [`OptionalParameter`]s into a parameters struct, using speculative parsing.
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
