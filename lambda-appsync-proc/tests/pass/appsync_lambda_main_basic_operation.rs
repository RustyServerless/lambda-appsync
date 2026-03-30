use lambda_appsync::{
    appsync_lambda_main, appsync_operation, subscription_filters::FilterGroup, AppsyncError, ID,
};

appsync_lambda_main!("../../../../schema.graphql", exclude_lambda_handler = true);

#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    Ok(vec![])
}

#[appsync_operation(query(gameStatus))]
async fn get_game_status() -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}

#[appsync_operation(query(player))]
async fn get_player(_id: ID) -> Result<Option<Player>, AppsyncError> {
    Ok(None)
}

#[appsync_operation(mutation(createPlayer))]
async fn create_player(name: String) -> Result<Player, AppsyncError> {
    Ok(Player {
        id: ID::new(),
        name,
        team: Team::Rust,
    })
}

#[appsync_operation(mutation(deletePlayer))]
async fn delete_player(id: ID) -> Result<Player, AppsyncError> {
    Ok(Player {
        id,
        name: "deleted".into(),
        team: Team::MultiWordsTeam,
    })
}

#[appsync_operation(mutation(setGameStatus))]
async fn set_game_status() -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}

#[appsync_operation(subscription(onCreatePlayer))]
async fn on_create_player(_name: String) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(None)
}

#[appsync_operation(subscription(onDeletePlayer))]
async fn on_delete_player(_id: ID) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(None)
}

#[appsync_operation(subscription(onGameStatusChange))]
async fn on_game_status_change() -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(None)
}

fn main() {}
