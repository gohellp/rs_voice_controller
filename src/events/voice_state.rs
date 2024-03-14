use std::{
    env::var,
    fmt::format
};
use crate::structs::{
    Result,
    FrameworkContext
};
use poise::serenity_prelude::{
    Context,
    CacheHttp,
    builder::CreateChannel,
    all::{
        GuildId,
        ChannelId,
        ChannelType,
        VoiceState
    }
};
use rs_voice_controller::{
    establish_connection,
    models::NewVoicesInfo,
    schema::voices_info::dsl::*
};
use diesel::{
    dsl::exists,
    prelude::*,
    select,
    delete,
    insert_into
};



pub async fn voice_state_update(
    framework_ctx: FrameworkContext<'_>,
    ctx: &Context,
    old: Option<&VoiceState>,
    new: &VoiceState,
) -> Result<()> {

    let data = framework_ctx.user_data;

    let member = &new.member.clone().unwrap();
    let connection = &mut establish_connection();
    let new_state_channel = &new.channel_id.unwrap_or_default();
    let guild = &ctx.http.get_guild(GuildId::new(data.guild_id)).await?;
    
    //optional var
    let _parent_id = var("PARENT_CHANNEL_ID").unwrap_or(
            guild.channels(ctx).await?.get(&ChannelId::new(data.voice_id)).expect("voice channel").parent_id.unwrap().get().to_string()
        ).parse::<u64>().expect("u64 parent_id");

    
    //Disconnect
    if old.is_some() {
        let old_state_channel = old.as_ref().unwrap().channel_id.unwrap();
        let _channel_id = old_state_channel.get();
    
        if _channel_id != data.voice_id && new_state_channel.get() != _channel_id {
            
            
            //Get some data from db
            let voice_info_existence = select(exists(voices_info.filter(channel_id.eq(_channel_id.to_string()))))
                .get_result(connection)
                .unwrap();

            if voice_info_existence {

                _ = old_state_channel.delete(ctx.http()).await;

                delete(voices_info.filter(channel_id.eq(_channel_id.to_string())))
                    .execute(connection)
                    .expect("Error deleting voices_info");
            }
        }

        return Ok(());
        
    //Connect
    } else {
        if new_state_channel.get() == data.voice_id {
            let reason= format(format_args!("Caused by {}", member.user.name));
            
            let _channel_builder = 
                CreateChannel::new(format(format_args!("{}'s channel", member.user.name)))
                    .category(_parent_id)
                    .kind(ChannelType::Voice)
                    .audit_log_reason(reason.as_str());
            
            let new_channel = guild.id.create_channel(&ctx.http(), _channel_builder).await?;
            
            _ = member.move_to_voice_channel(ctx.http(), new_channel.clone()).await;
            
            //Send data to db
            let new_voice_info = NewVoicesInfo {
                channel_id: &new_channel.id.to_string(),
                owner_id: &member.user.id.to_string(),
            };

            _ = insert_into(rs_voice_controller::schema::voices_info::table)
                .values(&new_voice_info)
                .execute(connection);

        }
        return Ok(());
    }
}