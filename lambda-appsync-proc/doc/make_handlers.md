Generates a `Handlers` trait and a `DefaultHandlers` struct for handling AWS AppSync Lambda events.

# Usage

```text
make_handlers!();

// or with options:
make_handlers!(
    batch = true,                                    // enable/disable batch handling (default: true)
    operation_type = my_crate::Operation,             // custom path to the Operation type
);
```

This macro is one of three composable macros for building an AppSync Direct Lambda resolver:

- [make_types!] — generates Rust types (structs and enums) from a GraphQL schema
- [make_operation!] — generates the `Operation` enum and dispatch logic
- `make_handlers!` — generates the Lambda handler trait and default implementation

[make_appsync!] is a convenience macro that combines all three.

Unlike [make_types!] and [make_operation!], this macro does **not** take a schema path as its
first argument. It only generates the handler trait and struct.

# Prerequisites

The `Operation` type must be in scope when `make_handlers!` is invoked. This is typically
achieved by calling [make_operation!] (or [make_appsync!]) in the same scope beforehand.
Alternatively, use the `operation_type` parameter to specify a custom path.

# Parameters

## batch

- `batch = true` (default): Generates both `appsync_handler` (single event) and
  `appsync_batch_handler` (batch of events processed concurrently via `tokio::spawn`).
  The `service_fn` method deserializes the payload as a `Vec` and calls `appsync_batch_handler`.
- `batch = false`: Only generates `appsync_handler`. The `service_fn` method deserializes
  the payload as a single event and calls `appsync_handler`.

## operation_type

Specifies the type to use for the `Operation` enum. Defaults to `Operation` (expected to be
in scope).

Use this when the `Operation` type is defined in a different module or crate:
- `operation_type = crate::operations::Operation`
- `operation_type = my_operations::Operation`

# What Gets Generated

## The `Handlers` trait

```rust
# use core::future::Future;
# use lambda_appsync::{lambda_runtime, serde_json, AppsyncEvent, AppsyncResponse};
# use lambda_runtime::{LambdaEvent, Error};
# use serde_json::Value;
# struct Operation;
trait Handlers {
    // Handles a single deserialized AppSync event.
    // Default: delegates to Operation::execute()
    fn appsync_handler(
        event: AppsyncEvent<Operation>
    ) -> impl Future<Output = AppsyncResponse> + Send + 'static;

    // (Only when batch = true)
    // Handles a batch of events concurrently using tokio::spawn.
    // Default: spawns appsync_handler for each event
    async fn appsync_batch_handler(
        events: Vec<AppsyncEvent<Operation>>
    ) -> Vec<AppsyncResponse>;

    // Top-level Lambda handler.
    // Default: deserializes the raw JSON payload and calls the appropriate handler
    async fn service_fn(
        event: LambdaEvent<Value>
    ) -> Result<Vec<AppsyncResponse>, Error>;
    // (returns Result<AppsyncResponse, Error> when batch = false)
}
```

When the `tracing` feature is enabled, the default Handlers trait methods are instrumented for observability.

## The `DefaultHandlers` struct

```rust
# lambda_appsync::make_appsync!("schema.graphql");
# mod sub {
# use super::Handlers;
struct DefaultHandlers;
impl Handlers for DefaultHandlers {}
# }
# fn main() {}
```

A zero-sized struct that uses all the default trait implementations. Use it directly if you
don't need to customize handler behavior.

# Customizing Handler Behavior

The key design principle: override specific trait methods while keeping defaults for the rest.
The default implementations serve as a starting point — copy the default body and modify it.

This replaces the old `hook`, `log_init`, and `event_logging` options from [appsync_lambda_main!]:

| Old option | New approach |
|-----------|-------------|
| `hook = fn_name` | Override `appsync_handler` to add pre/post processing |
| `event_logging = true` | Add logging in your `appsync_handler` or `service_fn` override |
| `log_init = fn_name` | Call your init function directly in `main()` |
| AWS SDK clients | Initialize clients directly in `main()` |

# Examples

## Basic usage with default handlers:
```rust,no_run 
# use lambda_appsync::{tokio, lambda_runtime};
use lambda_appsync::{make_types, make_operation, make_handlers};

make_types!("schema.graphql");
make_operation!("schema.graphql");
make_handlers!();

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(
        lambda_runtime::service_fn(DefaultHandlers::service_fn)
    ).await
}
```

## Disable batch processing:
```rust,no_run
# mod sub {
# lambda_appsync::make_types!("schema.graphql");
# lambda_appsync::make_operation!("schema.graphql");
lambda_appsync::make_handlers!(batch = false);
# }
# fn main() {}
```

## Custom handler with authentication hook:
```rust,no_run
# use lambda_appsync::{tokio, lambda_runtime};
use lambda_appsync::{make_types, make_operation, make_handlers};
use lambda_appsync::{AppsyncEvent, AppsyncResponse, AppsyncIdentity};

make_types!("schema.graphql");
make_operation!("schema.graphql");
make_handlers!();

struct MyHandlers;
impl Handlers for MyHandlers {
    fn appsync_handler(
        event: AppsyncEvent<Operation>
    ) -> impl std::future::Future<Output = AppsyncResponse> + Send + 'static {
        async move {
            // Custom authentication check
            if let AppsyncIdentity::ApiKey = &event.identity {
                return AppsyncResponse::unauthorized();
            }
            // Delegate to the default operation dispatch
            event.info.operation.execute(event).await
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(
        lambda_runtime::service_fn(MyHandlers::service_fn)
    ).await
}
```

## Using `operation_type` for cross-module Operation:
```rust,no_run
# mod operations {
#     lambda_appsync::make_types!("schema.graphql");
#     lambda_appsync::make_operation!("schema.graphql");
# }
# mod sub {
use lambda_appsync::make_handlers;

make_handlers!(
    operation_type = crate::operations::Operation,
);
# }
# fn main() {}
```

# Important Notes

- This macro does **not** take a schema path. It only generates the handler trait and struct.
- The `Operation` type must be in scope (or specified via `operation_type`). It must have an
  `execute` method, which is generated by [make_operation!].
- The generated `Handlers` trait is marked `#[deny(dead_code)]` — all methods must be used
  or you'll get a compilation warning.
- When `batch = true`, the batch handler uses `tokio::spawn` for concurrent processing, which
  requires the handler future to be `Send + 'static`. This is why `appsync_handler` returns
  `impl Future<Output = AppsyncResponse> + Send + 'static` instead of being a plain `async fn`
  in batch mode.
