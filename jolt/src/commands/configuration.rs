// guild-specific configuration commands

// TODO: add command for setting server prefix (backend is already finished)
mod types;

use std::time::Duration;

use poise::serenity_prelude;

use crate::colors;
use crate::commands::configuration::types::*;
use crate::database;

/// Set or change the mute role of the server
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_GUILD",
    help_text_fn = "mute_role_help",
    category = "configuration",
    rename = "muterole"
)]
pub async fn mute_role(
    ctx: crate::Context<'_>,
    #[description = "Mute role"]
    #[rename = "role"]
    role_id: serenity_prelude::RoleId,
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
    String::from(
        "Set the mute role in the server
**NOTE**: This does *not* change the permissions of the role, you will have to set them up yourself.
Example: %muterole @Muted
    ",
    )
}

/// Set or change the logging channel of the server
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_GUILD",
    help_text_fn = "logs_channel_help",
    category = "configuration",
    rename = "logschannel"
)]
pub async fn logs_channel(
    ctx: crate::Context<'_>,
    #[description = "Logs channel"]
    #[rename = "channel"]
    channel_id: serenity_prelude::ChannelId,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Couldn't get guild id!");
    let database = ctx.data().database.clone();

    database::set_logs_channel(&database, guild_id, channel_id).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::GREEN).description(format!(
                "Channel <#{}> has been assigned as the logs channel.",
                channel_id
            ))
        })
    })
    .await?;

    Ok(())
}

fn logs_channel_help() -> String {
    String::from(
        "Set or change the logs channel for the server
Example: %logschannel #logs
    ",
    )
}

/// Set or change the prefix for text-based commands in the server
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_GUILD",
    help_text_fn = "set_prefix_help",
    category = "moderation",
    rename = "setprefix"
)]
pub async fn set_prefix(
    ctx: crate::Context<'_>,
    #[description = "prefix"] prefix: String,
) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("failed to get guild id!");

    database::set_prefix(&ctx.data().database, guild_id, &prefix).await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::GREEN)
                .description(format!("Changed command prefix to {}.", prefix))
        })
    })
    .await?;

    Ok(())
}

fn set_prefix_help() -> String {
    String::from(
        "Set or change the prefix for the server
Example: %setprefix ~
    ",
    )
}

/// Set up all configuration options in an interactive fashion.
/// Ideal for first time setups.
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_GUILD",
    help_text_fn = "logs_channel_help",
    category = "moderation",
    rename = "setup"
)]
pub async fn setup(ctx: crate::Context<'_>) -> Result<(), crate::DynError> {
    async fn get_answer(
        ctx: &crate::Context<'_>,
        timeout: std::time::Duration,
    ) -> Result<String, ResponseTimedOut> {
        let response = ctx
            .channel_id()
            .await_reply(&ctx.discord().shard)
            .timeout(timeout)
            .author_id(ctx.author().id)
            .await;

        let response = match response {
            Some(response) => Ok(response),
            None => Err(ResponseTimedOut(timeout)),
        }?;

        Ok(response.content.clone())
    }

    let guild_id = ctx.guild_id().expect("Could not get guild ID!");
    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::GREEN).title("Setup").description(
                "Welcome! This command will guide you through the general setup of the server. \
                    If at any time you would like to quit, please respond with `quit`. \
                    You may also skip the option by responding with `*`.",
            )
        })
    })
    .await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::GREEN)
                .title("Prefix")
                .description(format!(
                    "What prefix would you like for your server? \
                    \nYour prefix determines what will be used for **non-slash commands**. The default prefix is {}.",
                    &ctx.data().config.prefix
                ))
        })
    })
    .await?;

    let prefix = get_answer(&ctx, Duration::from_secs(30)).await?;
    if prefix != "*" {
        database::set_prefix(&ctx.data().database, guild_id, &prefix).await?;
    }

    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::GREEN)
                .title("Logs Channel")
                .description(format!(
                    "What channel would you like to be the logs channel? \
                    \nThe logs channel is where **moderation actions** will be logged. \
                    This can be important when it comes to knowing which actions have been done in your server. \
                    \nBy default, there is no logs channel.",
                ))
        })
    })
    .await?;

    let mut logs_channel_global: Option<serenity_prelude::ChannelId> = None;
    loop {
        let logs_channel = get_answer(&ctx, Duration::from_secs(30)).await?;
        if logs_channel == "*" {
            break;
        }

        let channel_id: serenity_prelude::ChannelId = {
            let stripped_id = logs_channel
                .strip_prefix("<#")
                .unwrap_or(&logs_channel)
                .strip_suffix(">")
                .unwrap_or(&logs_channel);

            let id_raw: Result<u64, _> = stripped_id.parse();

            match id_raw {
                Ok(id) => serenity_prelude::ChannelId(id),
                Err(_) => {
                    ctx.send(|m| {
                        m.embed(|e| {
                            e.color(colors::RED)
                                .title("Error!")
                                .description("Please enter a valid channel id!")
                        })
                    })
                    .await?;
                    continue;
                }
            }
        };

        database::set_logs_channel(
            &ctx.data().database,
            &ctx.guild_id().expect("failed to get guild id!"),
            channel_id,
        )
        .await?;
        logs_channel_global = Some(channel_id);
        break;
    }

    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::GREEN)
                .title("Logs Channel")
                .description(format!(
                    "What role would you like to use for the mute role? \
                    The mute role is given to users who have been muted, as a way to change their permissions \
                    (typically to remove their ability to talk). By default, there is no set mute role."
                ))
        })
    })
    .await?;

    let mut mute_role_global: Option<serenity_prelude::RoleId> = None;
    loop {
        let mute_role = get_answer(&ctx, Duration::from_secs(30)).await?;
        if mute_role == "*" {
            break;
        }

        let mute_role_id: serenity_prelude::RoleId = {
            let stripped_id = mute_role
                .strip_prefix("<@&")
                .unwrap_or(&mute_role)
                .strip_suffix(">")
                .unwrap_or(&mute_role);

            let id_raw: Result<u64, _> = stripped_id.parse();

            match id_raw {
                Ok(id) => serenity_prelude::RoleId(id),
                Err(_) => {
                    ctx.send(|m| {
                        m.embed(|e| {
                            e.color(colors::RED)
                                .title("Error!")
                                .description("Please enter a valid role id!")
                        })
                    })
                    .await?;
                    continue;
                }
            }
        };

        database::set_mute_role(
            &ctx.data().database,
            &ctx.guild_id().expect("failed to get guild id!"),
            mute_role_id,
        )
        .await?;
        mute_role_global = Some(mute_role_id);
        break;
    }

    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::GREEN)
                .title("Setup finished!")
                .description(format!(
                    " \
                **Prefix**: {} \n\
                **Logs Channel**: <#{}> \n\
                **Mute Role**: <@&{}>",
                    if prefix != "*" {
                        prefix
                    } else {
                        String::from("Skipped")
                    },
                    if let Some(channel) = logs_channel_global {
                        channel.0.to_string()
                    } else {
                        String::from("Skipped")
                    },
                    if let Some(role) = mute_role_global {
                        role.0.to_string()
                    } else {
                        String::from("Skipped")
                    }
                ))
        })
    })
    .await?;

    Ok(())
}
