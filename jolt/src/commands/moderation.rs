/*
Moderation commands

Probably the most important part of the bot, as these are essential for managing any server at all.
Each command follows the same sort of format:
    - Get the guild id,
    - Get the moderator id,
    - Try to perform the moderation,
    - Add moderation to modlogs and announce the moderation so the moderator knows it succeeded.
Each command also has a required permission, and makes sure that any moderation done is not to a user
with any form of moderative permissions. In cases of ban / unban, the permission needed is obvious,
but when in doubt, go with KICK_MEMBERS.
*/

pub mod error;
pub mod types;
mod utilities;

use crate::colors;
use crate::commands::moderation::types::*;
use crate::commands::moderation::utilities::*;
use crate::database;
use crate::error::{Error, ModerationError};

use poise::serenity_prelude;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateEmbedFooter;
use poise::CreateReply;
use serenity_prelude::Error as SerenityError;

/// Warn a user
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "KICK_MEMBERS",
    help_text_fn = "warn_help",
    category = "moderation"
)]
pub async fn warn(
    ctx: crate::Context<'_>,
    #[description = "User to warn"] user: serenity_prelude::User,
    #[description = "Reason for warning"]
    #[rest]
    reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let moderator = ctx.author();
    let administered_at = ctx.created_at();

    let member = guild_id.member(&ctx, user.id).await?;
    if is_member_moderator(&ctx.cache(), &member)? {
        return Err(ModerationError::MemberIsModerator(member).into());
    }

    let dm_channel = user.create_dm_channel(&ctx.http()).await?;

    send_moderation_messages(
        &ctx,
        &dm_channel,
        &format!(
            "You have been warned in **{}**",
            &guild_id
                .name(&ctx.cache())
                .expect("Failed to get guild name!")
        ),
        colors::RED,
        "Zap!",
        &format!("User <@{}> has been warned", user.id),
        colors::GREEN,
        &format!("I was unable to DM <@{}> about their moderation.", user.id),
        colors::RED,
        reason.as_deref(),
    )
    .await?;

    database::add_moderation(
        &ctx.data().database,
        guild_id,
        user.id,
        moderator.id,
        ModerationType::Warning,
        administered_at,
        None,
        reason.as_deref(),
    )
    .await?;

    Ok(())
}

fn warn_help() -> String {
    String::from(
        "Warn a user in the server (with an optional reason).
Example: %warn @Joshument#0001 I am feeling evil today
        ",
    )
}

/// Delete a warn from a user. Specified by ID.
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "KICK_MEMBERS",
    help_text_fn = "delwarn_help",
    category = "moderation"
)]
pub async fn delwarn(
    ctx: crate::Context<'_>,
    #[description = "Modlog ID to remove"] id: u64,
) -> Result<(), crate::DynError> {
    let modlog = database::get_single_modlog(&ctx.data().database, id).await?;

    if modlog.guild_id != ctx.guild_id().expect("Failed to get guild id!") {
        return Err(ModerationError::ModlogNotInGuild(
            id,
            ctx.guild().expect("Failed to get guild!").to_owned(),
        )
        .into());
    }

    if modlog.moderation_type != ModerationType::Warning {
        return Err(ModerationError::NotAWarning(id).into());
    }
    database::clear_single_moderation(&ctx.data().database, id).await?;

    ctx.send(CreateReply::default().embed(
        CreateEmbed::default().color(crate::colors::GREEN).field(
            "Done!",
            &format!("Deleted warning **{id}**"),
            true,
        ),
    ))
    .await?;

    Ok(())
}

fn delwarn_help() -> String {
    String::from(
        "Delete a warn from a user.
Example: %delwarn 3872
        ",
    )
}

/// Get the mod logs for a specified user
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "KICK_MEMBERS",
    help_text_fn = "warnings_help",
    category = "moderation"
)]
pub async fn warnings(
    ctx: crate::Context<'_>,
    #[description = "User to get modlogs from"] user: serenity_prelude::User,
    #[description = "Modlogs page"] page: Option<usize>,
) -> Result<(), crate::DynError> {
    let page = match page {
        Some(page) => page,
        None => 1,
    };

    let max_page = database::get_warning_count(
        &ctx.data().database,
        ctx.guild_id().expect("Failed to get guild id!"),
        user.id,
    )
    .await?
        / 10
        + 1;
    let modlog_page = database::get_warning_page(
        &ctx.data().database,
        ctx.guild_id().expect("Failed to get guild id!"),
        user.id,
        page,
        10,
    )
    .await?;

    ctx.send(CreateReply::default().embed({
        let mut e = CreateEmbed::default().title(format!("Warnings for {}", user.name));

        for modlog in modlog_page {
            e = e.field(
                format!("ID {}", modlog.id),
                format!(
                    "{}{}{}",
                    format!("\n**Moderator:** <@{}>", modlog.moderator_id),
                    format!(
                        "\n**Administered At:** <t:{}:F>",
                        modlog.administered_at.unix_timestamp()
                    ),
                    match modlog.reason {
                        Some(reason) => format!("\n**Reason:** {}", reason),
                        None => String::new(),
                    },
                ),
                false,
            );
        }

        e.footer(CreateEmbedFooter::new(format!(
            "Page {} of {}",
            page, max_page
        )))
        .color(colors::BLUE)
    }))
    .await?;

    Ok(())
}

fn warnings_help() -> String {
    String::from(
        "Get the warnings for the specified user.
Example: %warnings @Joshument#0001 1
        ",
    )
}

/// Ban a user (with an optional specified time)
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS",
    help_text_fn = "ban_help",
    category = "moderation"
)]
pub async fn ban(
    ctx: crate::Context<'_>,
    #[description = "User to ban"] user: serenity_prelude::User,
    #[description = "Length of the ban"] length: Option<humantime::Duration>,
    #[description = "Reason for ban"]
    #[rest]
    reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let moderator = ctx.author();
    let administered_at = ctx.created_at();
    // Replaces Option<Duration> into Option<Timestamp>
    // .transpose()? brings out the inner result propagate upstream with `?`
    // Using `?` inside of .map() would instead return it to the closure, therefore making it invalid.
    let expiry_date = length
        .map(|duration| {
            serenity_prelude::Timestamp::from_unix_timestamp(
                administered_at.unix_timestamp() + duration.as_secs() as i64,
            )
        })
        .transpose()?;

    let member = guild_id.member(&ctx, user.id).await?;
    if is_member_moderator(&ctx.cache(), &member)? {
        return Err(ModerationError::MemberIsModerator(member).into());
    }

    let dm_channel = user.create_dm_channel(&ctx.http()).await?;

    send_moderation_messages(
        &ctx,
        &dm_channel,
        &append_expiry_date(
            &format!(
                "You have been banned from **{}**",
                &guild_id
                    .name(&ctx.cache())
                    .expect("Failed to get guild name!")
            ),
            expiry_date,
        ),
        colors::RED,
        "Zap!",
        &append_expiry_date(&format!("User <@{}> has been banned", user.id), expiry_date),
        colors::GREEN,
        &format!("I was unable to DM <@{}> about their moderation.", user.id),
        colors::RED,
        reason.as_deref(),
    )
    .await?;

    database::add_moderation(
        &ctx.data().database,
        guild_id,
        user.id,
        moderator.id,
        ModerationType::Ban,
        administered_at,
        expiry_date,
        reason.as_deref(),
    )
    .await?;

    if let Some(reason) = &reason {
        guild_id
            .ban_with_reason(&ctx.http(), &user.id, 0, &reason)
            .await?;
    } else {
        guild_id.ban(&ctx.http(), &user.id, 0).await?;
    }

    Ok(())
}

fn ban_help() -> String {
    String::from(
        "Ban a user from the server (with an optional specified time and reason).
Example: %ban @Joshument#0001 10s joined 10 seconds too early
        ",
    )
}

/// Unban a user
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS",
    help_text_fn = "unban_help",
    category = "moderation"
)]
pub async fn unban(
    ctx: crate::Context<'_>,
    #[description = "User to unban"] user: serenity_prelude::User,
    #[description = "Reason for the unban"]
    #[rest]
    reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let administered_at = ctx.created_at();
    let moderator = ctx.author();

    let dm_channel = user.create_dm_channel(&ctx.http()).await?;

    send_moderation_messages(
        &ctx,
        &dm_channel,
        &format!(
            "You have been unbanned from **{}**",
            &guild_id
                .name(&ctx.cache())
                .expect("Failed to get guild name!")
        ),
        colors::GREEN,
        "!paZ",
        &format!("User <@{}> has been unbanned", user.id),
        colors::GREEN,
        &format!("I was unable to DM <@{}> about their moderation.", user.id),
        colors::RED,
        reason.as_deref(),
    )
    .await?;

    guild_id.unban(&ctx.http(), &user.id).await?;
    database::clear_moderations(&ctx.data().database, guild_id, user.id, ModerationType::Ban)
        .await?;

    database::add_moderation(
        &ctx.data().database,
        guild_id,
        user.id,
        moderator.id,
        ModerationType::Unban,
        administered_at,
        None,
        reason.as_deref(),
    )
    .await?;

    Ok(())
}

fn unban_help() -> String {
    String::from(
        "Unban a user from the server.
Example: %unban @Joshument#0001 my perspective of you has changed
        ",
    )
}

/// Kick a user
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "KICK_MEMBERS",
    required_bot_permissions = "KICK_MEMBERS",
    help_text_fn = "kick_help",
    category = "moderation"
)]
pub async fn kick(
    ctx: crate::Context<'_>,
    #[description = "User to kick"] user: serenity_prelude::User,
    #[description = "Reason for kick"]
    #[rest]
    reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let moderator = ctx.author();
    let administered_at = ctx.created_at();

    let member = guild_id.member(&ctx, user.id).await?;
    if is_member_moderator(&ctx.cache(), &member)? {
        return Err(ModerationError::MemberIsModerator(member).into());
    }

    let dm_channel = user.create_dm_channel(&ctx).await?;

    send_moderation_messages(
        &ctx,
        &dm_channel,
        &format!(
            "You have been kicked from **{}**!",
            guild_id
                .name(&ctx.cache())
                .expect("Failed to get guild name!")
                .as_str(),
        ),
        colors::RED,
        "Zap!",
        &format!("User <@{}> has been kicked", user.id),
        colors::GREEN,
        &format!("I was unable to DM <@{}> about their moderation.", user.id),
        colors::RED,
        reason.as_deref(),
    )
    .await?;

    database::add_moderation(
        &ctx.data().database,
        guild_id,
        user.id,
        moderator.id,
        ModerationType::Kick,
        administered_at,
        None,
        reason.as_deref(),
    )
    .await?;

    if let Some(reason) = &reason {
        guild_id
            .kick_with_reason(&ctx.http(), user.id, &reason)
            .await?;
    } else {
        guild_id.kick(&ctx.http(), user.id).await?;
    }

    Ok(())
}

fn kick_help() -> String {
    String::from(
        "Kick a user from the server.
Example: %kick @Joshument#0001 nerd
    ",
    )
}

/// Timeout a user for a specified amount of time
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS",
    help_text_fn = "timeout_help",
    category = "moderation"
)]
pub async fn timeout(
    ctx: crate::Context<'_>,
    #[description = "User to timeout"] user: serenity_prelude::User,
    #[description = "Length of the timeout"] length: humantime::Duration,
    #[description = "Reason for timeout"]
    #[rest]
    reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let moderator = ctx.author();
    let administered_at = ctx.created_at();
    let expiry_date = serenity_prelude::Timestamp::from_unix_timestamp(
        administered_at.unix_timestamp() + length.as_secs() as i64,
    )?;

    let member = guild_id.member(&ctx, user.id).await?;
    if is_member_moderator(&ctx.cache(), &member)? {
        return Err(ModerationError::MemberIsModerator(member).into());
    }

    let dm_channel = user.create_dm_channel(&ctx).await?;

    // start with the timeout to see if the specified time is over 28d or not
    let successful_timeout = guild_id
        .member(&ctx.http(), &user.id)
        .await?
        .disable_communication_until_datetime(&ctx.http(), expiry_date)
        .await;

    if let Err(e) = &successful_timeout {
        match e {
            SerenityError::Http(_) => {
                ctx.send(
                    CreateReply::default()
                        .embed(
                            CreateEmbed::default()
                                .color(colors::RED)
                                .description("Timeouts must be shorter than 28 days."),
                        )
                        .ephemeral(true),
                )
                .await?;
            }
            _ => (),
        }

        // This seems to be the best way to return the error after checking it
        successful_timeout?
    }

    send_moderation_messages(
        &ctx,
        &dm_channel,
        &format!(
            "You have been timed out from **{}** until <t:{}:F>",
            guild_id
                .name(&ctx.cache())
                .expect("Failed to get guild name!"),
            expiry_date.unix_timestamp()
        ),
        colors::RED,
        "Zap!",
        &format!(
            "User <@{}> has been timed out until <t:{}:F>",
            user.id,
            expiry_date.unix_timestamp()
        ),
        colors::GREEN,
        &format!("I was unable to DM <@{}> about their moderation.", user.id),
        colors::RED,
        reason.as_deref(),
    )
    .await?;

    database::add_moderation(
        &ctx.data().database,
        guild_id,
        user.id,
        moderator.id,
        ModerationType::Timeout,
        administered_at,
        Some(expiry_date),
        reason.as_deref(),
    )
    .await?;

    Ok(())
}

fn timeout_help() -> String {
    String::from(
        "Time out a user from the server.
Example: %timeout @Paze#2936 10m not a fan of the inconsistencies
            ",
    )
}

/// Revoke the timeout for a user
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS",
    help_text_fn = "untimeout_help",
    category = "moderation",
    aliases("revoketimeout")
)]
pub async fn untimeout(
    ctx: crate::Context<'_>,
    #[description = "User to untimeout"] user: serenity_prelude::User,
    #[description = "Reason for the untimeout"]
    #[rest]
    reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let moderator = ctx.author();
    let administered_at = ctx.created_at();

    let dm_channel = user.create_dm_channel(&ctx).await?;
    database::clear_moderations(
        &ctx.data().database,
        guild_id,
        user.id,
        ModerationType::Timeout,
    )
    .await?;

    guild_id
        .member(&ctx.http(), &user.id)
        .await?
        .enable_communication(&ctx.http())
        .await?;

    send_moderation_messages(
        &ctx,
        &dm_channel,
        &format!(
            "You have been untimed out from **{}**",
            &guild_id
                .name(&ctx.cache())
                .expect("Failed to get guild name!")
        ),
        colors::GREEN,
        "!paZ",
        &format!("User <@{}> has been untimed out", user.id),
        colors::GREEN,
        &format!("I was unable to DM <@{}> about their moderation.", user.id),
        colors::RED,
        reason.as_deref(),
    )
    .await?;

    database::add_moderation(
        &ctx.data().database,
        guild_id,
        user.id,
        moderator.id,
        ModerationType::Untimeout,
        administered_at,
        None,
        reason.as_deref(),
    )
    .await?;

    Ok(())
}

fn untimeout_help() -> String {
    String::from(
        "Revoke the timeoutout a user.
Example: %untimeout @Paze#2936 fan of the consistencies :)
            ",
    )
}

/// Mute a user (with an optional specified time)
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MANAGE_ROLES",
    help_text_fn = "mute_help",
    category = "moderation"
)]
pub async fn mute(
    ctx: crate::Context<'_>,
    #[description = "User to mute"] user: serenity_prelude::User,
    #[description = "Length of the mute"] length: Option<humantime::Duration>,
    #[description = "Reason for mute"]
    #[rest]
    reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let moderator = ctx.author();

    let member = guild_id.member(&ctx, user.id).await?;
    if is_member_moderator(&ctx.cache(), &member)? {
        return Err(ModerationError::MemberIsModerator(member).into());
    }

    let mute_role = database::get_mute_role(&ctx.data().database, guild_id).await?;
    if let None = mute_role {
        return Err(Error::ConfigNotSetError(String::from("%muterole")).into());
    }

    let administered_at = ctx.created_at();
    // Replaces Option<Duration> into Option<Timestamp>
    // .transpose()? brings out the inner result to propagate upstream with `?`
    // Using `?` inside of .map() would instead return it to the closure, therefore making it invalid.
    let expiry_date = length
        .map(|duration| {
            serenity_prelude::Timestamp::from_unix_timestamp(
                administered_at.unix_timestamp() + duration.as_secs() as i64,
            )
        })
        .transpose()?;

    let dm_channel = user.create_dm_channel(&ctx).await?;

    send_moderation_messages(
        &ctx,
        &dm_channel,
        &append_expiry_date(
            &format!(
                "You have been muted in **{}**",
                &guild_id
                    .name(&ctx.cache())
                    .expect("Failed to get guild name!")
            ),
            expiry_date,
        ),
        colors::RED,
        "Zap!",
        &append_expiry_date(&format!("User <@{}> has been muted", user.id), expiry_date),
        colors::GREEN,
        &format!("I was unable to DM <@{}> about their moderation.", user.id),
        colors::RED,
        reason.as_deref(),
    )
    .await?;

    database::add_moderation(
        &ctx.data().database,
        guild_id,
        user.id,
        moderator.id,
        ModerationType::Mute,
        administered_at,
        expiry_date,
        reason.as_deref(),
    )
    .await?;

    let mut member = guild_id.member(&ctx, user.id).await?;
    // unwrap is safe to use here as there is already a check for `None` prior to this expression
    member.add_role(&ctx.http(), mute_role.unwrap()).await?;

    Ok(())
}

fn mute_help() -> String {
    String::from(
        "Mute a user (with an optional specified time).
Example: %mute @Joshument#0001 3d keeps procrastinating on the modlogs command
        ",
    )
}

/// Unmute a user
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MANAGE_ROLES",
    help_text_fn = "unmute_help",
    category = "moderation"
)]
pub async fn unmute(
    ctx: crate::Context<'_>,
    #[description = "User to mute"] user: serenity_prelude::User,
    #[description = "Length of the mute"] length: Option<humantime::Duration>,
    #[description = "Reason for mute"]
    #[rest]
    reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let moderator = ctx.author();

    let mute_role = database::get_mute_role(&ctx.data().database, guild_id).await?;
    if let None = mute_role {
        ctx.send(
            CreateReply::default()
            .embed(
                CreateEmbed::new()
                .color(colors::RED)
                .description(
                    "Mute role has not been set! Please set the mute role using the `muterole` command."
                )
            )
        ).await?;
    }

    let administered_at = ctx.created_at();
    // Replaces Option<Duration> into Option<Timestamp>
    // .transpose()? brings out the inner result propagate upstream with `?`
    // Using `?` inside of .map() would instead return it to the closure, therefore making it invalid.
    let expiry_date = length
        .map(|duration| {
            serenity_prelude::Timestamp::from_unix_timestamp(
                administered_at.unix_timestamp() + duration.as_secs() as i64,
            )
        })
        .transpose()?;

    let dm_channel = user.create_dm_channel(&ctx).await?;

    send_moderation_messages(
        &ctx,
        &dm_channel,
        &append_expiry_date(
            &format!(
                "You have been unmuted in **{}**",
                &guild_id
                    .name(&ctx.cache())
                    .expect("Failed to get guild name!")
            ),
            expiry_date,
        ),
        colors::RED,
        "!paZ",
        &append_expiry_date(
            &format!("User <@{}> has been unmuted", user.id),
            expiry_date,
        ),
        colors::GREEN,
        &format!("I was unable to DM <@{}> about their moderation.", user.id),
        colors::RED,
        reason.as_deref(),
    )
    .await?;

    database::clear_moderations(
        &ctx.data().database,
        guild_id,
        user.id,
        ModerationType::Mute,
    )
    .await?;

    let mut member = guild_id.member(&ctx, user.id).await?;
    // unwrap is safe to use here as there is already a check for `None` prior to this expression
    member.remove_role(&ctx.http(), mute_role.unwrap()).await?;

    database::add_moderation(
        &ctx.data().database,
        guild_id,
        user.id,
        moderator.id,
        ModerationType::Unmute,
        administered_at,
        None,
        reason.as_deref(),
    )
    .await?;

    Ok(())
}

fn unmute_help() -> String {
    String::from(
        "Mute a user (with an optional specified time).
Example: %mute @Joshument#0001 3d keeps procrastinating on the modlogs command
        ",
    )
}

/// Get the mod logs for a specified user
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "KICK_MEMBERS",
    help_text_fn = "modlogs_help",
    category = "moderation"
)]
pub async fn modlogs(
    ctx: crate::Context<'_>,
    #[description = "User to get modlogs from"] user: serenity_prelude::User,
    #[description = "Modlogs page"] page: Option<usize>,
) -> Result<(), crate::DynError> {
    let page = match page {
        Some(page) => page,
        None => 1,
    };

    let max_page = database::get_modlog_count(
        &ctx.data().database,
        ctx.guild_id().expect("Failed to get guild id!"),
        user.id,
    )
    .await?
        / 10
        + 1;
    let modlog_page = database::get_modlog_page(
        &ctx.data().database,
        ctx.guild_id().expect("Failed to get guild id!"),
        user.id,
        page,
        10,
    )
    .await?;
    ctx.send(CreateReply::default().embed({
        let mut e = CreateEmbed::default().title(format!("Modlogs for {}", user.name));

        e = modlog_embed(e, modlog_page);

        e.footer(CreateEmbedFooter::new(format!(
            "Page {} of {}",
            page, max_page
        )))
        .color(colors::BLUE)
    }))
    .await?;

    Ok(())
}

fn modlogs_help() -> String {
    String::from(
        "Get the modlogs for the specified user.
Example: %modlogs @Joshument#0001 1
        ",
    )
}
