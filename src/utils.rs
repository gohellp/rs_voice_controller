use poise::serenity_prelude::*;
use rand::seq::IndexedRandom;

pub fn select_random_owner_id(members: &Vec<UserId>) -> &UserId {
    members
        .choose(&mut rand::rng())
        .unwrap_or_else(|| panic!("Looks like members lost all it's items"))
}

pub async fn change_owner(
    ctx: &Context,
    channel_id: &ChannelId,
    user_id: UserId,
) -> anyhow::Result<()> {
    let perms = vec![PermissionOverwrite {
        allow: Permissions::MANAGE_CHANNELS
            | Permissions::MUTE_MEMBERS
            | Permissions::DEAFEN_MEMBERS,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Member(user_id),
    }];
    let name = format!("{}'s channel", user_id.to_user(ctx.http()).await?.name);
    let builder = EditChannel::new().name(name).permissions(perms);

    channel_id.edit(ctx.http(), builder).await?;

    Ok(())
}
