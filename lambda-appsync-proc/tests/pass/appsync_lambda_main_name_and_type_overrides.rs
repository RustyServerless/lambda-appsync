use lambda_appsync::{
    appsync_lambda_main, appsync_operation, subscription_filters::FilterGroup, AppsyncError,
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

    // MUST also override ALL the operation return type !!!
    type_override = Query.players: NewPlayer,
    type_override = Query.player: NewPlayer,
    type_override = Mutation.createPlayer: NewPlayer,
    type_override = Mutation.deletePlayer: NewPlayer,

    // TYPE OVERRIDES
    // Override Player to use String instead of ID
    type_override = Player.id: String,
    // Weird field names overrides
    type_override = WeirdFieldNames.as: String,
    type_override = WeirdFieldNames.async: String,
    type_override = WeirdFieldNames.await: String,
    type_override = WeirdFieldNames.crate: String,
    type_override = WeirdFieldNames.self: String,
    type_override = WeirdFieldNames.super: String,
    type_override = WeirdFieldNames.become: String,
    type_override = WeirdFieldNames.box: String,
    type_override = WeirdFieldNames.virtual: String,
    type_override = WeirdFieldNames.i8: String,
    type_override = WeirdFieldNames.i16: String,
    type_override = WeirdFieldNames.u8: String,
    // Return value override
    type_override = Query.gameStatus: String,
    type_override = Mutation.setGameStatus: String,
    // Argument override
    type_override = Query.player.id: String,
    type_override = Mutation.deletePlayer.id: String,
    type_override = Subscription.onDeletePlayer.id: String,
);

fn main() {
    let _weird = WeirdFieldNames {
        r#as: "test".into(),
        r#async: "test".into(),
        no_await: "test".into(), // As been renamed AND type changed
        r#break: true,
        r#const: true,
        r#continue: true,
        no_crate: "test".into(), // As been renamed AND type changed
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
        r_self: "test".into(),
        r#static: true,
        r#struct: true,
        r_super: "test".into(),
        r#trait: true,
        r#true: true,
        r#type: true,
        r#unsafe: true,
        r#use: true,
        r#where: true,
        r#while: true,
        r#abstract: true,
        r#become: "test".into(),
        r#box: "test".into(),
        r#do: true,
        r#final: true,
        r#macro: true,
        r#override: true,
        r#priv: true,
        r#try: true,
        r#typeof: true,
        r#unsized: true,
        r#virtual: "test".into(),
        r#yield: true,
        bool: true,
        char: "x".into(),
        f32: 1.0,
        f64: 1.0,
        i8: "test".into(),
        i16: "test".into(),
        i32: 1,
        i64: 1,
        i128: 1,
        isize: 1,
        str: "string".into(),
        no_u8: "test".into(), // As been renamed AND type changed
        u16: 1,
        u32: 1,
        u64: 1,
        u128: 1,
        usize: 1,
    };
}

// Id is now a string
#[appsync_operation(query(player))]
async fn get_player(id: String) -> Result<Option<NewPlayer>, AppsyncError> {
    Ok(Some(NewPlayer {
        id,
        email: "JohnDoe".to_string(), // Field is renamed email
        team: Team::Snake,            // Accepts "Snake"
    }))
}

// Id is now a string
#[appsync_operation(mutation(deletePlayer))]
async fn delete_player(id: String) -> Result<NewPlayer, AppsyncError> {
    Ok(NewPlayer {
        id,
        email: "deleted".into(), // Field is renamed email
        team: Team::Rust,
    })
}

// Id is now a string
#[appsync_operation(subscription(onDeletePlayer))]
async fn on_delete_player(_id: String) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(None)
}

// setGameStatus now expects a String
#[appsync_operation(mutation(setGameStatus))]
async fn set_game_status() -> Result<String, AppsyncError> {
    Ok("Started".to_owned())
}
