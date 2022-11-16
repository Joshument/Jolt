// guild-specific configuration commands
use poise::serenity_prelude;

use crate::database;
use crate::colors;

/// Set or change the mute role of the server
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_GUILD",
    help_text_fn = "mute_role_help",
    category = "moderation",
    rename = "muterole"
)]
pub async fn mute_role(
    ctx: crate::Context<'_>,
    #[description = "Mute role"] #[rename = "role"] role_id: serenity_prelude::RoleId
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Couldn't get guild id!");
    let database = ctx.data().database.clone();

    database::set_mute_role(&database, guild_id, role_id).await?;

    ctx.send(|m| m
        .embed(|e| e
            .color(colors::GREEN)
            .description(format!("Role <@&{}> has been assigned as the mute role.", role_id))
            .field(
                "NOTE", 
                "This action does *not* change the permissions of the role, make sure you set them up before using the mute commands.", 
                false
            )
        )
    ).await?;

    Ok(())
}

fn mute_role_help() -> String {
    String::from("Set the mute role in the server
**NOTE**: This does *not* change the permissions of the role, you will have to set them up yourself.
Example: %muterole @Muted
    ")
}

/// Set or change the logging channel of the server
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_GUILD",
    help_text_fn = "logs_channel_help",
    category = "moderation",
    rename = "logschannel"
)]
pub async fn logs_channel(
    ctx: crate::Context<'_>,
    #[description = "Logs channel"] #[rename = "channel"] channel_id: serenity_prelude::ChannelId
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Couldn't get guild id!");
    let database = ctx.data().database.clone();

    database::set_logs_channel(&database, guild_id, channel_id).await?;

    ctx.send(|m| m
        .embed(|e| e
            .color(colors::GREEN)
            .description(format!("Channel <#{}> has been assigned as the logs channel.", channel_id))
        )
    ).await?;

    Ok(())
}

fn logs_channel_help() -> String {
    String::from("Set or change the logs channel for the server
Example: %logschannel #logs
    ")
}