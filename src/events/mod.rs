mod ready;
mod voice_state_updated_connect;
mod voice_state_updated_disconnect;

use ready::ready;
use voice_state_updated_connect::voice_state_update_connect;
use voice_state_updated_disconnect::voice_state_update_disconnect;

use crate::structs::{
    FrameworkContext, Result
};
use poise::serenity_prelude::FullEvent as Event;


pub async fn event_handler(
    ctx: &poise::serenity_prelude::Context,
    event: &Event,
    framework_ctx: FrameworkContext<'_>
) -> Result<()> {
    match event {
        Event::Ready { data_about_bot } => {
            tracing::info!("Logged in as {}", data_about_bot.user.name);
            ready(framework_ctx, ctx, data_about_bot).await?;
        }
        Event::VoiceStateUpdate { old, new } => {
            voice_state_update_connect(framework_ctx, ctx, old.as_ref(), new).await?;
            voice_state_update_disconnect(framework_ctx, ctx, old.as_ref(), new).await?;
        }
        _ => {},
    }
    Ok(())
}
