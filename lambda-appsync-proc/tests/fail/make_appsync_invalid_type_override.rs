lambda_appsync::make_appsync!(
    "../../../../schema.graphql",
    type_override = InvalidType.field: String,
);
fn main() {}
