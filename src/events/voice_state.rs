use std::fmt::format;
use poise::serenity_prelude::{
    CacheHttp,
    builder::CreateChannel,
    all::{
        GuildId,
        ChannelId,
        ChannelType,
        VoiceState
    }
};
use rand::seq::SliceRandom;
use rs_voice_controller::models::VoicesInfo;


pub async fn voice_state_update(
    framework_ctx: crate::structs::FrameworkContext<'_>,
    ctx: &poise::serenity_prelude::Context,
    old: Option<&VoiceState>,
    new: &VoiceState,
) -> crate::structs::Result<()> {
    let data = framework_ctx.user_data;

    let pool = data.pool.clone();
    let member = new.member.as_ref().unwrap();
    let new_state_channel = &new.channel_id.unwrap_or_default();
    let guild = &ctx.http.get_guild(GuildId::new(data.guild_id)).await?;

    if guild.id.get()!=data.guild_id {
        return Ok(());
    }
    
    //optional var
    let parent_id = std::env::var("PARENT_CHANNEL_ID")
        .unwrap_or(
            guild.channels(ctx)
                .await?
                .get(
                    &ChannelId::new(data.voice_id)
                )
                .expect("voice channel")
                .parent_id.unwrap()
                .get()
                .to_string()
        ).parse::<u64>()
        .expect("u64 parent_id");

    
    //Disconnect
    if old.is_some() {
        let old_state_channel = old.as_ref().unwrap().channel_id.unwrap();
        let channel_id = old_state_channel.get();

        tracing::debug!("User disconnected from: {}", channel_id);
    
        //Is disconnect was from the voice_start_channel?
        if channel_id != data.voice_id && new_state_channel.get() != channel_id {
            
            //Get some data from db
            let voice_info_wrapped = sqlx::query_as::<_,VoicesInfo>(
                "SELECT * FROM voices_info WHERE channel_id = $1"
            ).bind(channel_id.to_string())
            .fetch_optional(&pool)
            .await?;

            //Is this channel was a child voice channel and is this member was the owner?
            if
                voice_info_wrapped.is_some()
                && member.user.id.to_string() == voice_info_wrapped.as_ref().unwrap().owner_id
            {

                let voice_info = voice_info_wrapped.as_ref().unwrap();
                let members = old_state_channel.to_channel(ctx.http())
                    .await?
                    .guild()
                    .unwrap()
                    .members(&ctx.cache)
                    .unwrap();

                //Is there anyone here?
                if members.len()>= 1 {
                    let new_owner = members.choose(&mut rand::thread_rng()).unwrap();

                    let new_voice_info = voice_info.change_owner(new_owner.user.id.to_string(), &pool).await;

                    tracing::info!("Sets owner_id to: {} for voice channel: {}", new_voice_info.owner_id, new_voice_info.channel_id)
                
                //There is no one here :c
                } else {

                    _ = old_state_channel.delete(ctx.http()).await;
                    _ = voice_info.delete(&pool).await?;

                    tracing::info!("Deleted voice channel: {}", channel_id);

                }
            }
        }

        return Ok(());
        
    //Connect
    } else {
        tracing::debug!("User connected to voice channel: {}", new_state_channel.get());

        //Is connecting was to the voice_start_channel?
        if new_state_channel.get() == data.voice_id {
            
            let voice_info_wrapped = sqlx::query_as::<_,VoicesInfo>(
                    "SELECT * FROM voices_info WHERE owner_id = $1"
                )
                .bind(member.user.id.to_string())
                .fetch_optional(&pool)
                .await?;

            //Is member already have voice channel in own?
            if voice_info_wrapped.is_some() {
                let voice_info = voice_info_wrapped.as_ref().unwrap();

                let channels =  guild.channels(ctx.http()).await?;
                let old_channel_wrapped = channels
                    .get(
                        &ChannelId::new(
                            voice_info.channel_id.parse::<u64>()
                                .expect("Error in parse voice_info.channel_id")
                        )
                    );
                
                //Is it real?:D
                if old_channel_wrapped.is_some() {
                    let old_channel = old_channel_wrapped.unwrap();
                    
                    _ = member.move_to_voice_channel(ctx.http(), old_channel).await;
                    

                    tracing::info!("user {} moved to voice channel {}", member.user.id, old_channel.id);


                    return Ok(());
                } else {
                    voice_info.delete(&pool).await?;

                    tracing::info!("Deleted old voice info with id: {}", voice_info.id);
                }

                drop(channels);
                drop(voice_info_wrapped);
            }
            
            let reason= format(format_args!("User {} created voice chat", member.user.name));

            let channel_builder = 
                CreateChannel::new(format(format_args!("{}'s channel", member.user.name)))
                    .category(parent_id)
                    .kind(ChannelType::Voice)
                    .audit_log_reason(reason.as_str());
            
            let new_channel = guild.id.create_channel(&ctx.http(), channel_builder).await?;
            
            tracing::debug!("Created voice channel {} in discord", new_channel.id);
            
            _ = member.move_to_voice_channel(ctx.http(), new_channel.clone()).await;
            
            tracing::debug!("user {} moved to voice channel {}", member.user.id, new_channel.id);
            
            //Send data to db
            let new_voice_info = VoicesInfo::new(
                new_channel.id.to_string(),
                member.user.id.to_string(),
                &pool
            ).await;

            tracing::info!("Created voice info with id: {}", new_voice_info.id);

        }
        return Ok(());
    }
}