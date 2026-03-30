use lambda_appsync::serde_json::json;
lambda_appsync::make_appsync!(
    "schema.graphql",
    // NAME OVERRIDES
    // Override Player struct name
    name_override = Player: NewPlayer,
    // Override Player struct field name
    name_override = Player.name: email,
    // Override team `PYTHON` to be `Snake` instead of `Python`
    name_override = Team.PYTHON: Snake,
    name_override = WeirdFieldNames.await: no_await,
    name_override = WeirdFieldNames.crate: no_crate,
    name_override = WeirdFieldNames.u8: no_u8,

    // MUST also override ALL the operation return types when renaming a type
    type_override = Query.players: NewPlayer,
    type_override = Query.player: NewPlayer,
    type_override = Mutation.createPlayer: NewPlayer,
    type_override = Mutation.deletePlayer: NewPlayer,
);

#[test]
fn renamed_player_serialization() {
    let player_id = lambda_appsync::ID::new();
    // The struct is now called NewPlayer, and the `name` field is now `email`
    let player = NewPlayer {
        id: player_id,
        email: "Test Player".to_string(),
        team: Team::Snake,
    };

    let json = serde_json::to_value(&player).unwrap();
    // Serde still uses the original GraphQL field names
    assert_eq!(
        json,
        json!({
            "id": player_id,
            "name": "Test Player",
            "team": "PYTHON"
        })
    );
}

#[test]
fn renamed_player_deserialization() {
    let player_id = lambda_appsync::ID::new();
    let json = json!({
        "id": player_id,
        "name": "Test Player",
        "team": "RUST"
    });

    let player: NewPlayer = serde_json::from_value(json).unwrap();
    assert_eq!(player.id, player_id);
    assert_eq!(player.email, "Test Player");
    assert_eq!(player.team, Team::Rust);
}

#[test]
fn renamed_team_enum_round_trip() {
    // Team::Python is now Team::Snake
    let teams = vec![Team::Rust, Team::Snake, Team::Js, Team::MultiWordsTeam];

    // Assert the `all()` method returns the renamed variants
    assert_eq!(teams, Team::all());

    for team in teams {
        let json = serde_json::to_value(team).unwrap();
        let deserialized: Team = serde_json::from_value(json).unwrap();
        assert_eq!(team, deserialized);
    }
}

#[test]
fn renamed_team_serde_uses_original_graphql_name() {
    // Team::Snake serializes to "PYTHON" (original GraphQL name)
    let json = serde_json::to_value(Team::Snake).unwrap();
    assert_eq!(json, json!("PYTHON"));

    // Deserializing "PYTHON" gives Team::Snake
    let team: Team = serde_json::from_value(json!("PYTHON")).unwrap();
    assert_eq!(team, Team::Snake);

    // "SNAKE" is not a valid value (the GraphQL name is "PYTHON")
    let result: Result<Team, _> = serde_json::from_str("\"SNAKE\"");
    assert!(result.is_err());
    let result: Result<Team, _> = serde_json::from_str("\"Snake\"");
    assert!(result.is_err());
}

#[test]
fn renamed_weird_field_names_round_trip() {
    let mut json_map = serde_json::Map::new();

    // Rust keywords
    json_map.insert("as".to_string(), json!(true));
    json_map.insert("async".to_string(), json!(false));
    json_map.insert("await".to_string(), json!(true));
    json_map.insert("break".to_string(), json!(false));
    json_map.insert("const".to_string(), json!(true));
    json_map.insert("continue".to_string(), json!(false));
    json_map.insert("crate".to_string(), json!(true));
    json_map.insert("dyn".to_string(), json!(false));
    json_map.insert("else".to_string(), json!(true));
    json_map.insert("enum".to_string(), json!(false));
    json_map.insert("extern".to_string(), json!(true));
    json_map.insert("false".to_string(), json!(false));
    json_map.insert("fn".to_string(), json!(true));
    json_map.insert("for".to_string(), json!(false));
    json_map.insert("if".to_string(), json!(true));
    json_map.insert("impl".to_string(), json!(false));
    json_map.insert("in".to_string(), json!(true));
    json_map.insert("let".to_string(), json!(false));
    json_map.insert("loop".to_string(), json!(true));
    json_map.insert("match".to_string(), json!(false));
    json_map.insert("mod".to_string(), json!(true));
    json_map.insert("move".to_string(), json!(false));
    json_map.insert("mut".to_string(), json!(true));
    json_map.insert("pub".to_string(), json!(false));
    json_map.insert("ref".to_string(), json!(true));
    json_map.insert("return".to_string(), json!(false));
    json_map.insert("self".to_string(), json!(true));
    json_map.insert("static".to_string(), json!(false));
    json_map.insert("struct".to_string(), json!(true));
    json_map.insert("super".to_string(), json!(false));
    json_map.insert("trait".to_string(), json!(true));
    json_map.insert("true".to_string(), json!(false));
    json_map.insert("type".to_string(), json!(true));
    json_map.insert("unsafe".to_string(), json!(false));
    json_map.insert("use".to_string(), json!(true));
    json_map.insert("where".to_string(), json!(false));
    json_map.insert("while".to_string(), json!(true));

    // Reserved keywords
    json_map.insert("abstract".to_string(), json!(false));
    json_map.insert("become".to_string(), json!(true));
    json_map.insert("box".to_string(), json!(false));
    json_map.insert("do".to_string(), json!(true));
    json_map.insert("final".to_string(), json!(false));
    json_map.insert("macro".to_string(), json!(true));
    json_map.insert("override".to_string(), json!(false));
    json_map.insert("priv".to_string(), json!(true));
    json_map.insert("try".to_string(), json!(false));
    json_map.insert("typeof".to_string(), json!(true));
    json_map.insert("unsized".to_string(), json!(false));
    json_map.insert("virtual".to_string(), json!(true));
    json_map.insert("yield".to_string(), json!(false));

    // Primitive types
    json_map.insert("bool".to_string(), json!(true));
    json_map.insert("char".to_string(), json!("x"));
    json_map.insert("f32".to_string(), json!(1.0));
    json_map.insert("f64".to_string(), json!(2.0));
    json_map.insert("i8".to_string(), json!(3));
    json_map.insert("i16".to_string(), json!(4));
    json_map.insert("i32".to_string(), json!(5));
    json_map.insert("i64".to_string(), json!(6));
    json_map.insert("i128".to_string(), json!(7));
    json_map.insert("isize".to_string(), json!(8));
    json_map.insert("str".to_string(), json!("test"));
    json_map.insert("u8".to_string(), json!(9));
    json_map.insert("u16".to_string(), json!(10));
    json_map.insert("u32".to_string(), json!(11));
    json_map.insert("u64".to_string(), json!(12));
    json_map.insert("u128".to_string(), json!(13));
    json_map.insert("usize".to_string(), json!(14));

    let json = serde_json::Value::Object(json_map);

    let weird: WeirdFieldNames = serde_json::from_value(json.clone()).unwrap();
    // Verify the renamed fields have the correct values
    assert!(weird.no_await);
    assert!(weird.no_crate);
    assert_eq!(weird.no_u8, 9);

    let serialized = serde_json::to_value(&weird).unwrap();

    // Verify round-trip serialization preserves all original GraphQL field names
    assert_eq!(json, serialized);
}

#[test]
fn invalid_deserialization_with_renames() {
    // Test invalid team enum value — "SNAKE" is not valid (GraphQL name is "PYTHON")
    let result: Result<Team, _> = serde_json::from_str("\"SNAKE\"");
    assert!(result.is_err());
    let result: Result<Team, _> = serde_json::from_str("\"Snake\"");
    assert!(result.is_err());

    // Test missing required fields — "name" is the GraphQL field name (not "email")
    let result: Result<NewPlayer, _> = serde_json::from_value(json!({
        "id": "123",
        // Missing "name" field (the GraphQL field name, not the Rust field name "email")
        "email": "Test Player",
        "team": "RUST"
    }));
    assert!(result.is_err());
}

// Verify that the generated DefaultHandlers works with the renamed types.
// No operation handlers are registered, so all operations return "Unimplemented".
#[tokio::test]
async fn handlers_work_with_renamed_types() {
    let event = json!([{
        "info": {
            "fieldName": "players",
            "parentTypeName": "Query",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {},
        "identity": null,
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_appsync::lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = DefaultHandlers::service_fn(lambda_event).await.unwrap();

    // No handlers registered → Unimplemented
    let response_value = serde_json::to_value(response).unwrap();
    assert_eq!(response_value[0]["errorType"], "Unimplemented");
}
