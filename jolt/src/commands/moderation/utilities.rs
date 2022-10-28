use poise::serenity_prelude;

use crate::database;
use crate::colors;

// This function saves a lot of repeated embeds that would be used in multiple contexts with slightly different values.
pub async fn send_moderation_messages(
    ctx: &crate::Context<'_>,
    dm_channel: &serenity_prelude::PrivateChannel,
    dm_message: &str,
    dm_color: u32,
    message_header: &str,
    message: &str,
    color: u32,
    dm_fail_message: &str,
    dm_fail_color: u32,
    reason: Option<&str>
) -> Result<(), crate::DynError> {
    let dm_success = dm_channel.send_message(&ctx.discord().http, |m| {
        m.embed(|e| { e
            .color(dm_color)
            .field(message_header, dm_message, true);

            if let Some(reason) = &reason {
                e.field("Reason:", &reason, false);
            }
    
            e
        })
    }).await;

    ctx.send(|m| {
        m.embed(|e| { e
            .color(color)
            .field(message_header, message, true);

            if let Some(reason) = reason {
                e.field("Reason:", &reason, false);
            }

            e
        })
    }).await?;

    if let Err(_) = dm_success {
        ctx.send(|m| {
            m.embed(|e| e
                .color(dm_fail_color)
                .description(dm_fail_message)
            )
        }).await?;
    }

    let guild_id = ctx.guild_id().expect("Failed to get guild id from context!");
    let logs_channel = database::get_logs_channel(&ctx.data().database, guild_id).await?;

    if let Some(channel) = logs_channel {
        channel.send_message(&ctx.discord().http, |m| m
            .embed(|e| { e 
                .color(colors::BLUE)
                .title("INFO")
                .description(message);

            if let Some(reason) = &reason {
                e.field("Reason:", &reason, false);
            }
        
            e
        })
        ).await?;
    }

    Ok(())
}

/// Appends the expiry date (if exists).
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

const MODERATOR_PERMISSIONS: [serenity_prelude::permissions::Permissions; 14] = [
    serenity_prelude::permissions::Permissions::KICK_MEMBERS,
    serenity_prelude::permissions::Permissions::BAN_MEMBERS,
    serenity_prelude::permissions::Permissions::ADMINISTRATOR,
    serenity_prelude::permissions::Permissions::MANAGE_CHANNELS,
    serenity_prelude::permissions::Permissions::MANAGE_GUILD,
    serenity_prelude::permissions::Permissions::MANAGE_MESSAGES,
    serenity_prelude::permissions::Permissions::MUTE_MEMBERS,
    serenity_prelude::permissions::Permissions::DEAFEN_MEMBERS,
    serenity_prelude::permissions::Permissions::MOVE_MEMBERS,
    serenity_prelude::permissions::Permissions::MANAGE_NICKNAMES,
    serenity_prelude::permissions::Permissions::MANAGE_ROLES,
    serenity_prelude::permissions::Permissions::MANAGE_WEBHOOKS,
    serenity_prelude::permissions::Permissions::MANAGE_THREADS,
    serenity_prelude::permissions::Permissions::MODERATE_MEMBERS,
];

pub fn is_member_moderator(
    cache: &serenity_prelude::Cache,
    member: &serenity_prelude::Member
) -> Result<bool, crate::DynError> {
    let permissions = member.permissions(cache)?;

    Ok(
        permissions.kick_members() ||
        permissions.ban_members() ||
        permissions.administrator() ||
        permissions.manage_channels() ||
        permissions.manage_guild() ||
        permissions.manage_messages() ||
        permissions.manage_channels() ||
        permissions.mute_members() ||
        permissions.deafen_members() ||
        permissions.move_members() ||
        permissions.manage_nicknames() ||
        permissions.manage_roles() ||
        permissions.manage_webhooks() ||
        permissions.manage_threads() ||
        permissions.moderate_members()
    )
}