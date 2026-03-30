// Test generating only GraphQL types using make_types!
lambda_appsync::make_types!("../../../../schema.graphql");

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
