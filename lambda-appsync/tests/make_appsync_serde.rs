use lambda_appsync::serde_json::json;

// Generate only the GraphQL types from the schema using the new make_types! macro
lambda_appsync::make_types!("schema.graphql");

#[test]
fn player_serialization() {
    let player_id = lambda_appsync::ID::new();
    let player = Player {
        id: player_id,
        name: "Test Player".to_string(),
        team: Team::Rust,
    };

    let json = serde_json::to_value(&player).unwrap();
    assert_eq!(
        json,
        json!({
            "id": player_id,
            "name": "Test Player",
            "team": "RUST"
        })
    );
}

#[test]
fn player_deserialization() {
    let player_id = lambda_appsync::ID::new();
    let json = json!({
        "id": player_id,
        "name": "Test Player",
        "team": "RUST"
    });

    let player: Player = serde_json::from_value(json).unwrap();
    assert_eq!(player.id, player_id);
    assert_eq!(player.name, "Test Player");
    assert_eq!(player.team, Team::Rust);
}

#[test]
fn team_enum_round_trip() {
    // Test all variants
    let teams = vec![Team::Rust, Team::Python, Team::Js, Team::MultiWordsTeam];

    // Assert the `all()` method
    assert_eq!(teams, Team::all());

    for team in teams {
        let json = serde_json::to_value(team).unwrap();
        let deserialized: Team = serde_json::from_value(json).unwrap();
        assert_eq!(team, deserialized);
    }
}

#[test]
fn game_status_enum_round_trip() {
    let statuses = vec![GameStatus::Started, GameStatus::Stopped];

    for status in statuses {
        let json = serde_json::to_value(status).unwrap();
        let deserialized: GameStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status, deserialized);
    }
}

#[test]
fn weird_field_names_round_trip() {
    // Test that Rust keywords are properly escaped in field names
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
    let serialized = serde_json::to_value(&weird).unwrap();

    // Verify round-trip serialization preserves all fields
    assert_eq!(json, serialized);
}

#[test]
fn invalid_deserialization() {
    // Test invalid team enum value
    let result: Result<Team, _> = serde_json::from_str("\"INVALID\"");
    assert!(result.is_err());

    // Test invalid game status
    let result: Result<GameStatus, _> = serde_json::from_str("\"PAUSED\"");
    assert!(result.is_err());

    // Test missing required fields
    let result: Result<Player, _> = serde_json::from_value(json!({
        "id": "123",
        // Missing "name" field
        "team": "RUST"
    }));
    assert!(result.is_err());
}

#[test]
fn null_handling_optional_team() {
    // Test that optional team accepts null
    let json = json!({
        "team": null
    });

    let optional_team: OptionalTeam = serde_json::from_value(json.clone()).unwrap();
    assert!(optional_team.team.is_none());

    // Test that optional team accepts a valid team value
    let json = json!({
        "team": "RUST"
    });

    let optional_team: OptionalTeam = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(optional_team.team, Some(Team::Rust));

    // Test that null team is not serialized
    let optional_team = OptionalTeam { team: None };
    assert_eq!(serde_json::to_value(optional_team).unwrap(), json!({}));
}
