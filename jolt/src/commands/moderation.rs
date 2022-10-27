pub mod types;
mod utilities;

use crate::database;
use crate::colors;
use crate::commands::moderation::types::*;
use crate::commands::moderation::utilities::*;

use poise::serenity_prelude;



/// Ban a user for a specified amount of time
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
        &ctx.discord(), 
        &dm_channel, 
        &append_expiry_date(&format!("You have been banned from **{}**", user.id.as_u64()), expiry_date),
        colors::RED, 
        &ctx.channel_id(), 
        &append_expiry_date(&format!("User <@{}> has been banned", user.id.as_u64()), expiry_date),
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user.id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    database::add_moderation(&ctx.data(), guild_id, user.id, ModerationType::Ban, administered_at, expiry_date, reason.as_deref()).await?;

    if let Some(reason) = &reason {
        guild_id.ban_with_reason(&ctx.discord().http, &user.id, 0, &reason).await?;
    } else {
        guild_id.ban(&ctx.discord().http, &user.id, 0).await?;
    }

    Ok(())
}

fn ban_help() -> String {
    "Ban a user for a specified amount of time".to_string()
}

#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "KICK_MEMBERS",
    required_bot_permissions = "KICK_MEMBERS",
    help_text_fn = "ban_help",
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
        &ctx.discord(), 
        &dm_channel, 
        &format!(
            "You have been kicked from **{}**!", 
            guild_id.name(&ctx.discord().cache).expect("Failed to get guild name!").as_str(),
        ), 
        colors::RED, 
        &ctx.channel_id(), 
        &format!("User <@{}> has been kicked", user.id.as_u64()), 
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user.id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    database::add_moderation(&ctx.data(), guild_id, user.id, ModerationType::Kick, administered_at, None, reason.as_deref()).await?;

    if let Some(reason) = &reason {
        guild_id.kick_with_reason(&ctx.discord().http, user.id, &reason).await?;
    } else {
        guild_id.kick(&ctx.discord().http, user.id,).await?;
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS",
    help_text_fn = "ban_help",
    category = "moderation",
)]
pub async fn timeout(
    ctx: crate::Context<'_>,
    #[description = "User to ban"] user: serenity_prelude::User,
    #[description = "Length of the ban"] length: humantime::Duration,
    #[description = "Reason for ban"] reason: Option<String>,
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
        &ctx.discord(), 
        &dm_channel, 
        &format!("You have been timed out from **{}** until <t:{}:F>", user.id.as_u64(), expiry_date.unix_timestamp()),
        colors::RED, 
        &ctx.channel_id(), 
        &format!("User <@{}> has been timed out until <t:{}:F>", user.id.as_u64(), expiry_date.unix_timestamp()),
        colors::GREEN,
        &format!(
            "I was unable to DM <@{}> about their moderation.", 
            user.id.as_u64()
        ), 
        colors::RED, 
        reason.as_deref()
    ).await?;

    database::add_moderation(&ctx.data(), guild_id, user.id, ModerationType::Ban, administered_at, Some(expiry_date), reason.as_deref()).await?;

    Ok(())
}