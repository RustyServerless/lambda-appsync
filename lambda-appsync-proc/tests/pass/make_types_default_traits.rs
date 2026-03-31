lambda_appsync::make_types!(
    "../../../../schema.graphql",
    default_traits = Player: false,
    derive = Player: Debug,
    derive = Player: lambda_appsync::serde::Serialize,
    derive = Player: lambda_appsync::serde::Deserialize,
);

fn main() {
    // Player should have Debug but NOT Clone (default_traits disabled)
    // It should still have Serialize/Deserialize from explicit derive
    let player = Player {
        id: lambda_appsync::ID::new(),
        name: "test".into(),
        team: Team::Rust,
    };
    // Debug works
    let _ = format!("{:?}", player);
    // Serialize works
    let _ = lambda_appsync::serde_json::to_value(&player).unwrap();
}
