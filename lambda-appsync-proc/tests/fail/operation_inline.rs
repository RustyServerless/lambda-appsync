use lambda_appsync::{
    appsync_operation, make_appsync,
    subscription_filters::{FieldPath, FilterGroup},
    AppsyncError, ID,
};

make_appsync!("../../../../schema.graphql");
fn main() {}

// Query operations
#[appsync_operation(query(players), inline_and_remove)]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    Ok(vec![])
}

#[appsync_operation(query(gameStatus), inline_and_remove)]
async fn get_game_status() -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}

#[appsync_operation(query(player), inline_and_remove)]
async fn get_player(id: ID) -> Result<Option<Player>, AppsyncError> {
    Ok(Some(Player {
        id,
        name: "Test".into(),
        team: Team::Rust,
    }))
}

// Mutation operations
#[appsync_operation(mutation(createPlayer), inline_and_remove)]
async fn create_player(name: String) -> Result<Player, AppsyncError> {
    Ok(Player {
        id: ID::new(),
        name,
        team: Team::Rust,
    })
}

#[appsync_operation(mutation(deletePlayer), inline_and_remove)]
async fn delete_player(id: ID) -> Result<Player, AppsyncError> {
    Ok(Player {
        id,
        name: "Deleted".into(),
        team: Team::MultiWordsTeam,
    })
}

#[appsync_operation(mutation(setGameStatus), inline_and_remove)]
async fn set_game_status() -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}

// Subscription operations
#[appsync_operation(subscription(onCreatePlayer), inline_and_remove)]
async fn on_create_player(name: String) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(FieldPath::new("name")?.contains(name).into()))
}

#[appsync_operation(subscription(onDeletePlayer), inline_and_remove)]
async fn on_delete_player(id: ID) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(FieldPath::new("id")?.eq(id).into()))
}

#[appsync_operation(subscription(onGameStatusChange), inline_and_remove)]
async fn on_game_status_change() -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(None)
}

// Test that we cannot call the original functions
async fn test_original_fn() {
    let _ = create_player("test".to_string()).await;
    let _ = get_players().await;
    let _ = get_game_status().await;
    let _ = get_player(ID::new()).await;
    let _ = delete_player(ID::new()).await;
    let _ = set_game_status().await;
    let _ = on_create_player("test".to_string()).await;
    let _ = on_delete_player(ID::new()).await;
    let _ = on_game_status_change().await;
}
