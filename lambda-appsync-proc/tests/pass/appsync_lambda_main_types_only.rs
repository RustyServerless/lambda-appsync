// Test generating only GraphQL types
lambda_appsync::appsync_lambda_main!("../../../../schema.graphql", only_appsync_types = true);

fn main() {
    // Verify we can construct the generated types
    let player = Player {
        id: lambda_appsync::ID::new(),
        name: "Test Player".into(),
        team: Team::Rust,
    };

    assert_eq!(player.name, "Test Player");
    assert_eq!(player.team, Team::Rust);
}
