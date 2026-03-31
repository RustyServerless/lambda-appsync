<!-- PROJECT SHIELDS -->
[![crates.io](https://img.shields.io/crates/v/lambda-appsync.svg)](https://crates.io/crates/lambda-appsync)
[![docs.rs](https://docs.rs/lambda-appsync/badge.svg)](https://docs.rs/lambda-appsync/latest/lambda_appsync)
[![CI](https://github.com/RustyServerless/lambda-appsync/workflows/CI/badge.svg)](https://github.com/RustyServerless/lambda-appsync/actions)
[![License](https://img.shields.io/github/license/RustyServerless/lambda-appsync.svg)](https://github.com/RustyServerless/lambda-appsync/blob/master/LICENSE)

# lambda-appsync

A type-safe framework for AWS AppSync Direct Lambda resolvers — generates Rust types, operation dispatch, and Lambda runtime glue from a GraphQL schema at compile time.

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#about">About</a></li>
    <li><a href="#features">Features</a></li>
    <li><a href="#known-limitations">Known Limitations</a></li>
    <li><a href="#installation">Installation</a></li>
    <li><a href="#quick-start">Quick Start</a></li>
    <li><a href="#example-project">Example Project</a></li>
    <li><a href="#additional-examples">Additional Examples</a></li>
    <li><a href="#macro-reference">Macro Reference</a></li>
    <li><a href="#feature-flags">Feature Flags</a></li>
    <li><a href="#migrating-from-appsync_lambda_main">Migrating from appsync_lambda_main!</a></li>
    <li><a href="#faq">FAQ</a></li>
    <li><a href="#minimum-supported-rust-version-msrv">MSRV</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#authors">Authors</a></li>
  </ol>
</details>

## About

The `lambda-appsync` crate provides procedural macros that read a GraphQL schema file at compile time and generate:

- **Rust types** for all GraphQL objects, inputs, and enums
- An **`Operation` enum** covering every query, mutation, and subscription field, with argument extraction and dispatch logic
- A **`Handlers` trait** and `DefaultHandlers` struct for wiring up the AWS Lambda runtime

You write resolver functions annotated with `#[appsync_operation(...)]` and the framework validates their signatures against the schema, handles deserialization, and produces properly formatted AppSync responses.

## Features

- ✨ Type-safe GraphQL schema conversion to Rust types at compile time
- 🔒 Compile-time validation of resolver function signatures against the schema
- 🚀 Full AWS Lambda runtime integration with batch support
- 🔔 AWS AppSync enhanced subscription filters
- 🔐 Comprehensive support for all AWS AppSync auth types (Cognito, IAM, OIDC, Lambda, API Key)
- 📦 Composable macro architecture — use `make_appsync!` for simple setups or `make_types!` / `make_operation!` / `make_handlers!` individually for multi-crate projects
- 🛡️ Customizable request handling via the `Handlers` trait (authentication hooks, logging, etc.)
- 🔧 Type and name overrides for fine-grained control over generated code
- 📊 Configurable logging with `log`/`env_logger` and `tracing` support
- 🧩 `const fn` enum utilities (`all()`, `index()`, `COUNT`) for generated enums

## Known Limitations

- GraphQL **unions** are not supported and will be ignored by the macro
- GraphQL **interfaces** are not directly supported, though concrete types that implement interfaces will work correctly
- **Arguments in fields** of non-operation types (i.e. not Query, Mutation, or Subscription) are ignored by the macro

If your project requires union or interface support, or you have ideas on how the macro could use field arguments for regular types, please [open a GitHub issue](https://github.com/RustyServerless/lambda-appsync/issues/new) detailing your use case.

## Installation

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
lambda-appsync = "0.10.0"
```

Or using cargo:

```sh
cargo add lambda-appsync
```

> **Note:** Starting with v0.10.0, the default feature set is empty. The crate works out of the box without any features. Enable `env_logger`, `tracing`, or `log` only if you need their re-exports or logging integration. See [Feature Flags](#feature-flags) for details.

## Quick Start

1. Create your GraphQL schema file (e.g. `schema.graphql`):

```graphql
type Query {
  players: [Player!]!
  gameStatus: GameStatus!
}

type Player {
  id: ID!
  name: String!
  team: Team!
}

enum Team {
  RUST
  PYTHON
  JS
}

enum GameStatus {
  STARTED
  STOPPED
}
```

2. Generate types, operations, and handlers from the schema:

```rust
use lambda_appsync::{make_appsync, appsync_operation, AppsyncError};

// Generate everything from the schema
make_appsync!("schema.graphql");

// Implement resolver functions:

#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    todo!()
}

#[appsync_operation(query(gameStatus))]
async fn get_game_status() -> Result<GameStatus, AppsyncError> {
    todo!()
}

// Wire up the Lambda runtime:

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(
        lambda_runtime::service_fn(DefaultHandlers::service_fn)
    ).await
}
```

When in a workspace context, all relative schema paths are resolved relative to the workspace root directory.

The framework's macros verify that your function signatures match the GraphQL schema at compile time and automatically wire everything up to handle AWS AppSync requests.

## Example Project

Check out the [benchmark-game](https://github.com/RustyServerless/benchmark-game) sample project that demonstrates lambda-appsync in action. This full-featured demo implements a GraphQL API for a mini-game web application using AWS AppSync and Lambda, showcasing:

- 🎮 Real-world GraphQL schema
- 📊 DynamoDB integration
- 🏗️ Infrastructure as code with AWS CloudFormation
- 🚀 CI/CD pipeline configuration

Clone the repo to get started with a production-ready template for building serverless GraphQL APIs with Rust.

## Additional Examples

### Custom Handler with Authentication Hook

Override the `Handlers` trait to add pre-processing logic such as authentication checks:

```rust
use lambda_appsync::{make_appsync, appsync_operation, AppsyncError};
use lambda_appsync::{AppsyncEvent, AppsyncResponse, AppsyncIdentity};

make_appsync!("schema.graphql");

struct MyHandlers;
impl Handlers for MyHandlers {
    async fn appsync_handler(event: AppsyncEvent<Operation>) -> AppsyncResponse {
        // Custom authentication check
        if let AppsyncIdentity::ApiKey = &event.identity {
            return AppsyncResponse::unauthorized();
        }
        // Delegate to the default operation dispatch
        event.info.operation.execute(event).await
    }
}

#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(
        lambda_runtime::service_fn(MyHandlers::service_fn)
    ).await
}
```

This replaces the old `hook` parameter from `appsync_lambda_main!`. See the [`make_handlers!` documentation](https://docs.rs/lambda-appsync/latest/lambda_appsync/macro.make_handlers.html) for the full migration table.

### Composable Macros for Multi-Crate Projects

For larger projects where you share GraphQL types across multiple Lambda functions, use the individual macros:

```rust
use lambda_appsync::{make_types, make_operation, make_handlers, appsync_operation, AppsyncError};

// Step 1: Generate types (could live in a shared lib crate)
make_types!("schema.graphql");

// Step 2: Generate Operation enum and dispatch logic
make_operation!("schema.graphql");

// Step 3: Generate Handlers trait and DefaultHandlers
make_handlers!();

#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(
        lambda_runtime::service_fn(DefaultHandlers::service_fn)
    ).await
}
```

A typical multi-crate layout:

```text
my-project/
├── schema.graphql
├── shared-types/     # lib crate using make_types!
│   └── src/lib.rs
├── lambda-a/         # bin crate using make_operation! + make_handlers!
│   └── src/main.rs
└── lambda-b/         # another bin crate
    └── src/main.rs
```

When types live in a different module, use the `type_module` parameter on `make_operation!`:

```rust
make_operation!(
    "schema.graphql",
    type_module = crate::types,
);
```

### Type and Name Overrides

Override generated Rust types or names for specific GraphQL elements:

```rust
make_appsync!(
    "schema.graphql",
    // Override a struct field type
    type_override = Player.id: String,
    // Override operation argument and return types to match
    type_override = Query.player.id: String,
    type_override = Mutation.deletePlayer.id: String,
    // Rename a type (must also override operation return types!)
    name_override = Player: GqlPlayer,
    type_override = Query.players: GqlPlayer,
    type_override = Query.player: GqlPlayer,
    type_override = Mutation.createPlayer: GqlPlayer,
    type_override = Mutation.deletePlayer: GqlPlayer,
);
```

### Controlling Default Traits and Derives

Control which traits are derived on generated types:

```rust
make_appsync!(
    "schema.graphql",
    // Disable all default traits for Player — you implement them yourself
    default_traits = Player: false,
    // Add specific derives back
    derive = Player: Debug,
    derive = Player: serde::Serialize,
    // Add extra derives on top of defaults for another type
    derive = Team: Default,
);
```

Default traits for structs: `Debug`, `Clone`, `Serialize`, `Deserialize`.
Default traits for enums: `Debug`, `Clone`, `Copy`, `Serialize`, `Deserialize`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Display`, `FromStr`.

### Subscription Filters

Build type-safe [enhanced subscription filters](https://docs.aws.amazon.com/appsync/latest/devguide/aws-appsync-real-time-enhanced-filtering.html):

```rust
use lambda_appsync::{appsync_operation, AppsyncError};
use lambda_appsync::subscription_filters::{FieldPath, FilterGroup};

#[appsync_operation(subscription(onCreatePlayer))]
async fn on_create_player(name: String) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(FieldPath::new("name")?.contains(name).into()))
}
```

> **Important:** When using enhanced subscription filters, your AppSync **Response** mapping template must contain:
> ```vtl
> #if($context.result.data)
> $extensions.setSubscriptionFilter($context.result.data)
> #end
> null
> ```

### Accessing the AppSync Event

Access the full AppSync event context in operation handlers with the `with_appsync_event` flag:

```rust
use lambda_appsync::{appsync_operation, AppsyncError, AppsyncEvent, AppsyncIdentity};

#[appsync_operation(mutation(createPlayer), with_appsync_event)]
async fn create_player(
    name: String,
    event: &AppsyncEvent<Operation>
) -> Result<Player, AppsyncError> {
    let user_id = if let AppsyncIdentity::Cognito(cognito) = &event.identity {
        cognito.sub.clone()
    } else {
        return Err(AppsyncError::new("Unauthorized", "Must be Cognito authenticated"));
    };
    todo!()
}
```

### Original Function Preservation

By default, `#[appsync_operation]` preserves the original function alongside the generated `impl Operation` method. You can call it directly elsewhere in your code:

```rust
#[appsync_operation(query(players))]
async fn fetch_players() -> Result<Vec<Player>, AppsyncError> {
    todo!()
}

// fetch_players() is still available as a regular function
```

If you want the function to be removed (its body is inlined into the generated method), use `inline_and_remove`. This can be handy when generating handlers with `macro_rules!`, where you don't want the function name to collide with itself:

```rust
macro_rules! game_status_mut {
    ($mut_name:ident, $status:path) => {
        #[appsync_operation(mutation($mut_name), inline_and_remove)]
        pub async fn _discarded() -> Result<GameStatus, AppsyncError> {
            dynamodb_set_game_status($status).await?;
            Ok($status)
        }
    };
}

game_status_mut!(startGame, GameStatus::Started);
game_status_mut!(stopGame, GameStatus::Stopped);
game_status_mut!(resetGame, GameStatus::Reset);
```

> **Note:** When the `compat` feature is enabled, the old default behavior is restored (inline and remove), and the `keep_original_function_name` parameter is available to opt back into preservation.

### AWS SDK Error Support

AWS SDK errors are automatically converted to `AppsyncError`, allowing use of the `?` operator:

```rust
async fn store_item(item: Item, client: &aws_sdk_dynamodb::Client) -> Result<(), AppsyncError> {
    client.put_item()
        .table_name("my-table")
        .item("id", AttributeValue::S(item.id.to_string()))
        .item("data", AttributeValue::S(item.data))
        .send()
        .await?;
    Ok(())
}
```

### Error Merging

Combine multiple errors using the pipe operator:

```rust
let err = AppsyncError::new("ValidationError", "Invalid email")
    | AppsyncError::new("DatabaseError", "User not found");
// error_type: "ValidationError|DatabaseError"
// error_message: "Invalid email\nUser not found in database"
```

## Macro Reference

| Macro | Kind | Purpose |
|---|---|---|
| [`make_appsync!`](https://docs.rs/lambda-appsync/latest/lambda_appsync/macro.make_appsync.html) | All-in-one | Generate types, `Operation` enum, and `Handlers` trait from a schema |
| [`make_types!`](https://docs.rs/lambda-appsync/latest/lambda_appsync/macro.make_types.html) | Composable | Generate Rust structs and enums from schema type definitions |
| [`make_operation!`](https://docs.rs/lambda-appsync/latest/lambda_appsync/macro.make_operation.html) | Composable | Generate the `Operation` enum and dispatch logic |
| [`make_handlers!`](https://docs.rs/lambda-appsync/latest/lambda_appsync/macro.make_handlers.html) | Composable | Generate the `Handlers` trait and `DefaultHandlers` struct |
| [`#[appsync_operation]`](https://docs.rs/lambda-appsync/latest/lambda_appsync/attr.appsync_operation.html) | Attribute | Bind an async function to a specific GraphQL operation |
| `appsync_lambda_main!` | Legacy (`compat`) | Deprecated monolithic macro — prefer `make_appsync!` for new code |

For detailed syntax, options, and examples for each macro, see the [API documentation on docs.rs](https://docs.rs/lambda-appsync/latest/lambda_appsync).

### When to Use Which Macro

| Scenario | Recommendation |
|----------|---------------|
| Single Lambda function, all code in one crate | `make_appsync!` |
| Shared types library + multiple Lambda binaries | `make_types!` in the lib, `make_operation!` + `make_handlers!` in each binary |
| Custom handler logic only | `make_appsync!` + override `Handlers` trait methods |
| Need different operations per Lambda | `make_types!` shared, separate `make_operation!` per Lambda |

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `compat` | ❌ | Enables the deprecated `appsync_lambda_main!` macro and re-exports `aws_config`. Not required for `make_appsync!` or the composable macros. |
| `log` | ❌ | Re-exports the [`log`](https://docs.rs/log) crate so resolver code can use `log::info!` etc. without a separate dependency. Enables `log::error!` in generated dispatch code. |
| `env_logger` | ❌ | Re-exports `env_logger` for local development. Implies `log` and `compat`. |
| `tracing` | ❌ | Re-exports `tracing` and `tracing-subscriber`. When enabled, the generated `Handlers` trait wraps each event dispatch in a `tracing::info_span!` for per-operation observability. |

```toml
# No features needed for basic usage
lambda-appsync = "0.10.0"

# Enable tracing instrumentation
lambda-appsync = { version = "0.10.0", features = ["tracing"] }

# Enable log + env_logger (similar to pre-0.10 defaults)
lambda-appsync = { version = "0.10.0", features = ["env_logger"] }

# Use both tracing and env_logger (migration scenarios)
lambda-appsync = { version = "0.10.0", features = ["env_logger", "tracing"] }

# Just the log crate re-export, bring your own logger
lambda-appsync = { version = "0.10.0", features = ["log"] }
```

## Migrating from `appsync_lambda_main!`

The legacy `appsync_lambda_main!` macro (now behind the `compat` feature) combined type generation, operation dispatch, Lambda runtime setup, logging initialization, and AWS SDK client initialization into a single call. The new architecture splits these concerns, giving you explicit control over each part.

### Before (v0.9)

```rust
use lambda_appsync::{appsync_lambda_main, appsync_operation, AppsyncError};
use lambda_appsync::{AppsyncEvent, AppsyncResponse, AppsyncIdentity};

fn custom_log_init() {
    use lambda_appsync::env_logger;
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or("info")
            .default_write_style_or("never"),
    )
    .format_timestamp_micros()
    .init();
}

async fn auth_hook(
    event: &AppsyncEvent<Operation>,
) -> Option<AppsyncResponse> {
    if let AppsyncIdentity::ApiKey = &event.identity {
        return Some(AppsyncResponse::unauthorized());
    }
    None
}

appsync_lambda_main!(
    "schema.graphql",
    hook = auth_hook,
    log_init = custom_log_init,
    event_logging = true,
    dynamodb() -> aws_sdk_dynamodb::Client,
    s3() -> aws_sdk_s3::Client,
);

#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    let client = dynamodb();
    todo!()
}
```

### After (v0.10)

```rust
use lambda_appsync::{make_appsync, appsync_operation, AppsyncError};
use lambda_appsync::{AppsyncEvent, AppsyncResponse, AppsyncIdentity};

// 1. Generate types, Operation enum, and Handlers trait
make_appsync!("schema.graphql");

// 2. Hook → custom Handlers impl
struct MyHandlers;
impl Handlers for MyHandlers {
    async fn appsync_handler(event: AppsyncEvent<Operation>) -> AppsyncResponse {
        // Auth check (was the `hook` parameter)
        if let AppsyncIdentity::ApiKey = &event.identity {
            return AppsyncResponse::unauthorized();
        }
        event.info.operation.execute(event).await
    }
}

// 3. Resolver functions — unchanged
#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    let client = dynamodb();
    todo!()
}

// 4. main() — you own the runtime, logging, and SDK clients
#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // log_init → call directly in main
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or("info")
            .default_write_style_or("never"),
    )
    .format_timestamp_micros()
    .init();

    // AWS SDK clients → initialize directly
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let _dynamodb = aws_sdk_dynamodb::Client::new(&config);
    let _s3 = aws_sdk_s3::Client::new(&config);

    // event_logging → add logging in your Handlers impl or here

    lambda_runtime::run(
        lambda_runtime::service_fn(MyHandlers::service_fn)
    ).await
}
```

### Migration cheat sheet

| `appsync_lambda_main!` option | New approach |
|-------------------------------|-------------|
| `hook = fn_name` | Override `Handlers::appsync_handler` |
| `log_init = fn_name` | Call your init function in `main()` |
| `event_logging = true` | Add logging in your `Handlers` impl or `main()` |
| `dynamodb() -> Client` | Initialize AWS SDK clients in `main()` |
| `only_appsync_types = true` | Use `make_types!` alone |
| `exclude_appsync_types = true` | Use `make_operation!` + `make_handlers!` |
| `batch = false` | `make_appsync!("schema.graphql", batch = false)` or `make_handlers!(batch = false)` |

### Cargo.toml changes

```diff
 [dependencies]
- lambda-appsync = { version = "0.9.0", features = ["tracing"] }
+ lambda-appsync = { version = "0.10.0", features = ["tracing"] }
+ tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
+ lambda_runtime = "1.0"
+ # Add AWS SDK crates you use directly:
+ aws-config = { version = "1.5", features = ["behavior-version-latest"] }
+ aws-sdk-dynamodb = "1.59"
```

> **Note:** `tokio` and `lambda_runtime` are re-exported by the crate (`lambda_appsync::tokio`, `lambda_appsync::lambda_runtime`), so adding them as direct dependencies is optional but recommended for clarity.

## FAQ

**Q: Why are the default features empty now?**

The crate works without any logging framework. This gives you full control — add `log`, `env_logger`, or `tracing` only when you need them. If you were relying on the old defaults, add `features = ["env_logger"]` to restore the previous behavior.

**Q: Do I need `tokio` and `lambda_runtime` as direct dependencies?**

The crate re-exports both `tokio` and `lambda_runtime`, so you can use `lambda_appsync::tokio` and `lambda_appsync::lambda_runtime` without adding them to your `Cargo.toml`. However, adding them directly is fine too.

**Q: Why do I have to write my own `main()` now?**

Generating `main()`, initializing loggers, and instantiating AWS SDK clients are application-level concerns — they have nothing to do with AppSync type generation or operation dispatch. The old `appsync_lambda_main!` bundled all of this together, which meant every new user preference (a different logging framework, a custom tracing setup, an SDK interceptor) required yet another macro option and feature flag. That approach doesn't scale.

The new design draws a clear boundary: `lambda-appsync` generates the types, the operation dispatch, and a handler trait. Everything else — runtime setup, logging, SDK clients, middleware — lives in your `main()` where it belongs.

A concrete example: the [`awssdk-instrumentation`](https://crates.io/crates/awssdk-instrumentation) crate provides out-of-the-box OpenTelemetry/X-Ray tracing for AWS SDK calls via a Tower layer wrapping the Lambda runtime. If `lambda-appsync` owns `main()`, there is no way to insert that layer. With the new design, composing the two is straightforward:

```rust
use lambda_appsync::{make_appsync, appsync_operation, AppsyncError};
use awssdk_instrumentation::interceptor::DefaultInterceptor;
use awssdk_instrumentation::lambda::layer::{DefaultTracingLayer, OTelFaasTrigger};

make_appsync!("schema.graphql");

#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // Initialize OpenTelemetry + X-Ray tracing
    let tracer_provider = awssdk_instrumentation::init::default_telemetry_init();

    // Build an instrumented DynamoDB client
    let sdk_config = aws_config::load_from_env().await;
    let _dynamo = aws_sdk_dynamodb::Client::from_conf(
        aws_sdk_dynamodb::config::Builder::from(&sdk_config)
            .interceptor(DefaultInterceptor::new())
            .build(),
    );

    // Wrap the Lambda runtime with the OTel Tower layer
    lambda_runtime::Runtime::new(lambda_runtime::service_fn(DefaultHandlers::service_fn))
        .layer(
            DefaultTracingLayer::new(move || {
                let _ = tracer_provider.force_flush();
            })
            .with_trigger(OTelFaasTrigger::Http),
        )
        .run()
        .await
}
```

**Q: How does batch processing work?**

By default (`batch = true`), the generated `service_fn` deserializes the Lambda payload as a `Vec<AppsyncEvent>` and processes events concurrently via `tokio::spawn`. Set `batch = false` if your AppSync API is not configured for batch invocations.

## Minimum Supported Rust Version (MSRV)

This crate requires Rust **1.82.0** or later.

## Contributing

We welcome contributions! Please read our [Contributing Guidelines](CONTRIBUTING.md) before submitting pull requests.

### Git Hooks

This project uses git hooks to ensure code quality. The hooks are automatically installed when you enter a development shell using `nix flakes` and `direnv`.

The following checks run before each commit:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Doc generation (`cargo doc`)
- Tests (`cargo test`)

To manually install the hooks:
```sh
./scripts/install-hooks.sh
```

## License

Distributed under the MIT License. See [`LICENSE`](LICENSE) for more information.

## Authors

- Jérémie RODON ([@JeremieRodon](https://github.com/JeremieRodon)) [![LinkedIn](https://img.shields.io/badge/linkedin-0077B5?style=for-the-badge&logo=linkedin&logoColor=white)](https://linkedin.com/in/JeremieRodon) — [RustyServerless](https://github.com/RustyServerless) [rustysl.com](https://rustysl.com/index.html?from=github-lambda-appsync)

If you find this crate useful, please star the repository and share your feedback!
