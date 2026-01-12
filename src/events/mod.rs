mod ready;
mod voice_state_updated_connect;
mod voice_state_updated_disconnect;

use std::sync::Arc;

use ready::ready;
use voice_state_updated_connect::voice_state_update_connect;
use voice_state_updated_disconnect::voice_state_update_disconnect;

use crate::{
    structs::{FrameworkContext, Result},
    threads::voice_exist_checker,
};
use poise::serenity_prelude::FullEvent as Event;

pub async fn event_handler(
    ctx: &poise::serenity_prelude::Context,
    event: &Event,
    framework_ctx: FrameworkContext<'_>,
) -> Result<()> {
    match event {
        Event::Ready { data_about_bot } => {
            tracing::info!("Logged in as {}", data_about_bot.user.name);
            ready(framework_ctx, ctx, data_about_bot).await?;
        }
        Event::CacheReady { guilds } => {
            let ctx_arc = Arc::new(ctx.clone());
            let ctx_clone = Arc::clone(&ctx_arc);
            let pool_arc = Arc::new(framework_ctx.user_data.pool.clone());
            let pool_clone = Arc::clone(&pool_arc);
            let guilds_clone = guilds.clone();
            let guild_id = framework_ctx.user_data.guild_id.clone();

            tokio::spawn(async move {
                _ = voice_exist_checker(&ctx_clone, &pool_clone, &guilds_clone, guild_id);
            });
        }
        Event::VoiceStateUpdate { old, new } => {
            if let Some(old_unwrapped) = old {
                if old_unwrapped
                    .channel_id
                    .is_some_and(|cid| cid == new.channel_id.unwrap_or_default())
                {
                    return Ok(());
                }

                if old_unwrapped.channel_id.is_some() {
                    voice_state_update_disconnect(framework_ctx, ctx, old_unwrapped).await?;
                }
            }

            if new.channel_id.is_some() {
                voice_state_update_connect(framework_ctx, ctx, new).await?;
            }
        }
        _ => {}
    }
    Ok(())
}
