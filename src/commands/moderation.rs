use crate::database;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use humantime;

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

#[derive(Debug, Clone)]
pub struct IntEnumError;

impl std::error::Error for IntEnumError {}

impl std::fmt::Display for IntEnumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to convert integer into enum type")
    }
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

#[command]
#[required_permissions(BAN_MEMBERS)]
pub async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = args.single::<UserId>()?;
    let time_string = args.single::<String>()?;
    let reason = args.rest();
    
    let time = humantime::parse_duration(&time_string)?;
    let now = Timestamp::now();
    let unban_time = Timestamp::from_unix_timestamp(now.unix_timestamp() + time.as_secs() as i64)?;
    let guild_id = msg.guild_id.expect("Failed to get guild id!");

    let dm_channel = user_id.create_dm_channel(&ctx.http).await?;

    let dm_success = dm_channel.send_message(&ctx.http, |m| {
        m.embed(|e| { e
            .color(0xf38ba8)
            .field("Zap!", format!(
                "You have been banned from **{}** until <t:{}:F>", 
                guild_id.name(&ctx.cache).expect("Failed to get guild name!").as_str(), 
                unban_time.unix_timestamp()
            ), true);
    
            if reason.len() > 0 {
                e.field("Reason:", reason, false);
            }
    
            e
        })
    }).await;

    database::add_temporary_moderation(&ctx.data, guild_id, user_id, ModerationType::Ban, unban_time, reason).await?;

    if reason.len() > 0 {
        guild_id.ban_with_reason(&ctx.http, &user_id, 0, reason).await?;
    } else {
        guild_id.ban(&ctx.http, &user_id, 0).await?;
    }

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| { e
            .color(0xa6e3a1)
            .field("Zap!", format!("User <@{}> has been banned until <t:{}:F>", user_id.as_u64(), unban_time.unix_timestamp()), true);

            if reason.len() > 0 {
                e.field("Reason:", reason, false);
            }

            e
        })
    }).await?;

    if let Err(_) = dm_success {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| e
                .color(0xf38ba8)
                .description(format!("I was unable to DM <@{}> about their moderation.", user_id.as_u64()))
            )
        }).await?;
    }

    Ok(())
}

#[command]
#[required_permissions(KICK_MEMBERS)]
pub async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = args.single::<UserId>()?;
    let reason = args.rest();

    let guild_id = msg.guild_id.expect("Failed to get guild id!");

    let dm_channel = user_id.create_dm_channel(&ctx.http).await?;

    let dm_success = dm_channel.send_message(&ctx.http, |m| {
        m.embed(|e| { e
            .color(0xf38ba8)
            .field("Zap!", format!(
                "You have been kicked from **{}**!", 
                guild_id.name(&ctx.cache).expect("Failed to get guild name!").as_str(),
            ), true);
    
            if reason.len() > 0 {
                e.field("Reason:", reason, false);
            }
    
            e
        })
    }).await;

    guild_id.kick_with_reason(&ctx.http, user_id, reason).await?;

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| { e
            .color(0xa6e3a1)
            .field("Zap!", format!("User <@{}> has been kicked", user_id.as_u64()), true);

            if reason.len() > 0 {
                e.field("Reason:", reason, false);
            }

            e
        })
    }).await?;

    if let Err(_) = dm_success {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| e
                .color(0xf38ba8)
                .description(format!("I was unable to DM <@{}> about their moderation.", user_id.as_u64()))
            )
        }).await?;
    }

    Ok(())
}