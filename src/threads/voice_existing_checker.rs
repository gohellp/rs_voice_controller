use std::{sync::{atomic::{AtomicU64, Ordering}, Arc}, time::Duration};

use poise::serenity_prelude::{CacheHttp, ChannelId, GuildId};
use rand::seq::IteratorRandom;
use sqlx::{query_as, SqlitePool};
use tokio::time::interval;

use crate::models::VoiceInfo;

static INSTANCE_ID: AtomicU64 = AtomicU64::new(0);

pub async fn voice_exist_checker(
    ctx: &Arc<poise::serenity_prelude::Context>,
    pool: &SqlitePool,
    guilds: &Vec<GuildId>,
    guild_id: u64
) {
    let this_id = INSTANCE_ID.fetch_add(1, Ordering::AcqRel);

    let mut interval_5m = interval(Duration::from_secs(300));
    let http = ctx.http();

    tracing::info!("Checker start working!");

    loop {
        let current_id = INSTANCE_ID.load(Ordering::Acquire);
        if !current_id > this_id {
            tracing::debug!("Voice checker instance {} detected newer instance {}, exiting", this_id, current_id);
            break;
        }

        let rows: Vec<VoiceInfo> =
            query_as("select * from voices_info where expire >= now()")
                .fetch_all(pool)
                .await.unwrap();

        for item in rows {
            let current_id = INSTANCE_ID.load(Ordering::Acquire);
            if !current_id > this_id {
                tracing::debug!("Voice checker instance {} detected newer instance {}, exiting", this_id, current_id);
                break;
            }

            let Some(guild) = guilds.iter()
                .find(
                    |gid|
                        gid.get() == guild_id
                ).map_or_else(
                    ||
                    None,
                    |gid|
                        ctx.cache.guild(gid)
                )
            else {
                tracing::warn!("Guild somehow not found lol");
                item.delete(pool).await.unwrap();
                continue;
            };

            let Some(channel) = guild.channels.get(
                    &ChannelId::new(item.voice_id.parse::<u64>().unwrap())
                ) else {
                    item.delete(pool).await.unwrap();
                    continue;
                };

            let Ok(members) = channel.members(&ctx)
                else {
                    continue;
                };
            let mut members_iter = members.iter();

            // If owner leave his channel
            if !members_iter.any(
                |m| m.user.id == item.owner_id.parse::<u64>().unwrap()
            )
            {
                // no one here :c
                if members.len() == 0 {
                    let result = http.delete_channel(
                        channel.id,
                        Some("No one here :c")
                    ).await;

                    match result {
                        Ok(_) => (),
                        Err(e) => tracing::error!("Failed to delete channel: {}", e),
                    }

                    item.delete(pool).await.unwrap();

                    continue;
                }

                let new_owner;
                {
                    let mut rng = rand::rng();
                    // If there is more than 0 members select new owner
                    let Some(new_owner_) = members_iter.choose(&mut rng) else {
                            tracing::warn!("Somehow no members found lol");
                            continue;
                        };
                    new_owner = new_owner_;
                }

                item.change_owner(new_owner.user.id.get().to_string(), pool).await;
            }
        }

        interval_5m.tick().await;
    }

    tracing::debug!("Voice checker instance {} exiting", this_id);
}
