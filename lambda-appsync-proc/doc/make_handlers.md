Generates the `Handlers` trait with default methods for handling AWS AppSync Lambda events.

# Usage

```text
make_handlers!();

// or with options:
make_handlers!(
    batch = true,                         // use batch handling (default: true)
    operation_type = my_crate::Operation, // custom path to the Operation type
);
```

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

Use this when the `Operation` type is defined in a different crate:

- `operation_type = other_crate::Operation`

# What Gets Generated

## The `Handlers` trait

```rust
# use core::future::Future;
# use lambda_appsync::{lambda_runtime, serde_json, AppsyncEvent, AppsyncResponse};
# use lambda_runtime::{LambdaEvent, Error};
# use serde_json::Value;
# struct Operation;
trait Handlers {
    // Called synchronously before every dispatch.
    // Returning Some(response) short-circuits the dispatch; None proceeds normally.
    // Default: returns None
    fn event_hook(_event: &AppsyncEvent<Operation>) -> Option<AppsyncResponse>;

    // Handles a single deserialized AppSync event.
    // Default: if event_hook returns None, delegates to Operation::execute()
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

# Customizing Handler Behavior

The key design principle: override specific trait methods while keeping defaults for the rest.
The default implementations serve as a starting point — copy the default body and modify it.

This replaces the old `hook`, `log_init`, and `event_logging` options from [appsync_lambda_main!]:

| Old option             | New approach                                                   |
| ---------------------- | -------------------------------------------------------------- |
| `hook = fn_name`       | Override `event_hook` to add pre processing                    |
| `event_logging = true` | Add logging in your `appsync_handler` or `service_fn` override |
| `log_init = fn_name`   | Call your init function directly in `main()`                   |
| AWS SDK clients        | Initialize clients directly in `main()`                        |

# Examples

## Basic usage with default handlers:

```rust,no_run
# use lambda_appsync::{tokio, lambda_runtime};
use lambda_appsync::{default_service_fn, make_types, make_operation, make_handlers};

make_types!("schema.graphql");
make_operation!("schema.graphql");
make_handlers!();

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(
        lambda_runtime::service_fn(default_service_fn!())
    ).await
}
```

## Disable batch processing:

```rust,no_run
# use lambda_appsync::{tokio, lambda_runtime};
use lambda_appsync::{default_service_fn, make_types, make_operation, make_handlers};

make_types!("schema.graphql");
make_operation!("schema.graphql");
make_handlers!(batch = false);

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(
        lambda_runtime::service_fn(default_service_fn!())
    ).await
}
```

## Custom handlers with authentication hook:

```rust,no_run
# use lambda_appsync::{tokio, lambda_runtime};
use lambda_appsync::make_appsync;
use lambda_appsync::{AppsyncEvent, AppsyncResponse, AppsyncIdentity};

make_appsync!("schema.graphql");

struct MyHandlers;
impl Handlers for MyHandlers {
    fn event_hook(event: &AppsyncEvent<Operation>) -> Option<AppsyncResponse> {
        // Custom authentication check
        if let AppsyncIdentity::ApiKey = &event.identity {
          return Some(AppsyncResponse::unauthorized())
        }
        None
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
