use crate::models::{User, VoiceInfo};
use poise::serenity_prelude::*;
use rand::seq::IndexedRandom;
use sqlx::query_as;

pub async fn voice_state_update_connect(
    framework_ctx: crate::structs::FrameworkContext<'_>,
    ctx: &poise::serenity_prelude::Context,
    old_wrapped: Option<&VoiceState>,
    new: &VoiceState,
) -> anyhow::Result<()> {
    let data = framework_ctx.user_data;
    let pool = &data.pool;
    let member = &mut new.member.clone().unwrap();
    let new_state_channel_id = &new.channel_id.unwrap_or_default();
    let guild = Guild::get(&ctx, new.guild_id.unwrap()).await?;

    let start_id = framework_ctx.user_data.voice_id;

    if new.guild_id.unwrap()!=data.guild_id {
        return Ok(());
    }

    //optional var
    let parent_id = std::env::var("PARENT_CHANNEL_ID")
        .unwrap_or(
            {
                let Some(guild) = ctx.cache.guild(data.guild_id) else {
                    return Ok(());
                };

                guild.channels
                    .get(&ChannelId::new(data.voice_id))
                    .unwrap()
                    .parent_id
                    .unwrap()
                    .to_string()

            }
        ).parse::<u64>()
        .expect("u64 parent_id");

    let user_info: User =
        query_as(
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
            "#
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

    if let Some(old) = old_wrapped {
        if let Some(old_channel_id) = old.channel_id {
            if old_channel_id == *new_state_channel_id{
                return Ok(());
            }
            if let Some(voice_info) = voice_info_wrapped {
                if voice_info.voice_id == old_channel_id.get().to_string() {
                    if user_info.return_to_owned_channel {
                        member.move_to_voice_channel(ctx, old_channel_id).await?;

                        tracing::info!(
                            "user {} moved to voice channel {}",
                            member.user.id,
                            old_channel_id
                        );

                        return Ok(());
                    } else {
                        let members: Vec<UserId> = {
                            let Some(guild) = ctx.cache.guild(guild.id) else {
                                return Ok(());
                            };

                            guild
                                .voice_states
                                .iter()
                                .filter(
                                    |(_, state)|
                                        state.channel_id.is_some_and(
                                            |cid|
                                                cid == old_channel_id
                                        )
                                )
                                .map(|(user_id, _)| *user_id)
                                .collect()
                        };

                        if members.len() >= 1 {
                            let new_owner_id = members.choose(
                                &mut rand::rng()
                            ).unwrap();
                            let new_owner = new_owner_id.to_user(ctx).await?;

                            let new_voice_info = voice_info
                                .change_owner(new_owner.id.get().to_string(), &pool)
                                .await;

                            let permissions = vec![PermissionOverwrite {
                                allow: Permissions::MANAGE_CHANNELS
                                    | Permissions::MUTE_MEMBERS
                                    | Permissions::DEAFEN_MEMBERS,
                                deny: Permissions::empty(),
                                kind: PermissionOverwriteType::Member(new_owner.id),
                            }];
                            let name = format!("{}'s channel", new_owner.name);
                            let builder = EditChannel::new()
                                .name(name)
                                .permissions(permissions);

                            old_channel_id.edit(ctx, builder).await?;


                            tracing::info!(
                                "Sets owner_id to: {} for voice channel: {}",
                                new_voice_info.owner_id,
                                new_voice_info.voice_id
                            )

                        //There is no one here :c
                        } else {
                            old_channel_id.delete(ctx).await?;
                            voice_info.delete(&pool).await?;

                            tracing::info!("Deleted voice channel: {}", old_channel_id.get());
                        }
                    }
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

    let channel_builder = CreateChannel::new(
        format!(
            "{}'s channel",
            member
                .user
                .global_name
                .as_ref()
                .unwrap_or(&member.user.name)
        )
    ).category(parent_id)
        .kind(ChannelType::Voice)
        .permissions(vec![PermissionOverwrite {
            allow: Permissions::MANAGE_CHANNELS
                | Permissions::MUTE_MEMBERS
                | Permissions::DEAFEN_MEMBERS,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member(member.user.id),
        }])
        .audit_log_reason(reason.as_str());

    let new_channel = guild.id.create_channel(
        &ctx,
        channel_builder
    ).await?;

    tracing::debug!("Created voice channel {} in discord", new_channel.id);

    member.move_to_voice_channel(&ctx, new_channel.clone()).await?;

    tracing::debug!(
        "user {} moved to voice channel {}",
        member.user.id,
        new_channel.id
    );

    //Send data to db
    let new_voice_info =
        VoiceInfo::new(
            new_channel.id.to_string(),
            member.user.id.get().to_string(),
            &pool
        ).await;

    tracing::info!("Created voice info with id: {}", new_voice_info.voice_id);


    Ok(())
}
