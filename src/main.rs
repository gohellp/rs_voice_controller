extern crate dotenv;

use dotenv::dotenv;
use std::{
    env::var,
    fmt::format
};
use poise::{
    FrameworkContext,
    FrameworkOptions,
    serenity_prelude::{
        FullEvent,
        ClientBuilder,
        GatewayIntents,
        http::CacheHttp,
        builder::CreateChannel,
        all::{
            GuildId,
            ChannelId,
            ChannelType,
        }
    }
};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;

#[allow(unused)]
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    voice_id: u64,
    guild_id: u64
}

#[tokio::main]
async fn main(){
    dotenv().ok();
    
    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var");
        
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
                    voice_id: var("GUILD_VOICE_ID").expect("voice_id").parse::<u64>().expect("u64 voice_id")
                })
            })
        })
        .options(FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            
            ..Default::default()
        })
        .build();

    let client = ClientBuilder::new(token, intents) 
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    ctx: &poise::serenity_prelude::Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        FullEvent::VoiceStateUpdate {old, new} => {
            
            let member = new.member.clone().unwrap();
            let new_state_channel = new.channel_id.unwrap_or_default();
            let guild = ctx.http.get_guild(GuildId::new(data.guild_id)).await?;
            
            //optional var
            let _parent_id = var("PARENT_CHANNEL_ID").unwrap_or(
                 guild.channels(ctx).await?.get(&ChannelId::new(data.voice_id)).expect("voice channel").parent_id.unwrap().get().to_string()
                ).parse::<u64>().expect("u64 parent_id");
            
            
            //Disconnect
            if old.is_some() {
                let old_state_channel = old.as_ref().unwrap().channel_id.unwrap();
            
                if old_state_channel.get() != data.voice_id && new_state_channel.get() != old_state_channel.get() {
                    //Get some data from db
                    
                    
                }
                
            //Connect
            } else {
                if new_state_channel.get() == data.voice_id {
                    let reason= format(format_args!("Caused by {}", member.user.name));
                    
                    let _channel_builder = 
                        CreateChannel::new(format(format_args!("{}'s channel", member.user.name)))
                        .category(_parent_id)
                        .kind(ChannelType::Voice)
                        .audit_log_reason(reason.as_str());
                    
                    let new_channel = guild.id.create_channel(&ctx.http(), _channel_builder).await?;
                    
                    _ = member.move_to_voice_channel(ctx.http(), new_channel.clone()).await?;
                    
                    //Add send data to db
                }
            }
        }
        _ => {}
    }
    Ok(())
}