use crate::{
    models::{User, VoiceInfo},
    utils::*,
};
use poise::serenity_prelude::*;
use sqlx::query_as;

pub async fn voice_state_update_connect(
    framework_ctx: crate::structs::FrameworkContext<'_>,
    ctx: &poise::serenity_prelude::Context,
    new: &VoiceState,
) -> anyhow::Result<()> {
    let data = framework_ctx.user_data;
    let pool = &data.pool;
    let Some(member) = &mut new.member.clone() else {
        tracing::error!("Member somehow is None in new: VoiceState");
        return Ok(());
    };
    let new_state_channel_id = &new.channel_id.unwrap_or_default();
    let Some(new_guild_id) = new.guild_id else {
        tracing::error!("GuildId somehow is None in new: VoiceState");
        return Ok(());
    };
    let guild = Guild::get(&ctx, new_guild_id).await?;

    let start_id = framework_ctx.user_data.voice_id;

    if new_guild_id != data.guild_id {
        return Ok(());
    }

    let parent_id = std::env::var("PARENT_CHANNEL_ID")
        .unwrap_or({
            let Some(guild) = ctx.cache.guild(data.guild_id) else {
                return Ok(());
            };

            guild
                .channels
                .get(&ChannelId::new(data.voice_id))
                .unwrap()
                .parent_id
                .unwrap()
                .to_string()
        })
        .parse::<u64>()
        .expect("u64 parent_id");

    let user_info: User = query_as(
        r#"
            SELECT * FROM users WHERE id = $1

            UNION ALL

            SELECT
                $1 as id,
                false as return_to_owned_channel

            WHERE NOT EXISTS (
                SELECT 1
                FROM users
                WHERE id = $1
            );
            "#,
    )
    .bind(member.user.id.get().to_string())
    .fetch_one(pool)
    .await?;

    tracing::debug!(
        "User connected to voice channel: {}",
        new_state_channel_id.get()
    );

    if new_state_channel_id.get() != start_id {
        return Ok(());
    }

    let voice_info_wrapped: Option<VoiceInfo> =
        query_as("SELECT * FROM voices_info WHERE owner_id = $1")
            .bind(member.user.id.get() as i64)
            .fetch_optional(pool)
            .await?;

    if let Some(voice_info) = voice_info_wrapped {
        let channels: Vec<ChannelId> = {
            let Some(guild_cache) = ctx.cache.guild(guild.id) else {
                return Ok(());
            };

            guild_cache
                .channels
                .iter()
                .map(|(channel_id, _)| *channel_id)
                .collect()
        };

        if channels.contains(&ChannelId::new(voice_info.voice_id.parse::<u64>()?)) {
            let members_vi: Vec<UserId> = {
                let Some(guild_cache) = ctx.cache.guild(guild.id) else {
                    return Ok(());
                };

                guild_cache
                    .voice_states
                    .iter()
                    .filter(|(_, state)| {
                        state
                            .channel_id
                            .is_some_and(|cid| cid.get().to_string() == voice_info.voice_id)
                    })
                    .map(|(uiser_id, _)| *uiser_id)
                    .collect()
            };

            if members_vi.len() == 0 {
                member
                    .move_to_voice_channel(ctx.http(), voice_info.voice_id.parse::<u64>()?)
                    .await?;

                tracing::debug!(
                    "User {} returned to his channel {}",
                    member.user.id,
                    voice_info.voice_id
                );

                return Ok(());
            } else {
                if user_info.return_to_owned_channel {
                    member
                        .move_to_voice_channel(ctx.http(), voice_info.voice_id.parse::<u64>()?)
                        .await?;

                    tracing::debug!(
                        "User {} returned to his channel {}",
                        member.user.id,
                        voice_info.voice_id
                    );
                } else {
                    let channel_id = ChannelId::new(voice_info.voice_id.parse::<u64>()?);
                    let new_owner = select_random_owner_id(&members_vi);
                    change_owner(&ctx, &channel_id, *new_owner).await?;
                    voice_info
                        .change_owner(new_owner.get().to_string(), pool)
                        .await;
                    tracing::debug!("Owner of {} was changed to {}", channel_id, new_owner);
                }
            }
        }
    }

    let reason = format!(
        "User {} created voice chat",
        member
            .user
            .global_name
            .as_ref()
            .unwrap_or(&member.user.name)
    );

    let channel_builder = CreateChannel::new(format!(
        "{}'s channel",
        member
            .user
            .global_name
            .as_ref()
            .unwrap_or(&member.user.name)
    ))
    .category(parent_id)
    .kind(ChannelType::Voice)
    .permissions(vec![PermissionOverwrite {
        allow: Permissions::MANAGE_CHANNELS
            | Permissions::MUTE_MEMBERS
            | Permissions::DEAFEN_MEMBERS,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Member(member.user.id),
    }])
    .audit_log_reason(reason.as_str());

    let new_channel = guild.id.create_channel(&ctx, channel_builder).await?;

    tracing::debug!("Created voice channel {} in discord", new_channel.id);

    member
        .move_to_voice_channel(&ctx, new_channel.clone())
        .await?;

    tracing::debug!(
        "user {} moved to voice channel {}",
        member.user.id,
        new_channel.id
    );

    //Send data to db
    let new_voice_info = VoiceInfo::new(
        new_channel.id.get().to_string(),
        member.user.id.get().to_string(),
        &pool,
    )
    .await;

    tracing::info!("Created voice info with id: {}", new_voice_info.voice_id);

    Ok(())
}
