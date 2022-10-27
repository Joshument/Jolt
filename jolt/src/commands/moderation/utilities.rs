use crate::commands::moderation::types::*;

use poise::serenity_prelude;
use serenity_prelude::{Timestamp};

use humantime;

/*
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
*/

// This function saves a lot of repeated embeds that would be used in multiple contexts with slightly different values.
pub async fn send_moderation_messages(
    ctx: &serenity_prelude::Context,
    dm_channel: &serenity_prelude::PrivateChannel,
    dm_message: &str,
    dm_color: u32,
    channel: &serenity_prelude::ChannelId,
    message: &str,
    color: u32,
    dm_fail_message: &str,
    dm_fail_color: u32,
    reason: Option<&str>
) -> Result<(), crate::DynError> {
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

/// Appends the expiry date (if exists)
/// Function exists to reduce boilerplate
pub fn append_expiry_date(message: &str, expiry_date: Option<serenity_prelude::Timestamp>) -> String {
    match expiry_date {
        Some(unix_time) => format!(
            "{} until <t:{}:F>",
            message, 
            unix_time.unix_timestamp()
        ),
        None => message.to_string()
    }
}