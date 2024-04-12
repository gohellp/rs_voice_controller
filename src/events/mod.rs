mod voice_state;

use voice_state::*;

use crate::structs::{
    Result,
    FrameworkContext
};
use poise::serenity_prelude::{
    CacheHttp,
    FullEvent as Event, GuildId
};


pub async fn event_handler(
    ctx: &poise::serenity_prelude::Context,
    event: &Event,
    framework_ctx: FrameworkContext<'_>
) -> Result<()> {
    match event {
        Event::Ready { data_about_bot } => {
            tracing::info!("Logged in as {}", data_about_bot.user.name);
            let permissions = &ctx.http
                .get_guild(
                    GuildId::new(framework_ctx.user_data.guild_id)
                )
                .await?
                .member(
                    ctx.http(),
                    data_about_bot.user.id
                ).
                await?
                .permissions(&ctx.cache)
                .unwrap();

            if !permissions.manage_channels() || !permissions.move_members() {
                tracing::error!("Bot can't work without MANAGE_CHANNELS and MOVE_MEMBERS permission!!!");
                std::process::exit(1);
            }
        }
        Event::VoiceStateUpdate { old, new } => {
            _ = voice_state_update( framework_ctx, ctx, old.as_ref(), new).await;
        }
        _ => {},
    }
    Ok(())
}