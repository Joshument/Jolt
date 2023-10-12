use poise::serenity_prelude;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateMessage;
use poise::CreateReply;

use crate::colors;
use crate::database;

use super::types::ModerationType;
use super::types::ModlogEntry;

/// Send a moderation message using the same reusable fields.
/// This function exists to reduce boilerplate, as it's much easier to just give a function parameters than to
/// repeatedly regenerate the embeds every time.
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
    reason: Option<&str>,
) -> Result<(), crate::error::Error> {
    let dm_success = dm_channel
        .send_message(
            &ctx.http(),
            CreateMessage::default().embed({
                let e =
                    CreateEmbed::default()
                        .color(dm_color)
                        .field(message_header, dm_message, true);

                if let Some(reason) = reason {
                    e.field("Reason:", reason, false)
                } else {
                    e
                }
            }),
        )
        .await;

    ctx.send(CreateReply::default().embed({
        let e = CreateEmbed::default()
            .color(color)
            .field(message_header, message, true);

        if let Some(reason) = reason {
            e.field("Reason:", reason, false)
        } else {
            e
        }
    }))
    .await?;

    if let Err(_) = dm_success {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::default()
                    .color(dm_fail_color)
                    .description(dm_fail_message),
            ),
        )
        .await?;
    }

    let guild_id = ctx
        .guild_id()
        .expect("Failed to get guild id from context!");
    let logs_channel = database::get_logs_channel(&ctx.data().database, guild_id).await?;

    if let Some(channel) = logs_channel {
        channel
            .send_message(
                &ctx.http(),
                CreateMessage::default().embed({
                    let e = CreateEmbed::default()
                        .color(colors::BLUE)
                        .title("INFO")
                        .description(message);

                    if let Some(reason) = reason {
                        e.field("Reason:", reason, false)
                    } else {
                        e
                    }
                }),
            )
            .await?;
    }

    Ok(())
}

/// Appends the expiry date (if exists).
/// Function exists to reduce boilerplate
pub fn append_expiry_date(
    message: &str,
    expiry_date: Option<serenity_prelude::Timestamp>,
) -> String {
    match expiry_date {
        Some(unix_time) => format!("{} until <t:{}:F>", message, unix_time.unix_timestamp()),
        None => message.to_string(),
    }
}

/// Checks if the member has any moderation related permissions.
/// This is mostly used to determine if a moderation action can be done on the user.
pub fn is_member_moderator(
    cache: &serenity_prelude::Cache,
    member: &serenity_prelude::Member,
) -> Result<bool, crate::error::Error> {
    let permissions = member.permissions(cache)?;

    // There has to be a better way to do this I swear to god
    Ok(permissions.kick_members()
        || permissions.ban_members()
        || permissions.administrator()
        || permissions.manage_channels()
        || permissions.manage_guild()
        || permissions.manage_messages()
        || permissions.manage_channels()
        || permissions.mute_members()
        || permissions.deafen_members()
        || permissions.move_members()
        || permissions.manage_nicknames()
        || permissions.manage_roles()
        || permissions.manage_webhooks()
        || permissions.manage_threads()
        || permissions.moderate_members())
}

// Formats a CreateEmbed into a modlog format
pub fn modlog_embed(mut embed: CreateEmbed, modlogs: Vec<ModlogEntry>) -> CreateEmbed {
    for modlog in modlogs {
        println!("{:?}\n{}", modlog.expiry_date, modlog.moderation_type);
        embed = embed.field(
            format!("ID {}", modlog.id),
            // The way I omit a certain part of the moderation is to replace the segment with an empty string.
            // This is because of the way that the field works, and since this involves display vs variable
            // checking, this is going somewhat against how you would expect this to be handled (no `Option<T>`)
            format!(
                "{}{}{}{}{}{}",
                format!("\n**Moderator:** <@{}>", modlog.moderator_id),
                format!("\n**Type:** {}", modlog.moderation_type,),
                format!(
                    "\n**Administered At:** <t:{}:F>",
                    modlog.administered_at.unix_timestamp()
                ),
                match modlog.reason {
                    Some(reason) => format!("\n**Reason:** {}", reason),
                    None => String::new(),
                },
                match modlog.expiry_date {
                    Some(expiration) =>
                        format!("\n**Expires:** <t:{}:F>", expiration.unix_timestamp()),
                    None => String::new(),
                },
                match modlog.moderation_type {
                    ModerationType::Kick
                    | ModerationType::Unban
                    | ModerationType::Unmute
                    | ModerationType::Untimeout => String::new(),
                    _ => format!("\n**Active:** {}", modlog.active),
                }
            ),
            false,
        );
    }

    embed
}
