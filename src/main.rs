mod events;
mod structs;

use events::*;
use rs_voice_controller::database_connect;
use structs::Data;
use poise::{
    FrameworkOptions,
    serenity_prelude::{
        ClientBuilder,
        GatewayIntents
    }
};
use std::env::var;


#[tokio::main]
async fn main(){
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    
    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var");

    let pool = database_connect(
        &var("DATABASE_URL").expect("Missing `DATABASE_URL` env var")
    ).await.unwrap();

    let intents =
        GatewayIntents::non_privileged()
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_VOICE_STATES;

    let framework = poise::Framework::builder()
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    guild_id: var("GUILD_ID").expect("guild_id").parse::<u64>().expect("u64 guild_id"),
                    voice_id: var("GUILD_VOICE_ID").expect("voice_id").parse::<u64>().expect("u64 voice_id"),
                    pool: pool
                })
            })
        })
        .options(FrameworkOptions {
            event_handler: |ctx, event, framework, _data| {
                Box::pin(event_handler(ctx, event, framework))
            },
            
            ..Default::default()
        })
        .build();

    let mut client = ClientBuilder::new(token, intents) 
        .framework(framework)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        tracing::error!("Client error: {:?}", why);
    };
}