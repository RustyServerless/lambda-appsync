Generates Rust types (structs and enums) from a GraphQL schema file.

# Usage

```text
make_types!("path/to/schema.graphql");

// or with options:
make_types!(
    "path/to/schema.graphql",
    type_override = Type.field: CustomType,               // override a field type
    name_override = TypeName: NewTypeName,                // rename a type
    name_override = Type.field: new_field_name,           // rename a field
    name_override = Enum.VARIANT: NewVariant,             // rename an enum variant
    default_traits = MyType: false,                       // disable default traits for a type
    derive = MyType: Default,                             // add an extra derive macro to a type
    derive = MyType: PartialEq,                           // (specify multiple times for multiple derives)
);
```

This is one of three composable macros that together replace the monolithic [appsync_lambda_main!]:

- `make_types!` — generates types (structs for GraphQL `type` and `input`, enums for GraphQL `enum`)
- [make_operation!] — generates the `Operation` enum and dispatch logic
- [make_handlers!] — generates the Lambda handler trait

[make_appsync!] is a convenience macro that combines all three.

# Schema Path Argument

The first argument to this macro must be a string literal containing the path to your GraphQL schema file.
The schema path can be:

- An absolute filesystem path (e.g. "/home/user/project/schema.graphql")
- A relative path, that will be relative to your crate's root directory (e.g. "schema.graphql", "graphql/schema.gql")
- When in a workspace context, the relative path will be relative to the workspace root directory

# Options

- `type_override` — see section below for details
- `name_override` — see section below for details
- `default_traits` — see section below for details
- `derive` — see section below for details

## Type Overrides

The `type_override` option allows overriding the Rust type generated for a struct field:

- `type_override = Type.field: CustomType`

These overrides are only for the Rust code and must be compatible for serialization/deserialization purposes,
i.e. you can use `String` for a GraphQL `ID` but you cannot use a `u32` for a GraphQL `Float`.

Note: operation-level overrides (`Query.op: Type` or `Query.op.arg: Type`) are not relevant here since
`make_types!` does not generate operation types. Use [make_operation!] or [make_appsync!] for those.

## Name Overrides

The `name_override` option supports renaming various schema elements:

- Type/input/enum names: `name_override = TypeName: NewTypeName`
- Field names: `name_override = Type.field: new_field_name`
- Enum variants: `name_override = Enum.VARIANT: NewVariant`

These overrides are only for the Rust code and will not change serialization/deserialization,
i.e. `serde` will rename to the original GraphQL schema name.

Note that when using `name_override`, the macro does not automatically change the case:
you are responsible to provide the appropriate casing or Clippy will complain.

## Default Traits

The `default_traits` option controls whether the macro applies its default set of traits
on a given type:

- `default_traits = TypeName: bool`

The default value is `true`. When set to `false` for a type, none of the default traits are
implemented for that type, giving you the opportunity to implement them yourself.

**Important:** certain traits are required by the generated operation dispatch code regardless
of this setting:

- `serde::Serialize` **must** be implemented on any type used as a return type of a Query or
  Mutation operation.
- `serde::Deserialize` **must** be implemented on any `input` type used as an operation argument.

You are free to disable the default implementations, but you must provide alternative ones —
whether via `derive` entries, manual `impl` blocks, or any other means.

The default traits are:

- **Structs** (GraphQL `type` and `input`): `Debug`, `Clone`, `Serialize`, `Deserialize`
- **Enums** (GraphQL `enum`): `Debug`, `Clone`, `Copy`, `Serialize`, `Deserialize`, `PartialEq`,
  `Eq`, `PartialOrd`, `Ord`, `Hash`, `Display` and `FromStr`

## Additional Derive Macros

The `derive` option allows adding an extra derive macro on top of the default ones for a given
type:

- `derive = TypeName: Trait`

Each `derive` entry adds a single trait. Specify the option multiple times to add several traits:

- `derive = Player: Default`
- `derive = Player: PartialEq`

This is additive: it does not replace the default traits (unless `default_traits` is
set to `false` for the same type, in which case only the explicitly listed traits are derived).

This is useful for deriving traits that the macro does not provide, such as `Default`
on a struct or `PartialEq`.

For enums, `derive = MyEnum: Default` is supported: the first variant declared in the GraphQL
schema becomes the default value.

# What Gets Generated

For each GraphQL `type` (excluding Query, Mutation, and Subscription), with default traits:
```rust,no_run
# use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
# struct FieldType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeName {
    pub field1: FieldType,
    pub field2: Option<FieldType>,
    // ...
}
```

For each GraphQL `input`, with default traits:
```rust,no_run
# use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
# struct FieldType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputName {
    pub field1: FieldType,
    // ...
}
```

For each GraphQL `enum`, with default traits:
```rust,no_run
# use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnumName {
    Variant1,
    Variant2,
    // ...
}
```

Each generated enum also gets:
- `const COUNT: usize` — number of variants
- `const fn all() -> [Self; Self::COUNT]` — array of all variants
- `const fn index(self) -> usize` — index of the variant (useful for array indexing)
- `Display` impl (when `default_traits` is `true`, displays the original GraphQL name)
- `FromStr` impl (when `default_traits` is `true`, parses from the original GraphQL name)

# GraphQL to Rust Type Mapping

| GraphQL | Rust |
|---------|------|
| `String` | `String` |
| `ID` | `lambda_appsync::ID` |
| `Int` | `i32` |
| `Float` | `f64` |
| `Boolean` | `bool` |
| `AWSEmail` | `lambda_appsync::AWSEmail` |
| `AWSPhone` | `lambda_appsync::AWSPhone` |
| `AWSTimestamp` | `lambda_appsync::AWSTimestamp` |
| `AWSDate` | `lambda_appsync::AWSDate` |
| `AWSTime` | `lambda_appsync::AWSTime` |
| `AWSDateTime` | `lambda_appsync::AWSDateTime` |
| `AWSJSON` | `serde_json::Value` |
| `AWSURL` | `lambda_appsync::AWSUrl` |
| `AWSIPAddress` | `core::net::IpAddr` |
| `[T]` (list) | `Vec<T>` |
| nullable | `Option<T>` |
| Custom type | The generated Rust struct/enum |

# Use Case: Shared Types Crate

The primary use case for `make_types!` alone (vs [make_appsync!]) is when you want to share
GraphQL types across multiple crates — e.g., a shared library crate for types and separate
binary crates for different Lambda functions:

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

When using `name_override` with `make_types!` in a multi-crate setup, you must use the same
overrides in [make_operation!] so that operation signatures reference the correct renamed types.

# Examples

## Basic usage:
```rust,no_run
# mod sub {
lambda_appsync::make_types!("schema.graphql");
# }
# fn main() {}
```

## With type overrides:
```rust,no_run
# mod sub {
lambda_appsync::make_types!(
    "schema.graphql",
    // Use String instead of the default lambda_appsync::ID
    type_override = Player.id: String,
);
# }
# fn main() {}
```

## With name overrides:
```rust,no_run
# mod sub {
lambda_appsync::make_types!(
    "schema.graphql",
    // Rename the Player struct
    name_override = Player: GqlPlayer,
    // Rename a field
    name_override = Player.name: email,
    // Rename an enum variant
    name_override = Team.PYTHON: Snake,
);
# }
# fn main() {}
```

## Combined overrides:
```rust,no_run
# mod sub {
lambda_appsync::make_types!(
    "schema.graphql",
    type_override = Player.id: String,
    name_override = Player: GqlPlayer,
);
# }
# fn main() {}
```

## Disable default derivations for a type:
```rust,no_run
# mod sub {
lambda_appsync::make_types!(
    "schema.graphql",
    // Player will have no derive macros — implement them yourself
    default_traits = Player: false,
);
# }
# fn main() {}
```

## Add extra derive macros to a type:
```rust,no_run
# mod sub {
lambda_appsync::make_types!(
    "schema.graphql",
    // Add Default and PartialEq on top of the default derives
    derive = Team: Default,
    derive = Player: Default,
    derive = Player: PartialEq,
);
# }
# fn main() {}
```

## Combine default_traits and derive:
```rust,no_run
# mod sub {
lambda_appsync::make_types!(
    "schema.graphql",
    derive = Team: Default,
    // Disable defaults and provide only the derives you need
    default_traits = Player: false,
    derive = Player: Debug,
    derive = Player: Default,
);
# }
# fn main() {}
```

This macro generates **only** types (structs and enums). It does not generate the `Operation` enum
or Lambda handler code. Use [make_operation!] for operations and [make_handlers!] for the handler,
or [make_appsync!] for all three at once.
