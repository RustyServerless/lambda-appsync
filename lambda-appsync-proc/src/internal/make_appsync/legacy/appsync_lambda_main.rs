use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parenthesized, parse::Parse, parse_macro_input, Ident, LitBool, LitStr, Token, Type};

use super::super::{
    graphql::GraphQLSchema,
    optional_parameter::{OptionalParameter, OptionalParameters, ParameterError, Unknown},
    overrides::{NameOverride, NameOverrides, OverrideParameters, TypeOverride, TypeOverrides},
};

/// A parsed AWS SDK client declaration of the form `client_fn() -> aws_sdk_foo::Client`.
struct AWSClient {
    fct_identifier: Ident,
    client_type: Type,
}
impl Parse for AWSClient {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Expected example:
        // dynamodb() -> aws_sdk_dynamodb::Client
        let fct_identifier = input.parse::<Ident>()?;
        let _empty;
        _ = parenthesized!(_empty in input);
        _ = input.parse::<Token![->]>()?;
        let client_type = input.parse::<syn::Type>()?;
        Ok(Self {
            fct_identifier,
            client_type,
        })
    }
}

impl AWSClient {
    fn is_next(input: syn::parse::ParseStream) -> bool {
        Self::parse(&input.fork()).is_ok()
    }
    fn aws_config_getter() -> TokenStream2 {
        quote! {
            static AWS_SDK_CONFIG: ::std::sync::OnceLock<::lambda_appsync::aws_config::SdkConfig> = ::std::sync::OnceLock::new();
            pub fn aws_sdk_config() -> &'static ::lambda_appsync::aws_config::SdkConfig {
                AWS_SDK_CONFIG.get().unwrap()
            }
        }
    }
    fn aws_config_init() -> TokenStream2 {
        quote! {
            AWS_SDK_CONFIG.set(::lambda_appsync::aws_config::load_from_env().await).unwrap();
        }
    }
    fn aws_client_getter(&self) -> impl ToTokens {
        let Self {
            fct_identifier,
            client_type,
        } = self;
        quote! {
            pub fn #fct_identifier() -> #client_type {
                static CLIENT: ::std::sync::OnceLock<#client_type> = ::std::sync::OnceLock::new();
                CLIENT.get_or_init(||<#client_type>::new(aws_sdk_config())).clone()
            }
        }
    }
}

/// A single parsed parameter for the legacy `appsync_lambda_main!` macro.
enum AppsyncLambdaMainParameter {
    Batch(bool),
    ExcludeLambdaHandler(bool),
    OnlyLambdaHandler(bool),
    ExcludeAppsyncTypes(bool),
    OnlyAppsyncTypes(bool),
    ExcludeAppsyncOperations(bool),
    OnlyAppsyncOperations(bool),
    Hook(Ident),
    LogInit(Ident),
    #[cfg(feature = "log")]
    EventLogging(bool),
    TypeOverride(Box<TypeOverride>),
    NameOverride(NameOverride),
}
impl OptionalParameter for AppsyncLambdaMainParameter {
    fn try_parse_parameter(input: syn::parse::ParseStream) -> Result<Self, ParameterError> {
        let ident = Self::parse_ident(input)?;
        match ident.to_string().as_str() {
            "batch" => Ok(Self::Batch(input.parse::<LitBool>()?.value())),
            "exclude_lambda_handler" => Ok(Self::ExcludeLambdaHandler(
                input.parse::<LitBool>()?.value(),
            )),
            "only_lambda_handler" => Ok(Self::OnlyLambdaHandler(input.parse::<LitBool>()?.value())),
            "exclude_appsync_types" => {
                Ok(Self::ExcludeAppsyncTypes(input.parse::<LitBool>()?.value()))
            }
            "only_appsync_types" => Ok(Self::OnlyAppsyncTypes(input.parse::<LitBool>()?.value())),
            "exclude_appsync_operations" => Ok(Self::ExcludeAppsyncOperations(
                input.parse::<LitBool>()?.value(),
            )),
            "only_appsync_operations" => Ok(Self::OnlyAppsyncOperations(
                input.parse::<LitBool>()?.value(),
            )),
            "hook" => Ok(Self::Hook(input.parse()?)),
            "log_init" => Ok(Self::LogInit(input.parse()?)),
            #[cfg(feature = "log")]
            "event_logging" => Ok(Self::EventLogging(input.parse::<LitBool>()?.value())),
            "type_override" => Ok(Self::TypeOverride(input.parse()?)),
            "name_override" => Ok(Self::NameOverride(input.parse()?)),
            // Deprecated options
            "field_type_override" => Ok(Self::TypeOverride(input.parse()?)),
            // Unknown option
            _ => ident.unknown(),
        }
    }
}

/// Accumulated parameters for the legacy `appsync_lambda_main!` macro after parsing.
struct AppsyncLambdaMainParameters {
    batch: bool,
    appsync_types: bool,
    appsync_operations: bool,
    lambda_handler: bool,
    hook: Option<Ident>,
    log_init: Option<Ident>,
    #[cfg(feature = "log")]
    event_logging: bool,
    tos: TypeOverrides,
    nos: NameOverrides,
}
impl Default for AppsyncLambdaMainParameters {
    fn default() -> Self {
        Self {
            batch: true,
            appsync_types: true,
            appsync_operations: true,
            lambda_handler: true,
            hook: None,
            log_init: None,
            #[cfg(feature = "log")]
            event_logging: false,
            tos: TypeOverrides::new(),
            nos: NameOverrides::new(),
        }
    }
}
impl OptionalParameters<AppsyncLambdaMainParameter> for AppsyncLambdaMainParameters {
    fn set_param(&mut self, p: AppsyncLambdaMainParameter) {
        match p {
            AppsyncLambdaMainParameter::Batch(batch) => self.batch = batch,
            AppsyncLambdaMainParameter::ExcludeLambdaHandler(b) if b => self.lambda_handler = false,
            AppsyncLambdaMainParameter::OnlyLambdaHandler(b) if b => {
                self.lambda_handler = true;
                self.appsync_types = false;
                self.appsync_operations = false;
            }
            AppsyncLambdaMainParameter::ExcludeAppsyncTypes(b) if b => self.appsync_types = false,
            AppsyncLambdaMainParameter::OnlyAppsyncTypes(b) if b => {
                self.lambda_handler = false;
                self.appsync_types = true;
                self.appsync_operations = false;
            }
            AppsyncLambdaMainParameter::ExcludeAppsyncOperations(b) if b => {
                self.appsync_operations = false
            }
            AppsyncLambdaMainParameter::OnlyAppsyncOperations(b) if b => {
                self.lambda_handler = false;
                self.appsync_types = false;
                self.appsync_operations = true;
            }
            AppsyncLambdaMainParameter::Hook(ident) => {
                self.hook.replace(ident);
            }
            AppsyncLambdaMainParameter::LogInit(ident) => {
                self.log_init.replace(ident);
            }
            #[cfg(feature = "log")]
            AppsyncLambdaMainParameter::EventLogging(b) => {
                self.event_logging = b;
            }
            AppsyncLambdaMainParameter::TypeOverride(to) => {
                // Retrieve the entry corresponding to `Type.field`
                let to_field_entry = self
                    .tos
                    .entry(to.type_name().to_string())
                    .or_default()
                    .entry(to.field_name().to_string())
                    .or_default();
                if let Some(arg_name) = to.arg_name() {
                    // There is a `.param`
                    // This is a parameter override
                    to_field_entry.1.insert(arg_name.to_string(), *to);
                } else {
                    // no `.param`
                    // This is just a field override
                    to_field_entry.0.replace(*to);
                }
            }
            AppsyncLambdaMainParameter::NameOverride(no) => {
                // Retrieve the entry corresponding to `Type`
                let no_type_entry = self.nos.entry(no.type_name().to_string()).or_default();
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
            AppsyncLambdaMainParameter::ExcludeLambdaHandler(_)
            | AppsyncLambdaMainParameter::OnlyLambdaHandler(_)
            | AppsyncLambdaMainParameter::ExcludeAppsyncTypes(_)
            | AppsyncLambdaMainParameter::OnlyAppsyncTypes(_)
            | AppsyncLambdaMainParameter::ExcludeAppsyncOperations(_)
            | AppsyncLambdaMainParameter::OnlyAppsyncOperations(_) => (),
        }
    }
}

/// Fully parsed input for the legacy `appsync_lambda_main!` macro, combining schema, AWS clients, and options.
struct AppsyncLambdaMain {
    graphql_schema: GraphQLSchema,
    aws_clients: Vec<AWSClient>,
    options: AppsyncLambdaMainParameters,
}

impl Parse for AppsyncLambdaMain {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let graphql_schema_path = input.parse::<LitStr>()?;

        let mut parameters = AppsyncLambdaMainParameters::default();
        let mut aws_clients = vec![];

        while input.peek(Token![,]) {
            _ = input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            match parameters.try_parse_parameter(input) {
                // Matched, so go on to the next one
                Ok(()) => continue,
                Err(pe) => match pe {
                    // Not a parameter, try matching an AWS Client
                    ParameterError::NotParameter(_) => {}
                    // Hard errors, return
                    ParameterError::InexistantParameter(error)
                    | ParameterError::ArgumentError(error) => return Err(error),
                },
            }
            if AWSClient::is_next(input) {
                aws_clients.push(input.parse::<AWSClient>()?);
            } else {
                return Err(syn::Error::new(input.span(), "Unknown argument"));
            }
        }

        let graphql_schema = GraphQLSchema::new(
            graphql_schema_path,
            OverrideParameters {
                type_overrides: std::mem::take(&mut parameters.tos),
                name_overrides: std::mem::take(&mut parameters.nos),
            },
            Default::default(),
            Default::default(),
        )?;

        Ok(Self {
            graphql_schema,
            aws_clients,
            options: parameters,
        })
    }
}

impl AppsyncLambdaMain {
    fn appsync_event_handler(&self, tokens: &mut TokenStream2) {
        #[allow(unused_mut)]
        let mut log_lines = proc_macro2::TokenStream::new();
        #[cfg(feature = "log")]
        if self.options.event_logging {
            log_lines.extend(quote! {
                ::lambda_appsync::log::debug!("event={event:?}");
            });
        }
        #[cfg(feature = "log")]
        log_lines.extend(quote! {
            ::lambda_appsync::log::info!("operation={:?}", event.info.operation);
        });

        let call_hook = if let Some(ref hook) = self.options.hook {
            quote_spanned! {hook.span()=>
                mod _check_sig {
                    use super::Operation;
                    use ::lambda_appsync::{AppsyncEvent, AppsyncResponse};
                    use ::core::future::Future;
                    #[inline(always)]
                    pub(super) async fn call_hook<'a, Fut, H>(hook: H, event: &'a AppsyncEvent<Operation>) -> Option<AppsyncResponse>
                    where
                        Fut: Future<Output = Option<AppsyncResponse>>,
                        H: Fn(&'a AppsyncEvent<Operation>) -> Fut {
                        hook(event).await
                    }
                }
                if let Some(resp) = _check_sig::call_hook(#hook, &event).await{
                    return resp;
                }
            }
        } else {
            quote! {}
        };

        #[cfg(feature = "tracing")]
        tokens.extend(quote! {
            #[::lambda_appsync::tracing::instrument(skip(event), fields(operation = ?event.info.operation))]
        });

        tokens.extend(quote! {
            async fn appsync_handler(event: ::lambda_appsync::AppsyncEvent<Operation>) -> ::lambda_appsync::AppsyncResponse {
                #log_lines

                #call_hook

                event.info.operation.execute(event).await
            }
        });
        if self.options.batch {
            tokens.extend(quote! {
                async fn appsync_batch_handler(
                    events: ::std::vec::Vec<::lambda_appsync::AppsyncEvent<Operation>>,
                ) -> ::std::vec::Vec<::lambda_appsync::AppsyncResponse> {
                    let handles = events
                        .into_iter()
                        .map(|e| ::lambda_appsync::tokio::spawn(appsync_handler(e)))
                        .collect::<::std::vec::Vec<_>>();

                    let mut results = vec![];
                    for h in handles {
                        results.push(h.await.unwrap())
                    }
                    results
                }

            });
        }
    }

    #[allow(dead_code)]
    fn default_env_logger_init() -> TokenStream2 {
        quote! {
            ::lambda_appsync::env_logger::Builder::from_env(
                ::lambda_appsync::env_logger::Env::default()
                    .default_filter_or("info,tracing::span=warn")
                    .default_write_style_or("never"),
            )
            .format_timestamp_micros()
            .init();
        }
    }

    #[allow(dead_code)]
    fn default_tracing_init() -> TokenStream2 {
        quote! {
            ::lambda_appsync::tracing_subscriber::fmt()
                    .json()
                    .with_env_filter(
                        ::lambda_appsync::tracing_subscriber::EnvFilter::from_default_env()
                            .add_directive(tracing::Level::INFO.into()),
                    )
                    // this needs to be set to remove duplicated information in the log.
                    .with_current_span(false)
                    // this needs to be set to false, otherwise ANSI color codes will
                    // show up in a confusing manner in CloudWatch logs.
                    .with_ansi(false)
                    // remove the name of the function from every log entry
                    .with_target(false)
                    .init();
        }
    }

    fn lambda_function_handler(&self, tokens: &mut TokenStream2) {
        let (appsync_handler, ret_type) = if self.options.batch {
            (
                format_ident!("appsync_batch_handler"),
                quote! {::std::vec::Vec<::lambda_appsync::AppsyncResponse>},
            )
        } else {
            (
                format_ident!("appsync_handler"),
                quote! {::lambda_appsync::AppsyncResponse},
            )
        };

        #[cfg(feature = "tracing")]
        tokens.extend(quote! {
            #[::lambda_appsync::tracing::instrument(skip(event), fields(req_id = %event.context.request_id))]
        });

        #[allow(unused_mut)]
        let mut log_lines = proc_macro2::TokenStream::new();
        #[cfg(feature = "log")]
        if self.options.event_logging {
            log_lines.extend(quote! {
                ::lambda_appsync::log::debug!("{}", ::lambda_appsync::serde_json::json!(event.payload));
            });
        }

        tokens.extend(quote! {
            async fn function_handler(
                event: ::lambda_appsync::lambda_runtime::LambdaEvent<::lambda_appsync::serde_json::Value>,
            ) -> ::core::result::Result<#ret_type, ::lambda_appsync::lambda_runtime::Error> {
                #log_lines
                Ok(#appsync_handler(::lambda_appsync::serde_json::from_value(event.payload)?).await)
            }
        });
    }

    fn lambda_main(&self, tokens: &mut TokenStream2) {
        let (config_init, config_getter) = if !self.aws_clients.is_empty() {
            (AWSClient::aws_config_init(), AWSClient::aws_config_getter())
        } else {
            (TokenStream2::new(), TokenStream2::new())
        };
        let aws_client_getters = self.aws_clients.iter().map(|ac| ac.aws_client_getter());

        let log_init = if let Some(ref log_init) = self.options.log_init {
            quote_spanned! {log_init.span()=>
                mod _check_sig {
                    #[inline(always)]
                    pub(super) fn call_log_init<F: Fn() -> ()>(f: F) {f()}
                }
                _check_sig::call_log_init(#log_init);
            }
        } else {
            #[allow(unused_mut)]
            let mut default_log_init = proc_macro2::TokenStream::new();
            #[cfg(feature = "env_logger")]
            default_log_init.extend(Self::default_env_logger_init());
            // The code initializing tracing fails if the env_logger initialization already happened
            #[cfg(all(feature = "tracing", not(any(feature = "env_logger"))))]
            default_log_init.extend(Self::default_tracing_init());
            // Future default inits can be inserted here like that for feature "fastrace" (for example):
            // #[cfg(all(feature = "fastrace", not(any(feature = "env_logger", feature = "tracing"))))]
            // default_log_init.extend(Self::default_fastrace_init());
            default_log_init
        };

        #[allow(unused_mut)]
        let mut bring_in_scope = TokenStream2::new();
        bring_in_scope.extend(quote! {
            use ::lambda_appsync::tokio;
        });
        #[cfg(feature = "tracing")]
        bring_in_scope.extend(quote! {
            use ::lambda_appsync::tracing;
        });

        tokens.extend(quote! {

            #config_getter

            #(#aws_client_getters)*

            #bring_in_scope

            #[tokio::main]
            async fn main() -> ::core::result::Result<(), ::lambda_appsync::lambda_runtime::Error> {
                #log_init

                #config_init

                ::lambda_appsync::lambda_runtime::run(::lambda_appsync::lambda_runtime::service_fn(function_handler)).await
            }
        });
    }
}

impl ToTokens for AppsyncLambdaMain {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.options.appsync_types {
            self.graphql_schema.appsync_types_to_tokens(tokens);
        }
        if self.options.appsync_operations {
            self.graphql_schema.appsync_operations_to_tokens(tokens);
        }
        if self.options.lambda_handler {
            self.appsync_event_handler(tokens);
            self.lambda_function_handler(tokens);
            self.lambda_main(tokens);
        }
    }
}

/// Entry point for the legacy `appsync_lambda_main!` proc-macro implementation.
pub(crate) fn appsync_lambda_main_impl(input: TokenStream) -> TokenStream {
    let alm = parse_macro_input!(input as AppsyncLambdaMain);
    alm.into_token_stream().into()
}
