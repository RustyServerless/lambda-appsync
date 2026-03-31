lambda_appsync::make_appsync!(
    "../../../../schema.graphql",
    type_override = Query.player.invalidArg: String,
);
fn main() {}
