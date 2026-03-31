use lambda_appsync::{
    appsync_lambda_main, appsync_operation, subscription_filters::FilterGroup, AppsyncError, ID,
};

// Test field type overrides
appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,

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

    // MUST also override ALL the operations return type !!!
    type_override = Query.players: NewPlayer,
    type_override = Query.player: NewPlayer,
    type_override = Mutation.createPlayer: NewPlayer,
    type_override = Mutation.deletePlayer: NewPlayer,
);

fn main() {
    let _weird = WeirdFieldNames {
        r#as: true,
        r#async: true,
        no_await: true, // As been renamed
        r#break: true,
        r#const: true,
        r#continue: true,
        no_crate: true, // As been renamed
        r#dyn: true,
        r#else: true,
        r#enum: true,
        r#extern: true,
        r#false: true,
        r#fn: true,
        r#for: true,
        r#if: true,
        r#impl: true,
        r#in: true,
        r#let: true,
        r#loop: true,
        r#match: true,
        r#mod: true,
        r#move: true,
        r#mut: true,
        r#pub: true,
        r#ref: true,
        r#return: true,
        r_self: true,
        r#static: true,
        r#struct: true,
        r_super: true,
        r#trait: true,
        r#true: true,
        r#type: true,
        r#unsafe: true,
        r#use: true,
        r#where: true,
        r#while: true,
        r#abstract: true,
        r#become: true,
        r#box: true,
        r#do: true,
        r#final: true,
        r#macro: true,
        r#override: true,
        r#priv: true,
        r#try: true,
        r#typeof: true,
        r#unsized: true,
        r#virtual: true,
        r#yield: true,
        bool: true,
        char: "x".into(),
        f32: 1.0,
        f64: 1.0,
        i8: 1,
        i16: 1,
        i32: 1,
        i64: 1,
        i128: 1,
        isize: 1,
        str: "string".into(),
        no_u8: 1, // As been renamed
        u16: 1,
        u32: 1,
        u64: 1,
        u128: 1,
        usize: 1,
    };
}

// Id is now a string
#[appsync_operation(query(player))]
async fn get_player(id: ID) -> Result<Option<NewPlayer>, AppsyncError> {
    Ok(Some(NewPlayer {
        id,
        email: "JohnDoe".to_string(), // Field is renamed email
        team: Team::Snake,            // Accepts "Snake"
    }))
}

// Id is now a string
#[appsync_operation(mutation(deletePlayer))]
async fn delete_player(id: ID) -> Result<NewPlayer, AppsyncError> {
    Ok(NewPlayer {
        id,
        email: "deleted".into(), // Field is renamed email
        team: Team::Rust,
    })
}

// Id is now a string
#[appsync_operation(subscription(onDeletePlayer))]
async fn on_delete_player(_id: ID) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(None)
}

// setGameStatus now expects a String
#[appsync_operation(mutation(setGameStatus))]
async fn set_game_status() -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}
