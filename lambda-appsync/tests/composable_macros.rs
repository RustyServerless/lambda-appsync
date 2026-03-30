#![allow(unused)]
// Tests for the composable macros: make_types!, make_operation!, make_handlers!
// with their new options that don't exist in the legacy appsync_lambda_main! macro.
//
// NOTE: #[appsync_operation] generates `impl crate::Operation { ... }`, so it can only
// be used at the crate root level. The root of this integration test file IS the crate
// root, so we place the primary make_operation! + make_handlers! invocations here.

use lambda_appsync::{appsync_operation, AppsyncError};

// Root-level composable macro invocations (used by appsync_operation handlers below)
lambda_appsync::make_types!("schema.graphql");
lambda_appsync::make_operation!("schema.graphql");
lambda_appsync::make_handlers!();

// Root-level operation handlers (required because appsync_operation generates impl crate::Operation)
#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    Ok(vec![])
}

#[appsync_operation(query(player))]
async fn get_player(id: lambda_appsync::ID) -> Result<Option<Player>, AppsyncError> {
    let _ = id;
    Ok(None)
}

// ── Test 1: make_types! with extra `derive` options ──────────────────────────
//
// Verify that `derive = Team: Default` and `derive = Player: PartialEq` add the
// expected trait implementations on top of the default ones.
mod extra_derives {
    lambda_appsync::make_types!(
        "schema.graphql",
        derive = Team: Default,
        derive = Player: PartialEq,
    );

    #[test]
    fn team_has_default_derive() {
        // Default for an enum with `derive = Default` gives the first variant
        let team: Team = Default::default();
        assert_eq!(team, Team::Rust);
    }

    #[test]
    fn player_has_partial_eq_derive() {
        let id = lambda_appsync::ID::new();
        let p1 = Player {
            id,
            name: "Alice".to_string(),
            team: Team::Rust,
        };
        let p2 = Player {
            id,
            name: "Alice".to_string(),
            team: Team::Rust,
        };
        let p3 = Player {
            id: lambda_appsync::ID::new(),
            name: "Bob".to_string(),
            team: Team::Python,
        };

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn default_traits_still_present_alongside_extra_derives() {
        // Default traits (Debug, Clone, Serialize, Deserialize) are still present
        let player = Player {
            id: lambda_appsync::ID::new(),
            name: "Test".to_string(),
            team: Team::Js,
        };
        // Clone is a default trait for structs
        let cloned = player.clone();
        assert_eq!(cloned.name, player.name);

        // Serialize is a default trait
        let json = lambda_appsync::serde_json::to_value(&player).unwrap();
        assert_eq!(json["name"], "Test");
    }
}

// ── Test 2: make_types! with `default_traits = false` + explicit `derive` ────
//
// Verify that when `default_traits = Player: false`, only the explicitly listed
// derives are applied (no Clone, no Serialize, etc. unless added back).
mod no_default_traits {
    lambda_appsync::make_types!(
        "schema.graphql",
        default_traits = Player: false,
        derive = Player: Debug,
    );

    #[test]
    fn player_has_only_debug() {
        let player = Player {
            id: lambda_appsync::ID::new(),
            name: "DebugOnly".to_string(),
            team: Team::Rust,
        };
        // Debug is present — format with {:?}
        let debug_str = format!("{:?}", player);
        assert!(debug_str.contains("DebugOnly"));
    }

    // Note: we cannot test that Clone/Serialize are absent at runtime — that's a
    // compile-time property. The fact that this module compiles without Clone/Serialize
    // usage confirms the default traits were stripped.
}

// ── Test 3: make_operation! with `type_module` ────────────────────────────────
//
// Types are generated in a submodule; make_operation! uses `type_module` to
// reference them without a `use types::*` glob import.
mod type_module_test {
    mod types {
        lambda_appsync::make_types!("schema.graphql");
    }

    // make_operation! with type_module pointing at the types submodule.
    // No #[appsync_operation] here — we just verify the generated enums are correct.
    lambda_appsync::make_operation!("schema.graphql", type_module = types,);

    #[test]
    fn operation_enum_is_accessible() {
        // Verify the Operation enum and sub-enums were generated correctly
        let op = Operation::Query(QueryField::Players);
        match op {
            Operation::Query(QueryField::Players) => {}
            _ => panic!("Unexpected operation variant"),
        }
    }

    #[test]
    fn mutation_field_enum_is_accessible() {
        let op = Operation::Mutation(MutationField::CreatePlayer);
        match op {
            Operation::Mutation(MutationField::CreatePlayer) => {}
            _ => panic!("Unexpected mutation variant"),
        }
    }

    #[test]
    fn subscription_field_enum_is_accessible() {
        let op = Operation::Subscription(SubscriptionField::OnCreatePlayer);
        match op {
            Operation::Subscription(SubscriptionField::OnCreatePlayer) => {}
            _ => panic!("Unexpected subscription variant"),
        }
    }
}

// ── Test 4: make_handlers! with `operation_type` ─────────────────────────────
//
// Operation is generated in a submodule; make_handlers! uses `operation_type`
// to reference it without bringing it into scope.
mod operation_type_test {
    mod ops {
        lambda_appsync::make_types!("schema.graphql");
        lambda_appsync::make_operation!("schema.graphql");
    }

    // make_handlers! with operation_type pointing at the ops submodule.
    // No #[appsync_operation] here — we just verify the handler is callable.
    lambda_appsync::make_handlers!(operation_type = ops::Operation,);

    #[tokio::test]
    async fn service_fn_dispatches_correctly_with_custom_operation_type() {
        let event = lambda_appsync::serde_json::json!([{
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

        let lambda_event =
            lambda_appsync::lambda_runtime::LambdaEvent::new(event, Default::default());
        let response = DefaultHandlers::service_fn(lambda_event).await.unwrap();

        // No handlers registered → Unimplemented error
        let response_value = lambda_appsync::serde_json::to_value(response).unwrap();
        assert_eq!(response_value[0]["errorType"], "Unimplemented");
    }
}

// ── Test 5: make_handlers! with batch = false and operation_type ──────────────
mod non_batch_with_operation_type {
    mod ops {
        lambda_appsync::make_types!("schema.graphql");
        lambda_appsync::make_operation!("schema.graphql");
    }

    lambda_appsync::make_handlers!(batch = false, operation_type = ops::Operation,);

    #[tokio::test]
    async fn non_batch_service_fn_returns_single_response() {
        use lambda_appsync::AppsyncResponse;

        let event = lambda_appsync::serde_json::json!({
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
        });

        let lambda_event =
            lambda_appsync::lambda_runtime::LambdaEvent::new(event, Default::default());
        // Non-batch returns AppsyncResponse (not Vec<AppsyncResponse>)
        let response: AppsyncResponse = DefaultHandlers::service_fn(lambda_event).await.unwrap();

        // No handlers registered → Unimplemented error
        let value = lambda_appsync::serde_json::to_value(&response).unwrap();
        assert_eq!(value["errorType"], "Unimplemented");
    }
}

// ── Test 6: Root-level composable flow tests ──────────────────────────────────
//
// These tests use the root-level make_types! + make_operation! + make_handlers!
// invocations and the #[appsync_operation] handlers defined above.

#[tokio::test]
async fn composable_flow_handles_query() {
    let event = lambda_appsync::serde_json::json!([{
        "info": {
            "fieldName": "player",
            "parentTypeName": "Query",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": lambda_appsync::ID::new().to_string()
        },
        "identity": null,
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_appsync::lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = DefaultHandlers::service_fn(lambda_event).await.unwrap();

    let response_value = lambda_appsync::serde_json::to_value(response).unwrap();
    // player returns None for a nonexistent ID → data is null
    assert!(response_value[0]["data"].is_null());
}

#[tokio::test]
async fn composable_flow_unimplemented_returns_error() {
    let event = lambda_appsync::serde_json::json!([{
        "info": {
            "fieldName": "setGameStatus",
            "parentTypeName": "Mutation",
            "variables": {},
            "selectionSetList": [],
            "selectionSetGraphQL": ""
        },
        "arguments": {},
        "identity": null,
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_appsync::lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = DefaultHandlers::service_fn(lambda_event).await.unwrap();

    let response_value = lambda_appsync::serde_json::to_value(response).unwrap();
    assert_eq!(response_value[0]["errorType"], "Unimplemented");
}

#[tokio::test]
async fn composable_flow_get_players_returns_empty_array() {
    let event = lambda_appsync::serde_json::json!([{
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

    let response_value = lambda_appsync::serde_json::to_value(response).unwrap();
    assert!(response_value[0].get("data").is_some_and(|v| v.is_array()));
}
