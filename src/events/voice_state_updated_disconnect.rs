use poise::serenity_prelude::*;
use sqlx::query_as;

use crate::{models::VoiceInfo, utils::*};

pub async fn voice_state_update_disconnect(
    framework_ctx: crate::structs::FrameworkContext<'_>,
    ctx: &poise::serenity_prelude::Context,
    old: &VoiceState,
) -> anyhow::Result<()> {
    let data = framework_ctx.user_data;
    let pool = &framework_ctx.user_data.pool;
    let member = &mut old.member.clone().unwrap();
    let guild = Guild::get(&ctx, old.guild_id.unwrap()).await?;

    if old.guild_id.unwrap() != data.guild_id {
        return Ok(());
    }

    let Some(old_channel_id) = old.channel_id else {
        return Ok(());
    };

    let voice_info_wrapped: Option<VoiceInfo> =
        query_as("select * from voices_info where voice_id = $1")
            .bind(old_channel_id.to_string())
            .fetch_optional(pool)
            .await?;

    if let Some(voice_info) = voice_info_wrapped {
        if member.user.id.to_string() != voice_info.owner_id {
            return Ok(());
        }

        let members: Vec<UserId> = {
            let Some(guild) = ctx.cache.guild(guild.id) else {
                return Ok(());
            };

            guild
                .voice_states
                .iter()
                .filter(|(_, state)| {
                    state
                        .channel_id
                        .is_some_and(|cid| cid == old.channel_id.unwrap())
                })
                .map(|(user_id, _)| *user_id)
                .collect()
        };

        if members.is_empty() {
            old_channel_id.delete(ctx.http()).await?;
            voice_info.delete(pool).await?;

            return Ok(());
        }

        let new_owner = select_random_owner_id(&members);
        change_owner(&ctx, &old_channel_id, *new_owner).await?;
        voice_info.change_owner(new_owner.to_string(), pool).await;
        tracing::debug!("Owner of {} was changed to {}", old_channel_id, new_owner);
    }

    if member.deaf || member.mute {
        let builder = EditMember::new().deafen(false).mute(false);
        member.edit(ctx, builder).await?;
    }

    Ok(())
}
