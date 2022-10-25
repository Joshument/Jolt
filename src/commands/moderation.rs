use crate::database;
use crate::colors;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;

use humantime;

#[derive(Debug, Clone)]
pub struct IntEnumError;

impl std::error::Error for IntEnumError {}

impl std::fmt::Display for IntEnumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to convert integer into enum type")
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)] // some values are going to be used later, no need to have useless warnings
pub enum ModerationType {
    Warning = 0,
    Kick = 1,
    Mute = 2,
    Timeout = 3,
    Ban = 4,
}

impl TryFrom<u8> for ModerationType {
    type Error = IntEnumError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ModerationType::Warning),
            1 => Ok(ModerationType::Kick),
            2 => Ok(ModerationType::Mute),
            3 => Ok(ModerationType::Timeout),
            4 => Ok(ModerationType::Ban),
            _ => Err(IntEnumError)
        }
    }
}

struct TimedModerationInfo {
    guild_id: GuildId,
    user_id: UserId,
    expiry_date: Timestamp,
    reason: Option<String>,
}

struct ModerationInfo {
    guild_id: GuildId,
    user_id: UserId,
    reason: Option<String>,
}

async fn get_timed_moderation_info(msg: &Message, mut args: Args) -> Result<TimedModerationInfo, CommandError> {
    let user_id = args.single::<UserId>()?;
    let time_string = args.single::<String>()?;
    let reason = {
        let temp = args.rest();

        if temp.len() > 0 {
            Some(temp.to_string())
        } else {
            None
        }
    };

    let time = humantime::parse_duration(&time_string)?;
    let now = Timestamp::now();
    let expiry_date = Timestamp::from_unix_timestamp(now.unix_timestamp() + time.as_secs() as i64)?;
    let guild_id = msg.guild_id.expect("Failed to get guild id!");

    Ok(TimedModerationInfo { guild_id, user_id, expiry_date, reason: reason })
}

async fn get_moderation_info(msg: &Message, mut args: Args) -> Result<ModerationInfo, CommandError> {
    let user_id = args.single::<UserId>()?;
    let reason = {
        let temp = args.rest();

        if temp.len() > 0 {
            Some(temp.to_string())
        } else {
            None
        }
    };

    let guild_id = msg.guild_id.expect("Failed to get guild id!");

    Ok(ModerationInfo { guild_id, user_id, reason: reason })
}

// This function saves a lot of repeated embeds that would be used in multiple contexts with slightly different values.
async fn send_moderation_messages(
    ctx: &Context,
    dm_channel: &PrivateChannel,
    dm_message: &str,
    dm_color: u32,
    channel: &ChannelId,
    message: &str,
    dm_fail_message: &str,
    color: u32,
    reason: Option<&str>
) -> CommandResult {
    let dm_success = dm_channel.send_message(&ctx.http, |m| {
        m.embed(|e| { e
            .color(dm_color)
            .field("Zap!", dm_message, true);
    
            if let Some(reason) = &reason {
                e.field("Reason:", &reason, false);
            }
    
            e
        })
    }).await;

    channel.send_message(&ctx.http, |m| {
        m.embed(|e| { e
            .color(color)
            .field("Zap!", message, true);

            if let Some(reason) = reason {
                e.field("Reason:", &reason, false);
            }

            e
        })
    }).await?;

    if let Err(_) = dm_success {
        channel.send_message(&ctx.http, |m| {
            m.embed(|e| e
                .color(0xf38ba8)
                .description(dm_fail_message)
            )
        }).await?;
    }

    Ok(())
}

#[command]
#[required_permissions(BAN_MEMBERS)]
pub async fn ban(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let moderation_info = get_timed_moderation_info(msg, args).await?;
    let guild_id = moderation_info.guild_id;
    let user_id = moderation_info.user_id;
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
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user_id.as_u64()
        ), 
        colors::GREEN, 
        reason.as_deref()
    ).await?;

    database::add_temporary_moderation(&ctx.data, guild_id, user_id, ModerationType::Ban, expiry_date, reason.as_deref()).await?;

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
    let moderation_info = get_moderation_info(msg, args).await?;
    let guild_id = moderation_info.guild_id;
    let user_id = moderation_info.user_id;
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
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user_id.as_u64()
        ), 
        colors::GREEN, 
        reason.as_deref()
    ).await?;

    database::add_moderation(&ctx.data, guild_id, user_id, ModerationType::Kick, reason.as_deref()).await?;

    if let Some(reason) = &reason {
        guild_id.kick_with_reason(&ctx.http, user_id, &reason).await?;
    } else {
        guild_id.kick(&ctx.http, user_id).await?;
    }

    Ok(())
}