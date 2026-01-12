use poise::serenity_prelude::*;

pub async fn ready(
    framework_ctx: crate::structs::FrameworkContext<'_>,
    ctx: &poise::serenity_prelude::Context,
    data_about_bot: &Ready
) -> anyhow::Result<()> {
    #[allow(deprecated)]
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

    if !(
            permissions.manage_channels()
            | permissions.move_members()
            | permissions.mute_members()
            | permissions.deafen_members()
        )
    {
        tracing::error!("Bot can't work without MANAGE_CHANNELS, MOVE_MEMBERS, MUTE_MEMBERS and DEAFEN_MEMBERS permission!!!");
        std::process::exit(1);
    }

    Ok(())
}
