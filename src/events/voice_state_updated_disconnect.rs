use poise::serenity_prelude::*;
use rand::seq::IndexedRandom;
use sqlx::query_as;

use crate::models::VoiceInfo;

pub async fn voice_state_update_disconnect(
    framework_ctx: crate::structs::FrameworkContext<'_>,
    ctx: &poise::serenity_prelude::Context,
    old_wrapped: Option<&VoiceState>,
    new: &VoiceState,
) -> anyhow::Result<()> {
    if None == new.guild_id {
        return Ok(());
    }

    let data = framework_ctx.user_data;
    let pool = &framework_ctx.user_data.pool;
    let member = &mut new.member.clone().unwrap();
    let guild = Guild::get(&ctx, new.guild_id.unwrap()).await?;

    let start_id = framework_ctx.user_data.voice_id;

    if new.guild_id.unwrap()!=data.guild_id {
        return Ok(());
    }


    if let Some(old) = old_wrapped {
        if let Some(old_channel_id) = old.channel_id {
            if let Some(new_channel_id) = new.channel_id {
                if old_channel_id == new_channel_id{
                    return Ok(());
                }
            }

            tracing::info!(
                "User disconnected from voice channel: {}",
                old_channel_id.get()
            );

            if old_channel_id.get() == start_id {
                return Ok(());
            }

            let voice_info_wrapped: Option<VoiceInfo> =
                query_as("SELECT * FROM voices_info WHERE voice_id = $1")
                    .bind(old_channel_id.get().to_string())
                    .fetch_optional(pool)
                    .await?;

            if let Some(voice_info) = voice_info_wrapped {
                if member.user.id.get().to_string() != voice_info.owner_id.clone() {
                    return Ok(());
                }

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

            if member.deaf || member.mute {
                let builder = EditMember::new().deafen(false).mute(false);
                member.edit(ctx, builder).await?;
            }
        }
    }

    Ok(())
}
