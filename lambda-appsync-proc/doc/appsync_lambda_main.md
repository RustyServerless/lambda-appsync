Generates the code required to handle AWS AppSync Direct Lambda resolver events based on a GraphQL schema.

This macro takes a path to a GraphQL schema file and generates the complete foundation
for implementing an AWS AppSync Direct Lambda resolver:

- Rust types for all GraphQL types (enums, inputs, objects)
- Query/Mutation/Subscription operation enums
- AWS Lambda runtime setup with logging to handle the AWS AppSync event
- Optional AWS SDK client initialization

# Schema Path Argument

The first argument to this macro must be a string literal containing the path to your GraphQL schema file.
The schema path can be:

- An absolute filesystem path (e.g. "/home/user/project/schema.graphql")
- A relative path, that will be relative to your crate's root directory (e.g. "schema.graphql", "graphql/schema.gql")
- When in a workspace context, the relative path will be relative to the workspace root directory

# Options

- `batch = bool`: Enable/disable batch request handling (default: true)
- `hook = fn_name`: Add a custom hook function for request validation/auth
- `log_init = fn_name`: Use a custom log initialization function instead of the default one
- (feature: `log`) `event_logging = bool`: If true, the macro will generate code to dump the
  lambda payload JSON as well as parsed `AppsyncEvent<Operation>`s in the logs at debug level (default: `false`)
- `exclude_lambda_handler = bool`: Skip generation of Lambda handler code
- `only_lambda_handler = bool`: Only generate Lambda handler code
- `exclude_appsync_types = bool`: Skip generation of GraphQL type definitions
- `only_appsync_types = bool`: Only generate GraphQL type definitions
- `exclude_appsync_operations = bool`: Skip generation of operation enums
- `only_appsync_operations = bool`: Only generate operation enums
- `type_override` - see section below for details
- `name_override` - see section below for details
- `field_type_override` (Deprecated): Same as `type_override`

## Type Overrides

The `type_override` option allows overriding Rust types affected to various schema elements:

- GraphQL `type` and `input` Field types: `type_override = Type.field: CustomType`
- Operation return types (Query/Mutation): `type_override = OpType.operation: CustomType`
- Operation arguments (Query/Mutation/Subscription): `type_override = OpType.operation.arg: CustomType`

These overrides are only for the Rust code and must be compatible for serialization/deserialization purposes,
i.e. you can use `String` for a GraphQL `ID` but you cannot use a `u32` for a GraphQL `Float`.

## Name Overrides

The `name_override` option supports renaming various schema elements:

- Type/input/enum names: `name_override = TypeName: NewTypeName`
- Field names: `name_override = Type.field: new_field_name`
- Enum variants: `name_override = Enum.VARIANT: NewVariant`

These overrides are only for the Rust code and will not change serialization/deserialization,
i.e. `serde` will rename to the original GraphQL schema name.

# AWS SDK Clients

AWS SDK clients can be initialized by providing function definitions that return a cached SDK client type.
Each client is initialized only once and stored in a static [OnceLock](std::sync::OnceLock), making subsequent function calls
essentially free:

- Function name: Any valid Rust identifier that will be used to access the client
- Return type: Must be a valid AWS SDK client like `aws_sdk_dynamodb::Client`

```rust,no_run
# mod sub {
use lambda_appsync::appsync_lambda_main;

// Single client
appsync_lambda_main!(
    "schema.graphql",
    dynamodb() -> aws_sdk_dynamodb::Client,
);
# }
# fn main() {}
```
```rust,no_run
# mod sub {
# use lambda_appsync::appsync_lambda_main;
// Multiple clients
appsync_lambda_main!(
    "schema.graphql",
    dynamodb() -> aws_sdk_dynamodb::Client,
    s3() -> aws_sdk_s3::Client,
);
# }
# fn main() {}
```

These client functions can then be called from anywhere in the Lambda crate:
```rust,no_run
# fn dynamodb() -> aws_sdk_dynamodb::Client {
#   todo!()
# }
# fn s3() -> aws_sdk_s3::Client {
#   todo!()
# }
# mod sub {
use crate::{dynamodb, s3};
async fn do_something() {
    let dynamodb_client = dynamodb();
    let s3_client = s3();
    // Use clients...
}
# }
# fn main() {}
```

# Examples

## Basic usage with authentication hook:
```rust,no_run
# mod sub {
use lambda_appsync::{appsync_lambda_main, AppsyncEvent, AppsyncResponse, AppsyncIdentity};

fn is_authorized(identity: &AppsyncIdentity) -> bool {
    todo!()
}

// If the function returns Some(AppsyncResponse), the Lambda function will immediately return it.
// Otherwise, the normal flow of the AppSync operation processing will continue.
// This is primarily intended for advanced authentication checks that AppSync cannot perform, such as verifying that a user is requesting their own ID.
async fn auth_hook(
    event: &lambda_appsync::AppsyncEvent<Operation>
) -> Option<lambda_appsync::AppsyncResponse> {
    // Verify JWT token, check permissions etc
    if !is_authorized(&event.identity) {
        return Some(AppsyncResponse::unauthorized());
    }
    None
}

appsync_lambda_main!(
    "schema.graphql",
    hook = auth_hook,
    dynamodb() -> aws_sdk_dynamodb::Client
);
# }
# fn main() {}
```

## Generate only types for lib code generation:
```rust,no_run
# mod sub {
use lambda_appsync::appsync_lambda_main;
appsync_lambda_main!(
    "schema.graphql",
    only_appsync_types = true
);
# }
# fn main() {}
```

## Override field types, operation return type or argument types:
```rust,no_run
# mod sub {
use lambda_appsync::appsync_lambda_main;
appsync_lambda_main!(
    "schema.graphql",
    // Use String instead of the default lambda_appsync::ID
    // Override Player.id to use String instead of ID
    type_override = Player.id: String,
    // Multiple overrides, here changing another `Player` field type
    type_override = Player.team: String,
    // Return value override
    type_override = Query.gameStatus: String,
    type_override = Mutation.setGameStatus: String,
    // Argument override
    type_override = Query.player.id: String,
    type_override = Mutation.deletePlayer.id: String,
    type_override = Subscription.onDeletePlayer.id: String,
);
# }
# fn main() {}
```

## Override type, input, enum, fields or variants names:
```rust,no_run
# mod sub {
use lambda_appsync::appsync_lambda_main;
appsync_lambda_main!(
    "schema.graphql",
    // Override Player struct name
    name_override = Player: NewPlayer,
    // Override Player struct field name
    name_override = Player.name: email,
    // Override team `PYTHON` to be `Snake` (instead of `Python`)
    name_override = Team.PYTHON: Snake,
    // MUST also override ALL the operations return type !!!
    type_override = Query.players: NewPlayer,
    type_override = Query.player: NewPlayer,
    type_override = Mutation.createPlayer: NewPlayer,
    type_override = Mutation.deletePlayer: NewPlayer,
);
# }
# fn main() {}
```
Note that when using `name_override`, the macro does not automatically change the case:
you are responsible to provide the appropriate casing or Clippy will complain.

## Use a custom log initialization function:

### Feature `env_logger` (default)

By default, `lambda_appsync` exposes and uses `log` and `env_logger`. You can override the
initialization code if you wish:

```rust,no_run
# mod sub {
// This is in fact equivalent to the default initialization code
fn log_init_fct() {
    use lambda_appsync::env_logger;
    env_logger::Builder::from_env(
        env_logger::Env::default()
        // Default log level is info, expect tracing::span is warn
        .default_filter_or("info,tracing::span=warn")
        .default_write_style_or("never"),
    )
    // Format timestamps with microseconds
    .format_timestamp_micros()
    .init();
}
lambda_appsync::appsync_lambda_main!(
    "schema.graphql",
    log_init = log_init_fct
);
# }
# fn main() {}
```

### Feature `tracing`

Alternatively, you can use the `tracing` feature so `lambda_appsync` exposes and uses `log`, `tracing` and `tracing-subscriber`

```rust,no_run
# mod sub {
// This is in fact equivalent to the default initialization code
fn tracing_init_fct() {
    use lambda_appsync::{tracing, tracing_subscriber};
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
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
lambda_appsync::appsync_lambda_main!(
    "schema.graphql",
    log_init = tracing_init_fct
);
# }
# fn main() {}
```

## Disable batch processing:
```rust,no_run
# mod sub {
lambda_appsync::appsync_lambda_main!(
    "schema.graphql",
    batch = false
);
# }
# fn main() {}
```
