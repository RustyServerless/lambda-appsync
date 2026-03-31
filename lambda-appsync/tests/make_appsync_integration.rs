use std::{cell::RefCell, collections::HashMap, ops::Deref};

use lambda_appsync::{appsync_operation, make_appsync, AppsyncError, AppsyncResponse, ID};
use serde_json::json;

thread_local! {
    static TEST_DB: InnerDatabase = InnerDatabase::new();
}

struct InnerDatabase(RefCell<HashMap<ID, Player>>);
impl InnerDatabase {
    pub fn new() -> Self {
        Self(RefCell::new(HashMap::new()))
    }

    fn get(&self, id: &ID) -> Option<Player> {
        self.0.borrow().get(id).cloned()
    }

    fn insert(&self, id: ID, player: Player) -> Option<Player> {
        self.0.borrow_mut().insert(id, player)
    }

    fn remove(&self, id: &ID) -> Option<Player> {
        self.0.borrow_mut().remove(id)
    }

    fn values(&self) -> Vec<Player> {
        self.0.borrow().values().cloned().collect()
    }
}
struct Database;
impl Deref for Database {
    type Target = InnerDatabase;

    fn deref(&self) -> &Self::Target {
        // Safety: This is safe because we're only accessing the thread-local
        // storage, which is guaranteed to exist for the duration of the thread
        TEST_DB.with(|db| unsafe {
            // Convert the reference to 'static because we know the thread-local
            // storage will live for the remainder of the program
            std::mem::transmute::<&InnerDatabase, &'static InnerDatabase>(db)
        })
    }
}

// Generate AppSync types, Operation enum, and Handlers trait from schema
make_appsync!("schema.graphql");

// Operation Handlers
#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    Ok(Database.values())
}

#[appsync_operation(query(player))]
async fn get_player(id: ID) -> Result<Option<Player>, AppsyncError> {
    Ok(Database.get(&id))
}

#[appsync_operation(mutation(createPlayer))]
async fn create_player(name: String) -> Result<Player, AppsyncError> {
    let player = Player {
        id: ID::new(),
        name,
        team: Team::Rust,
    };

    Database.insert(player.id, player.clone());

    Ok(player)
}

#[appsync_operation(mutation(deletePlayer))]
async fn delete_player(id: ID) -> Result<Player, AppsyncError> {
    Database
        .remove(&id)
        .ok_or_else(|| AppsyncError::new("NotFound", "Player not found"))
}

// Tests
#[tokio::test]
async fn create_and_get_player() {
    // Create player
    let create_event = json!([{
        "info": {
            "fieldName": "createPlayer",
            "parentTypeName": "Mutation",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "name": "Test Player"
        },
        "identity": null,
        "request": null,
        "source": null
    }]);

    let create_lambda_event =
        lambda_appsync::lambda_runtime::LambdaEvent::new(create_event, Default::default());
    let create_response = DefaultHandlers::service_fn(create_lambda_event)
        .await
        .unwrap();

    let response_value = serde_json::to_value(create_response).unwrap();
    let player_id = response_value[0]["data"]["id"].as_str().unwrap();

    // Get created player
    let get_event = json!([{
        "info": {
            "fieldName": "player",
            "parentTypeName": "Query",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": player_id
        },
        "identity": null,
        "request": null,
        "source": null
    }]);

    let get_lambda_event =
        lambda_appsync::lambda_runtime::LambdaEvent::new(get_event, Default::default());
    let get_response = DefaultHandlers::service_fn(get_lambda_event).await.unwrap();

    let response_value = serde_json::to_value(get_response).unwrap();
    assert_eq!(response_value[0]["data"]["name"], "Test Player");
}

#[tokio::test]
async fn get_nonexistent_player() {
    let event = json!([{
        "info": {
            "fieldName": "player",
            "parentTypeName": "Query",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": ID::new().to_string()
        },
        "identity": null,
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_appsync::lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = DefaultHandlers::service_fn(lambda_event).await.unwrap();

    let response_value = serde_json::to_value(response).unwrap();
    assert!(response_value[0]["data"].is_null());
}

#[tokio::test]
async fn unimplemented_operation() {
    let set_status_event = json!([{
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

    let set_status_lambda_event =
        lambda_appsync::lambda_runtime::LambdaEvent::new(set_status_event, Default::default());
    let set_status_response = DefaultHandlers::service_fn(set_status_lambda_event)
        .await
        .unwrap();

    let response_value = serde_json::to_value(set_status_response).unwrap();
    assert_eq!(response_value[0]["errorType"], "Unimplemented");
    assert_eq!(
        response_value[0]["errorMessage"],
        "Mutation `setGameStatus` is unimplemented"
    );
}

#[tokio::test]
async fn delete_nonexistent_player() {
    let event = json!([{
        "info": {
            "fieldName": "deletePlayer",
            "parentTypeName": "Mutation",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": ID::new().to_string()
        },
        "identity": null,
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_appsync::lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = DefaultHandlers::service_fn(lambda_event).await.unwrap();

    let response_value = serde_json::to_value(response).unwrap();
    assert_eq!(response_value[0]["errorType"], "NotFound");
    assert_eq!(response_value[0]["errorMessage"], "Player not found");
}

#[tokio::test]
async fn batch_create_multiple_players() {
    let names = ["Batch Player 1", "Batch Player 2", "Batch Player 3"];

    // Create multiple players in a single batch request
    let create_event = json!(names
        .iter()
        .map(|name| json!({
            "info": {
                "fieldName": "createPlayer",
                "parentTypeName": "Mutation",
                "variables": {},
                "selectionSetList": ["id", "name", "team"],
                "selectionSetGraphQL": "{id name team}"
            },
            "arguments": {
                "name": name
            },
            "identity": null,
            "request": null,
            "source": null
        }))
        .collect::<Vec<_>>());

    let create_lambda_event =
        lambda_appsync::lambda_runtime::LambdaEvent::new(create_event, Default::default());
    let response = DefaultHandlers::service_fn(create_lambda_event)
        .await
        .unwrap();

    // Verify each creation was successful and names match
    let response_values = serde_json::to_value(response).unwrap();
    assert_eq!(response_values.as_array().unwrap().len(), names.len());
    for (response_value, name) in response_values.as_array().unwrap().iter().zip(names.iter()) {
        assert_eq!(response_value["data"]["name"], *name);
    }
}

#[tokio::test]
async fn get_all_players_returns_array() {
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

    let response_value = serde_json::to_value(response).unwrap();
    assert!(response_value[0].get("data").is_some_and(|v| v.is_array()));
}

#[tokio::test]
async fn service_fn_returns_vec_of_responses() {
    // Verify that batch mode returns Vec<AppsyncResponse>
    let event = json!([
        {
            "info": {
                "fieldName": "createPlayer",
                "parentTypeName": "Mutation",
                "variables": {},
                "selectionSetList": ["id", "name", "team"],
                "selectionSetGraphQL": "{id name team}"
            },
            "arguments": { "name": "Alpha" },
            "identity": null,
            "request": null,
            "source": null
        },
        {
            "info": {
                "fieldName": "createPlayer",
                "parentTypeName": "Mutation",
                "variables": {},
                "selectionSetList": ["id", "name", "team"],
                "selectionSetGraphQL": "{id name team}"
            },
            "arguments": { "name": "Beta" },
            "identity": null,
            "request": null,
            "source": null
        }
    ]);

    let lambda_event = lambda_appsync::lambda_runtime::LambdaEvent::new(event, Default::default());
    let responses: Vec<AppsyncResponse> = DefaultHandlers::service_fn(lambda_event).await.unwrap();

    assert_eq!(responses.len(), 2);
    let values = serde_json::to_value(&responses).unwrap();
    assert_eq!(values[0]["data"]["name"], "Alpha");
    assert_eq!(values[1]["data"]["name"], "Beta");
}
