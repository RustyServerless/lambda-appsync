# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2026-01-09

### Added
- **Tracing support**: New `tracing` feature flag enables tracing/tracing-subscriber integration as an alternative to env_logger
- **Configurable logging initialization**: New `log_init = function_name` macro parameter allows custom log initialization functions instead of relying on defaults
- **Event logging control**: New `event_logging = bool` macro parameter controls whether Lambda event payloads are logged (defaults to `false` for better security)
- Structured JSON logging by default when tracing is enabled
- `#[tracing::instrument]` attributes on generated handler functions with request ID and operation name in trace spans
- Signature validation for the `hook` option with clear error messages for invalid function signatures

### Changed
- **Breaking**: Event payload logging is now disabled by default for security reasons (can be re-enabled with `event_logging = true`)
- When Lambda event payloads logging is enabled, Lambda event payloads are now logged at `debug` level instead of `info`
- `env_logger` is now an optional feature (still enabled by default for backward compatibility)
- Independent `log` feature flag controls log statement generation in generated code

### Fixed
- Generated `main` and `function_handler` now use fully qualified `::core::result::Result` instead of `Result`, preventing compilation failures when users override the Result type
- Request hook trybuild test was not actually testing anything
- Release CI could fail tests but still proceed with publishing

[0.9.0]: https://github.com/RustyServerless/lambda-appsync/compare/v0.8.0...v0.9.0

## [0.8.0] - 2025-11-16

### Changed
- Updated lambda_runtime dependency minimum version from 0.12 to 1.0, ensuring use of the version officially supported by the AWS Lambda team
- Bumped Minimum Supported Rust Version (MSRV) from 1.81.0 to 1.82.0

[0.8.0]: https://github.com/RustyServerless/lambda-appsync/compare/v0.7.0...v0.8.0

## [0.7.0] - 2025-01-26

### Added
- `Serialize` derivation for GraphQL input types, enabling serialization when building client libraries for the GraphQL API
- `FromStr` implementation for all string-like GraphQL scalar types (`ID`, and other AWS scalar types), enabling parsing from string slices:
  ```rust
  let id: ID = "123e4567-e89b-12d3-a456-426614174000".parse().unwrap();
  ```

### Fixed
- MSRV compatibility by pinning base64ct to v1.7.3

### Breaking Changes
- GraphQL input types now derive both `Serialize` and `Deserialize` instead of only `Deserialize`. This may affect code that had a manual implementation of `Serialize` for input types.

[0.7.0]: https://github.com/RustyServerless/lambda-appsync/compare/v0.6.0...v0.7.0

## [0.6.1] - 2025-08-24 [YANKED]

### Added
- `FromStr` implementation for all string-like GraphQL scalar types (`ID`, and other AWS scalar types), enabling parsing from string slices:
  ```rust
  let id: ID = "123e4567-e89b-12d3-a456-426614174000".parse().unwrap();
  ```

### Fixed
- Documentation badge in README now correctly links to the latest version instead of hardcoded v0.1.0

[0.6.1]: https://github.com/RustyServerless/lambda-appsync/compare/v0.6.0...v0.6.1

## [0.6.0] - 2025-04-21

### Added
- New `name_override` option for the `appsync_lambda_main!` macro allowing fine-grained control over generated Rust names:
  ```rust
  appsync_lambda_main!(
      "schema.graphql",
      // Override type names, the struct will be called `GqlPlayer` in Rust
      name_override = Player: GqlPlayer,
      // Override field names, `name` becomes `email` in Rust
      name_override = Player.name: email,
      // Override enum variants, `Python` becomes `Snake`
      name_override = Team.PYTHON: Snake,
      // Handle Rust keywords gracefully, rename to `kind`
      name_override = WeirdFieldNames.type: kind,
  );
  ```

  Do not forget to also override operation return types if you change type names!
- Comprehensive end-to-end integration tests for both batch and non-batch configurations, ensuring non-regression of the full AppSync request processing.

### Changed
- **Major improvement**: Completely revamped compilation error messages for invalid operation handlers:
  - Error messages now point directly to the problematic code instead of the macro invocation
  - Clear indication of expected vs found types for arguments and return values
  - More accurate function signature mismatch reporting

  See [Issue#7](https://github.com/RustyServerless/lambda-appsync/issues/7) for detailed examples of the improved error messages.
- Internal restructuring of code generation to use trait bounds for signature verification instead of dummy function calls

### Fixed
- Compilation error messages that previously showed inverse type expectations (e.g., for return type mismatches)
- Various edge cases in error reporting related to argument types and counts

### Breaking Changes
- Removed `pub` visibility from `Operation::execute()` as it was not intended for direct use
- Moved some internal types to new module structure. While these weren't meant for direct use, code explicitly referencing them may need updates
- The improved error reporting system may affect existing trybuild tests that were matching against previous error messages

### Internal
- Better module organization with clearer separation between public and internal APIs
- Increased generated code size (approximately 2x) to support better error messages. While this may slightly increase compilation times, it has no runtime performance impact as the additional code is optimized away during compilation.

[0.6.0]: https://github.com/RustyServerless/lambda-appsync/compare/v0.5.4...v0.6.0

## [v0.5.4] - 2025-04-24

### Fixed
- Bug where the `type_override` option was not working correctly for fields or arguments that were Rust keywords

[v0.5.4]: https://github.com/RustyServerless/lambda-appsync/compare/v0.5.3...v0.5.4

## [v0.5.3] - 2025-04-24

### Changed
- Deprecated `field_type_override` option in favor of `type_override` to better reflect its broader scope covering both fields and arguments
- (For project devs) Improved internal implementation of type override options for better clarity

[v0.5.3]: https://github.com/RustyServerless/lambda-appsync/compare/v0.5.2...v0.5.3

## [v0.5.2] - 2025-04-23

### Added
- Extended type override capabilities to support operation return types and arguments
- Integration tests to validate macro behavior with different argument sets

### Changed
- Improved error messages for schema file not found scenarios to better explain path resolution

### Fixed
- Bug in `keep_original_function_name` attribute where it would fail when operation handlers had arguments

[v0.5.2]: https://github.com/RustyServerless/lambda-appsync/compare/v0.5.1...v0.5.2

## [v0.5.1] - 2025-04-21

### Changed
- Improved internal documentation linking structure between crates

### Fixed
- Documentation linking issues between lambda-appsync and lambda-appsync-proc crates

[v0.5.1]: https://github.com/RustyServerless/lambda-appsync/compare/v0.5.0...v0.5.1

## [v0.5.0] - 2025-04-21

### Added
- New option to get a reference to the AppsyncEvent structure in operation handlers through the `with_appsync_event` attribute parameter
- Comprehensive support for all AWS AppSync auth types (API Key, Cognito User Pools, IAM, OpenID Connect, Lambda)

### Changed
- **Breaking**: Complete rewrite of the `AppsyncIdentity` type to support all AWS AppSync auth modes (previously only supported Cognito)
- Better error handling for auth types serialization/deserialization
- Improved documentation links and clarity about workspace-relative paths

### Fixed
- Missing documentation links in the crate documentation

[v0.5.0]: https://github.com/RustyServerless/lambda-appsync/compare/v0.4.2...v0.5.0

## [v0.4.2] - 2025-04-19

### Fixed
- Clippy warnings and code style improvements

[v0.4.2]: https://github.com/RustyServerless/lambda-appsync/compare/v0.4.1...v0.4.2

## [v0.4.1] - 2025-04-19 [YANKED]

### Changed
- Enhanced test coverage for null handling in GraphQL schema type generation

[v0.4.1]: https://github.com/RustyServerless/lambda-appsync/compare/v0.4.0...v0.4.1

## [v0.4.0] - 2025-04-19 [YANKED]

### Added
- Comprehensive integration tests for macro-generated structure serialization/deserialization

### Changed
- **Breaking**: Changed GraphQL Int scalar from i64 to i32 to comply with GraphQL specification
- Improved handling of Rust keywords in GraphQL field names (proper escaping with 'r#' prefix)

### Fixed
- Proper handling of Rust keywords when used as field names in GraphQL schema
- Documentation examples and subscription filter code blocks
- README referenced crate version

[v0.4.0]: https://github.com/RustyServerless/lambda-appsync/compare/v0.3.0...v0.4.0

## [v0.3.0] - 2025-04-04

### Added
- Support for AWS AppSync Enhanced Subscription Filters
- New types for subscription filtering: `FilterGroup`, `Filter`, and `FieldPath`
- Convenience conversions between filter types
- Documentation for subscription filters usage and configuration

### Changed
- **Breaking**: `AppsyncIdentity` structure now has optional groups field for Cognito identity
- DefaultOperation generator now returns optional FilterGroup for subscriptions
- Improved documentation with compiled examples

### Fixed
- Case sensitivity issues with GraphQL schema field names
- Double logging of AppSync Operations
- Memory allocation optimization
- Documentation improvements

[v0.3.0]: https://github.com/RustyServerless/lambda-appsync/compare/v0.2.0...v0.3.0

## [v0.2.0] - 2025-04-01

### Added
- New `unauthorized` constructor for AppsyncResponse struct
- Several convenience implementations for AWSTimestamp:
  - AddAssign<Duration>
  - SubAssign<Duration>
  - Display formatting
  - PartialEq
  - into_u64/from_u64 methods

### Changed
- Improved documentation with compiled examples
- Reorganized workspace dependencies

[v0.2.0]: https://github.com/RustyServerless/lambda-appsync/compare/v0.1.0...v0.2.0

## [v0.1.0] - 2025-03-30

### Added
- Initial release with core functionality
- Type-safe GraphQL schema conversion to Rust types
- Complete AWS Lambda runtime integration
- Built-in validation of resolver function signatures
- AWS SDK client initialization support
- AWS SDK error built-in conversion
- Batching support for improved performance
- Optional request validation hook
- Support for custom type overrides
- Basic examples and documentation

[v0.1.0]: https://github.com/RustyServerless/lambda-appsync/releases/tag/v0.1.0
