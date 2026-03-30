lambda_appsync::make_appsync!(
    "../../../../schema.graphql",
    type_override = Mutation.invalidMutation: String,
);
fn main() {}
