Convenience macro that combines [make_types!], [make_operation!], and [make_handlers!] into a single invocation.

# Usage

```text
make_appsync!("path/to/schema.graphql");

// or with options:
make_appsync!(
    "path/to/schema.graphql",
    batch = true,                                        // enable/disable batch handling
    type_override = Type.field: CustomType,               // override a field type
    type_override = Query.operation: CustomType,          // override an operation return type
    type_override = Query.operation.arg: CustomType,      // override an operation argument type
    name_override = TypeName: NewTypeName,                // rename a type
    name_override = Type.field: new_field_name,           // rename a field
    default_traits = MyType: false,                       // disable default traits for a type
    derive = MyType: Default,                             // add an extra derive macro to a type
    derive = MyType: PartialEq,                           // (specify multiple times for multiple derives)
    error_logging = false,                               // disable error logging (feature: log)
);
```

This generates everything needed to implement an AWS AppSync Direct Lambda resolver from a
GraphQL schema:

- Rust types for all GraphQL types (enums, inputs, objects) — same as [make_types!]
- The `Operation` enum with `QueryField`/`MutationField`/`SubscriptionField` sub-enums and
  dispatch logic — same as [make_operation!]
- The `Handlers` trait and `DefaultHandlers` struct — same as [make_handlers!]

This is the recommended macro for the common single-Lambda-function use case. For multi-crate
setups or when you need finer control over what gets generated where, use the individual macros
instead.

# Schema Path Argument

The first argument to this macro must be a string literal containing the path to your GraphQL schema file.
The schema path can be:

- An absolute filesystem path (e.g. "/home/user/project/schema.graphql")
- A relative path, that will be relative to your crate's root directory (e.g. "schema.graphql", "graphql/schema.gql")
- When in a workspace context, the relative path will be relative to the workspace root directory

# Options

`make_appsync!` accepts the union of parameters from [make_types!], [make_operation!], and
[make_handlers!], except for parameters that only make sense in a multi-crate setup.

## From make_types! / make_operation!

- `type_override` — Override Rust types for fields, operation return types, or operation arguments.
  See [make_types!] and [make_operation!] for full syntax.
- `name_override` — Rename types, fields, or enum variants. See [make_types!] for full syntax.
- `default_traits` — Control whether default traits are implemented for a type.
  See [make_types!] for full syntax and details.
- `derive` — Add extra derive macros on top of the defaults for a type.
  See [make_types!] for full syntax and details.

## From make_operation! (feature: `log`)

- `error_logging = bool` (default: `true`): When enabled, the generated `execute` method logs
  errors via `log::error!` before converting them to an `AppsyncResponse`. Only available with the
  `log` feature.

## From make_handlers!

- `batch = bool` (default: `true`): Enable/disable batch request handling. See [make_handlers!]
  for details.

## Parameters NOT available

- `type_module` (from [make_operation!]) — Not needed since types are generated in the same scope
- `operation_type` (from [make_handlers!]) — Not needed since `Operation` is generated in the same scope

## Type Overrides

The `type_override` option allows overriding Rust types affected to various schema elements:

- GraphQL `type` and `input` field types: `type_override = Type.field: CustomType`
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

Note that when using `name_override`, the macro does not automatically change the case:
you are responsible to provide the appropriate casing or Clippy will complain.

# Examples

## Basic usage:
```rust,no_run
# use lambda_appsync::{tokio, lambda_runtime};
use lambda_appsync::make_appsync;

make_appsync!("schema.graphql");

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(
        lambda_runtime::service_fn(DefaultHandlers::service_fn)
    ).await
}
```

## With type and name overrides:
```rust,no_run
# mod sub {
lambda_appsync::make_appsync!(
    "schema.graphql",
    type_override = Player.id: String,
    type_override = Query.player.id: String,
    type_override = Mutation.deletePlayer.id: String,
    type_override = Subscription.onDeletePlayer.id: String,
    name_override = Player: GqlPlayer,
    // *MUST* also override ALL the operations return type when changing the name of a type!!!
    type_override = Query.players: GqlPlayer,
    type_override = Query.player: GqlPlayer,
    type_override = Mutation.createPlayer: GqlPlayer,
    type_override = Mutation.deletePlayer: GqlPlayer,
);
# }
# fn main() {}
```

## Disable batch processing:
```rust,no_run
# mod sub {
lambda_appsync::make_appsync!(
    "schema.graphql",
    batch = false,
);
# }
# fn main() {}
```

## With a custom handler:
```rust,no_run
# use lambda_appsync::{tokio, lambda_runtime};
use lambda_appsync::{make_appsync, appsync_operation, AppsyncError, AppsyncResponse, AppsyncEvent, AppsyncIdentity};

make_appsync!("schema.graphql");

struct MyHandlers;
impl Handlers for MyHandlers {
    async fn appsync_handler(event: AppsyncEvent<Operation>) -> AppsyncResponse {
        if let AppsyncIdentity::ApiKey = &event.identity {
            return AppsyncResponse::unauthorized();
        }
        event.info.operation.execute(event).await
    }
}

#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    Ok(vec![])
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    lambda_runtime::run(
        lambda_runtime::service_fn(MyHandlers::service_fn)
    ).await
}
```

# Equivalence

The following `make_appsync!` invocation:
```rust,no_run
# use lambda_appsync::serde;
# mod sub {
lambda_appsync::make_appsync!(
    "schema.graphql",
    batch = true,
    type_override = Player.id: String,
    default_traits = Player: false,
    derive = Player: Debug,
    derive = Player: serde::Serialize,
);
# }
# fn main() {}
```

is equivalent to:
```rust,no_run
# use lambda_appsync::serde;
# mod sub {
lambda_appsync::make_types!(
    "schema.graphql",
    type_override = Player.id: String,
    default_traits = Player: false,
    derive = Player: Debug,
    derive = Player: serde::Serialize,
);
lambda_appsync::make_operation!(
    "schema.graphql",
    type_override = Player.id: String,
);
lambda_appsync::make_handlers!(
    batch = true,
);
# }
# fn main() {}
```

# When to Use make_appsync! vs Individual Macros

| Scenario | Recommendation |
|----------|---------------|
| Single Lambda function, all code in one crate | Use `make_appsync!` |
| Shared types library + multiple Lambda binaries | Use [make_types!] in the lib, [make_operation!] + [make_handlers!] in each binary |
| Custom handler logic only | Use `make_appsync!` + override `Handlers` trait methods |
| Need different operations per Lambda | Use [make_types!] shared, separate [make_operation!] per Lambda |
