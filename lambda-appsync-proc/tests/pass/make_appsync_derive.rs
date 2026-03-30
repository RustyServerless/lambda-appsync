lambda_appsync::make_appsync!(
    "../../../../schema.graphql",
    derive = Team: Default,
    derive = Player: Default,
    derive = Player: PartialEq,
);

fn main() {
    // Verify Default works for enum — first variant (RUST) becomes default
    let team = Team::default();
    assert_eq!(team, Team::Rust);
}
