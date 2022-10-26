pub mod types;
mod utilities;

use crate::database;
use crate::colors;
use crate::commands::moderation::types::*;
use crate::commands::moderation::utilities::*;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::model::{error, permissions};

#[command]
#[required_permissions(BAN_MEMBERS)]
pub async fn ban(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let moderation_info = get_timed_moderation_info(msg, args)?;
    let guild_id = moderation_info.guild_id;
    let user_id = moderation_info.user_id;
    let administered_at = moderation_info.administered_at;
    let expiry_date = moderation_info.expiry_date;
    let reason = moderation_info.reason;

    let dm_channel = user_id.create_dm_channel(&ctx.http).await?;

    send_moderation_messages(
        ctx, 
        &dm_channel, 
        &format!(
            "You have been banned from **{}** until <t:{}:F>", 
            guild_id.name(&ctx.cache).expect("Failed to get guild name!").as_str(), 
            expiry_date.unix_timestamp()
        ), 
        colors::RED, 
        &msg.channel_id, 
        &format!(
            "User <@{}> has been banned until <t:{}:F>", 
            user_id.as_u64(), 
            expiry_date.unix_timestamp()
        ), 
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user_id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    database::add_temporary_moderation(&ctx.data, guild_id, user_id, ModerationType::Ban, administered_at, expiry_date, reason.as_deref()).await?;

    if let Some(reason) = &reason {
        guild_id.ban_with_reason(&ctx.http, &user_id, 0, &reason).await?;
    } else {
        guild_id.ban(&ctx.http, &user_id, 0).await?;
    }

    Ok(())
}

#[command]
#[required_permissions(KICK_MEMBERS)]
pub async fn kick(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let moderation_info = get_moderation_info(msg, args)?;
    let guild_id = moderation_info.guild_id;
    let user_id = moderation_info.user_id;
    let administered_at = moderation_info.administered_at;
    let reason = moderation_info.reason;

    let dm_channel = user_id.create_dm_channel(&ctx.http).await?;

    send_moderation_messages(
        ctx, 
        &dm_channel, 
        &format!(
            "You have been kicked from **{}**!", 
            guild_id.name(&ctx.cache).expect("Failed to get guild name!").as_str(),
        ), 
        colors::RED, 
        &msg.channel_id, 
        &format!("User <@{}> has been kicked", user_id.as_u64()), 
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user_id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    database::add_moderation(&ctx.data, guild_id, user_id, ModerationType::Kick, administered_at, reason.as_deref()).await?;

    if let Some(reason) = &reason {
        guild_id.kick_with_reason(&ctx.http, user_id, &reason).await?;
    } else {
        guild_id.kick(&ctx.http, user_id).await?;
    }

    Ok(())
}

#[command]
pub async fn timeout(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let author_permissions = msg.guild(&ctx.cache)
        .expect("Failed to get guild!")
        .member_permissions(&ctx.http, msg.author.id).await?;

    if !author_permissions.moderate_members() {
       return Err(From::from(error::Error::InvalidPermissions(permissions::Permissions::MODERATE_MEMBERS)));
    }

    let moderation_info = get_timed_moderation_info(msg, args)?;
    let guild_id = moderation_info.guild_id;
    let user_id = moderation_info.user_id;
    let expiry_date = moderation_info.expiry_date;
    let reason = moderation_info.reason;

    let dm_channel = user_id.create_dm_channel(&ctx.http).await?;

    // start with the timeout to see if the specified time is over 28d or not
    let successful_timeout = guild_id
    .member(&ctx.http, &user_id).await?
    .disable_communication_until_datetime(&ctx.http, expiry_date).await;

    if let Err(e) = &successful_timeout {
        match e {
            SerenityError::Http(_) => {
                msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| e
                        .color(colors::RED)
                        .description("Timeouts must be shorter than 28 days.")
                    )
                }).await?;
            }
            _ => ()
        }

        // This seems to be the best way to return the error after checking it
        successful_timeout?
    }

    send_moderation_messages(
        ctx, 
        &dm_channel, 
        &format!(
            "You have been timed out in **{}** until <t:{}:F>", 
            guild_id.name(&ctx.cache).expect("Failed to get guild name!").as_str(), 
            expiry_date.unix_timestamp()
        ), 
        colors::RED, 
        &msg.channel_id, 
        &format!(
            "User <@{}> has been timed out until <t:{}:F>", 
            user_id.as_u64(), 
            expiry_date.unix_timestamp()
        ), 
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user_id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    database::add_temporary_moderation(&ctx.data, guild_id, user_id, ModerationType::Timeout, moderation_info.administered_at, expiry_date, reason.as_deref()).await?;

    Ok(())
}