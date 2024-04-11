mod voice_state;

use voice_state::*;

use crate::structs::{
    Result,
    FrameworkContext
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
        }
        Event::VoiceStateUpdate { old, new } => {
            _ = voice_state_update( framework_ctx, ctx, old.as_ref(), new).await;
        }
        _ => {},
    }
    Ok(())
}