lambda_appsync::make_appsync!(
    "../../../../schema.graphql",
    type_override = Player.inexistant: String,
);
fn main() {}
