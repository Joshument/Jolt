use crate::commands::moderation::types::*;

use serenity::framework::standard::{Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;

use humantime;

pub fn get_timed_moderation_info(msg: &Message, mut args: Args) -> Result<TimedModerationInfo, CommandError> {
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

    Ok(TimedModerationInfo { guild_id, user_id, administered_at: now, expiry_date, reason: reason })
}

pub fn get_moderation_info(msg: &Message, mut args: Args) -> Result<ModerationInfo, CommandError> {
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

    Ok(ModerationInfo { guild_id, user_id, administered_at: Timestamp::now(), reason: reason })
}

// This function saves a lot of repeated embeds that would be used in multiple contexts with slightly different values.
pub async fn send_moderation_messages(
    ctx: &Context,
    dm_channel: &PrivateChannel,
    dm_message: &str,
    dm_color: u32,
    channel: &ChannelId,
    message: &str,
    color: u32,
    dm_fail_message: &str,
    dm_fail_color: u32,
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
                .color(dm_fail_color)
                .description(dm_fail_message)
            )
        }).await?;
    }

    Ok(())
}