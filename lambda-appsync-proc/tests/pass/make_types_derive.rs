lambda_appsync::make_types!(
    "../../../../schema.graphql",
    derive = Team: Default,
    derive = Player: Default,
    derive = Player: PartialEq,
);

fn main() {
    // Verify Default works for enum — first variant (RUST) becomes default
    let team = Team::default();
    assert_eq!(team, Team::Rust);

    // Verify Default works for struct
    let player = Player::default();
    // Verify PartialEq works
    let mut player2 = Player::default();
    // They are different because of the random UUID
    assert_ne!(player, player2);
    player2.id = player.id;
    // Now they are equals because both Default
    assert_eq!(player, player2);
}
