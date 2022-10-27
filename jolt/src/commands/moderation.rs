pub mod types;
mod utilities;

use crate::database;
use crate::colors;
use crate::commands::moderation::types::*;
use crate::commands::moderation::utilities::*;

use poise::serenity_prelude;

/// Ban a user (with an optional specified time)
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS",
    help_text_fn = "ban_help",
    category = "moderation",
)]
pub async fn ban(
    ctx: crate::Context<'_>,
    #[description = "User to ban"] user: serenity_prelude::User,
    #[description = "Length of the ban"] length: Option<humantime::Duration>,
    #[description = "Reason for ban"] reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let administered_at = ctx.created_at();
    // Replaces Option<Duration> into Option<Timestamp>
    // .transpose()? brings out the inner result propagate upstream with `?`
    // Using `?` inside of .map() would instead return it to the closure, therefore making it invalid.
    let expiry_date = length.map(|duration| 
        serenity_prelude::Timestamp::from_unix_timestamp(administered_at.unix_timestamp() + duration.as_secs() as i64)
    ).transpose()?;

    let dm_channel = user.create_dm_channel(&ctx.discord().http).await?;

    send_moderation_messages(
        &ctx, 
        &dm_channel, 
        &append_expiry_date(&format!("You have been banned from **{}**", 
            &guild_id.name(&ctx.discord().cache).expect("Failed to get guild name!")), 
            expiry_date
        ),
        colors::RED, 
        "Zap!", 
        &append_expiry_date(&format!("User <@{}> has been banned", user.id.as_u64()), expiry_date),
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user.id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    database::add_moderation(
        &ctx.data().database, 
        guild_id, 
        user.id, 
        ModerationType::Ban, 
        administered_at, 
        expiry_date, 
        reason.as_deref()
    ).await?;

    if let Some(reason) = &reason {
        guild_id.ban_with_reason(&ctx.discord().http, &user.id, 0, &reason).await?;
    } else {
        guild_id.ban(&ctx.discord().http, &user.id, 0).await?;
    }

    Ok(())
}

fn ban_help() -> String {
    String::from("Kick a user from the server.
Example: %ban @Joshument#0001 10s joined 10 seconds too early
        ")
}

/// Unban a user (with an optional specified time)
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS",
    help_text_fn = "unban_help",
    category = "moderation",
)]
pub async fn unban(
    ctx: crate::Context<'_>,
    #[description = "User to unban"] user: serenity_prelude::User,
    #[description = "Reason for the unban"] reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    // let administered_at = ctx.created_at();

    let dm_channel = user.create_dm_channel(&ctx.discord().http).await?;

    send_moderation_messages(
        &ctx, 
        &dm_channel, 
        &format!("You have been unbanned from **{}**",
            &guild_id.name(&ctx.discord().cache).expect("Failed to get guild name!")
        ),
        colors::GREEN, 
        "!paZ", 
        &format!("User <@{}> has been unbanned", user.id.as_u64()),
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user.id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    guild_id.unban(&ctx.discord().http, &user.id).await?;
    database::clear_moderations(&ctx.data().database, guild_id.0 as i64, user.id.0 as i64, ModerationType::Ban).await?;

    Ok(())
}

fn unban_help() -> String {
    String::from("Unban a user from the server.
Example: %unban @Joshument#0001 my perspective of you has changed
        ")
}

/// Kick a user
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "KICK_MEMBERS",
    required_bot_permissions = "KICK_MEMBERS",
    help_text_fn = "kick_help",
    category = "moderation",
)]
pub async fn kick(
    ctx: crate::Context<'_>,
    #[description = "User to kick"] user: serenity_prelude::User,
    #[description = "Reason for kick"] reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let administered_at = ctx.created_at();

    let dm_channel = user.create_dm_channel(&ctx.discord().http).await?;

    send_moderation_messages(
        &ctx, 
        &dm_channel, 
        &format!(
            "You have been kicked from **{}**!", 
            guild_id.name(&ctx.discord().cache).expect("Failed to get guild name!").as_str(),
        ), 
        colors::RED, 
        "Zap!", 
        &format!("User <@{}> has been kicked", user.id.as_u64()), 
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user.id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    database::add_moderation(
        &ctx.data().database, 
        guild_id, 
        user.id, 
        ModerationType::Kick, 
        administered_at, 
        None, 
        reason.as_deref()
    ).await?;

    if let Some(reason) = &reason {
        guild_id.kick_with_reason(&ctx.discord().http, user.id, &reason).await?;
    } else {
        guild_id.kick(&ctx.discord().http, user.id,).await?;
    }

    Ok(())
}

fn kick_help() -> String {
    String::from("Kick a user from the server.
Example: %kick @Joshument#0001 nerd
    ")
}

/// Timeout a user for a specified amount of time
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS",
    help_text_fn = "timeout_help",
    category = "moderation",
)]
pub async fn timeout(
    ctx: crate::Context<'_>,
    #[description = "User to timeout"] user: serenity_prelude::User,
    #[description = "Length of the timeout"] length: humantime::Duration,
    #[description = "Reason for timeout"] reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");
    let administered_at = ctx.created_at();
    let expiry_date = 
        serenity_prelude::Timestamp::from_unix_timestamp(administered_at.unix_timestamp() + length.as_secs() as i64)?;

    let dm_channel = user.create_dm_channel(&ctx.discord().http).await?;


    // start with the timeout to see if the specified time is over 28d or not
    let successful_timeout = guild_id
    .member(&ctx.discord().http, &user.id).await?
    .disable_communication_until_datetime(&ctx.discord().http, expiry_date).await;

    if let Err(e) = &successful_timeout {
        match e {
            serenity_prelude::SerenityError::Http(_) => {
                ctx.send(|m| {
                    m.embed(|e| e
                        .color(colors::RED)
                        .description("Timeouts must be shorter than 28 days.")
                    )
                    .ephemeral(true)
                }).await?;
            }
            _ => ()
        }

        // This seems to be the best way to return the error after checking it
        successful_timeout?
    }

    send_moderation_messages(
        &ctx, 
        &dm_channel, 
        &format!("You have been timed out from **{}** until <t:{}:F>", 
            guild_id.name(&ctx.discord().cache).expect("Failed to get guild name!"), 
            expiry_date.unix_timestamp()
        ),
        colors::RED, 
        "Zap!", 
        &format!("User <@{}> has been timed out until <t:{}:F>", user.id.as_u64(), expiry_date.unix_timestamp()),
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user.id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    database::add_moderation(
        &ctx.data().database, 
        guild_id, 
        user.id, 
        ModerationType::Timeout, 
        administered_at, 
        Some(expiry_date), 
        reason.as_deref()
    ).await?;

    Ok(())
}

fn timeout_help() -> String {
    String::from("Time out a user from the server.
Example: %timeout @Paze#2936 10m not a fan of the inconsistencies
            ")
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
    #[description = "Reason for the untimeout"] reason: Option<String>,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Failed to get guild ID!");

    let dm_channel = user.create_dm_channel(&ctx.discord().http).await?;
    database::clear_moderations(&ctx.data().database, guild_id.0 as i64, user.id.0 as i64, ModerationType::Timeout).await?;

    guild_id
        .member(&ctx.discord().http, &user.id).await?
        .enable_communication(&ctx.discord().http).await?;

    send_moderation_messages(
        &ctx, 
        &dm_channel, 
        &format!("You have been untimed out from **{}**",
            &guild_id.name(&ctx.discord().cache).expect("Failed to get guild name!")
        ),
        colors::GREEN, 
        "!paZ", 
        &format!("User <@{}> has been untimed out", user.id.as_u64()),
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user.id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    Ok(())
}

fn untimeout_help() -> String {
    String::from("Revoke the timeoutout a user.
Example: %untimeout @Paze#2936 fan of the consistencies :)
            ")
}