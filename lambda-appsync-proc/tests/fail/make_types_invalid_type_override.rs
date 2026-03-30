lambda_appsync::make_types!(
    "../../../../schema.graphql",
    type_override = InvalidType.field: String,
);
fn main() {}
