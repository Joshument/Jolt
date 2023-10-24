// guild-specific configuration commands

pub mod error;

use std::time::Duration;

use poise::futures_util::StreamExt;
use poise::serenity_prelude::{
    self, collect, ActionRow, ActionRowComponent, ChannelType, ComponentInteractionDataKind,
    ComponentType, CreateActionRow, CreateButton, CreateEmbed, CreateInputText,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateModal,
    CreateSelectMenu, CreateSelectMenuKind, EditMessage, Event, InputTextStyle, Interaction,
    InteractionCreateEvent,
};
use poise::{CreateReply, MessageDispatchTrigger};
use tokio::select;

use crate::colors;
use crate::commands::configuration::error::ConfigurationError;
use crate::database;

#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_GUILD",
    category = "configuration",
    rename = "testcommand"
)]
pub async fn test_command(ctx: crate::Context<'_>) -> Result<(), crate::DynError> {
    let reply = ctx
        .send(
            CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .color(colors::GREEN)
                        .title("I love buttons!!!"),
                )
                .components(vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    "mute_role_select",
                    CreateSelectMenuKind::User {
                        default_users: None,
                    },
                ))]),
        )
        .await?;

    let message = reply.into_message().await?;

    let interaction = match message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_secs(60))
        .await
    {
        Some(x) => Ok(x),
        None => Err(ConfigurationError::ResponseTimedOut(Duration::from_secs(
            60,
        ))),
    }?;

    let user_id = match &interaction.data.kind {
        ComponentInteractionDataKind::UserSelect { values } => values[0],
        _ => panic!("how did this happen"),
    };

    let member = ctx
        .guild_id()
        .expect("Failed to get guild!")
        .member(ctx, user_id)
        .await?;

    interaction
        .create_response(
            ctx,
            serenity_prelude::CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default().embed(
                    CreateEmbed::default()
                        .color(colors::BLUE)
                        .description(format!("FUCK YOU <@{}>", member.user.id)),
                ),
            ),
        )
        .await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_GUILD",
    category = "configuration",
    rename = "configure"
)]
pub async fn configure(ctx: crate::Context<'_>) -> Result<(), crate::DynError> {
    let guild_id = ctx.guild_id().expect("Couldn't get guild id!");
    let database = ctx.data().database.clone();
    let user = ctx.author();

    let main_tab = CreateReply::default()
        .embed(
            CreateEmbed::default()
                .color(colors::GREEN)
                .title("Bot Configuration")
                .description(
                    "Please select one of the below buttons to modify the bot's server settings.",
                ),
        )
        .components(vec![CreateActionRow::Buttons(vec![
            CreateButton::new("prefix_button").label("Prefix"),
            CreateButton::new("mute_role_button").label("Mute Role"),
            CreateButton::new("logs_channel_button").label("Logs Channel"),
            CreateButton::new("exit_button").label("Exit"),
        ])])
        .ephemeral(true);
    let exit = CreateInteractionResponseMessage::default()
        .embed(
            CreateEmbed::default()
                .color(colors::BLUE)
                .title("Configuration Finished"),
        )
        .components(vec![])
        .ephemeral(true); // components method is necessary to overwrite the pre-existing ones
    let prefix_input =
        CreateModal::default().components(vec![CreateActionRow::InputText(CreateInputText::new(
            InputTextStyle::Short,
            "Enter new prefix",
            "prefix_text_input",
        ))]);
    let mute_role_input = CreateInteractionResponseMessage::default()
        .embed(
            CreateEmbed::default()
                .color(colors::GREEN)
                .title("Mute Role")
                .description(
                    "Select the role to be assigned on mute. \
                            `timeout` is typically considered to be a better option, \
                            but this is kept for compatability if you so desire.",
                ),
        )
        .components(vec![
            CreateActionRow::SelectMenu(CreateSelectMenu::new(
                "mute_role_select_input",
                CreateSelectMenuKind::Role {
                    default_roles: database::get_mute_role(&database, guild_id)
                        .await?
                        .map(|id| vec![id]),
                },
            )),
            CreateActionRow::Buttons(vec![
                CreateButton::new("back_button").label("Back"),
                CreateButton::new("exit_button").label("Exit"),
            ]),
        ])
        .ephemeral(true);
    let logs_channel_input = CreateInteractionResponseMessage::default()
        .embed(
            CreateEmbed::default()
                .color(colors::GREEN)
                .title("Logs Channel")
                .description(
                    "Select the channel to record logs to. \
                These logs are a collection of moderation actions done in your server.",
                ),
        )
        .components(vec![
            CreateActionRow::SelectMenu(CreateSelectMenu::new(
                "logs_channel_select_input",
                CreateSelectMenuKind::Channel {
                    channel_types: Some(vec![ChannelType::Text]),
                    default_channels: database::get_logs_channel(&database, guild_id)
                        .await?
                        .map(|id| vec![id]),
                },
            )),
            CreateActionRow::Buttons(vec![
                CreateButton::new("back_button").label("Back"),
                CreateButton::new("exit_button").label("Exit"),
            ]),
        ])
        .ephemeral(true);

    let reply = ctx.send(main_tab.clone()).await?;
    let message = reply.into_message().await?;

    let mut stream = collect(&ctx.serenity_context().shard, |event| match event {
        Event::InteractionCreate(event) => Some(event.clone()),
        _ => None,
    });

    loop {
        let event = stream.next().await;
        match event {
            Some(event) => match &event.interaction {
                Interaction::Component(interaction) => {
                    if interaction.user != *user {
                        continue;
                        /*
                        interaction
                            .create_response(
                                &ctx,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::default().embed(
                                        CreateEmbed::default()
                                            .title("This isn't your command!")
                                            .color(colors::RED),
                                    ),
                                ),
                            )
                            .await?
                        */
                    };
                    match &interaction.data.kind {
                        ComponentInteractionDataKind::Button => {
                            match interaction.data.custom_id.as_str() {
                                "exit_button" => {
                                    interaction
                                        .create_response(
                                            &ctx,
                                            CreateInteractionResponse::UpdateMessage(exit.clone()),
                                        )
                                        .await?
                                }
                                "back_button" => {
                                    interaction
                                        .create_response(
                                            &ctx,
                                            CreateInteractionResponse::UpdateMessage(
                                                main_tab.clone().to_slash_initial_response(),
                                            ),
                                        )
                                        .await?
                                }
                                "prefix_button" => {
                                    interaction
                                        .create_response(
                                            &ctx,
                                            CreateInteractionResponse::Modal(prefix_input.clone()),
                                        )
                                        .await?
                                }

                                "mute_role_button" => {
                                    interaction
                                        .create_response(
                                            &ctx,
                                            CreateInteractionResponse::UpdateMessage(
                                                mute_role_input.clone(),
                                            ),
                                        )
                                        .await?
                                }
                                "logs_channel_button" => {
                                    interaction
                                        .create_response(
                                            &ctx,
                                            CreateInteractionResponse::UpdateMessage(
                                                logs_channel_input.clone(),
                                            ),
                                        )
                                        .await?
                                }
                                _ => panic!("button not expected or unimplemented!"),
                            }
                        }
                        ComponentInteractionDataKind::RoleSelect { values } => {
                            match interaction.data.custom_id.as_str() {
                                "mute_role_select_input" => {
                                    let role_id = values[0];
                                    database::set_mute_role(&database, guild_id, role_id).await?;
                                    interaction
                                        .create_response(
                                            &ctx.http(),
                                            CreateInteractionResponse::UpdateMessage(
                                                main_tab.clone().to_slash_initial_response(),
                                            ),
                                        )
                                        .await?
                                }
                                _ => (),
                            }
                        }
                        ComponentInteractionDataKind::ChannelSelect { values } => {
                            match interaction.data.custom_id.as_str() {
                                "logs_channel_select_input" => {
                                    let channel_id = values[0];
                                    database::set_logs_channel(&database, guild_id, channel_id)
                                        .await?;
                                    interaction
                                        .create_response(
                                            &ctx.http(),
                                            CreateInteractionResponse::UpdateMessage(
                                                main_tab.clone().to_slash_initial_response(),
                                            ),
                                        )
                                        .await?
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                }
                Interaction::Modal(interaction) => {
                    for action_row in &interaction.data.components {
                        for component in &action_row.components {
                            match component {
                                ActionRowComponent::InputText(input) => match &input.value {
                                    Some(value) => {
                                        database::set_prefix(&database, guild_id, &value).await?;
                                        interaction
                                            .create_response(
                                                &ctx.http(),
                                                CreateInteractionResponse::Acknowledge,
                                            )
                                            .await?
                                    }
                                    None => (),
                                },
                                _ => (),
                            }
                        }
                    }
                }
                _ => (),
            },
            None => (),
        }
    }
}

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

    ctx.send(CreateReply::default()
        .embed( CreateEmbed::default()
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

    ctx.send(
        CreateReply::default().embed(CreateEmbed::default().color(colors::GREEN).description(
            format!(
                "Channel <#{}> has been assigned as the logs channel.",
                channel_id
            ),
        )),
    )
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

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .color(colors::GREEN)
                .description(format!("Changed command prefix to {}.", prefix)),
        ),
    )
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

/* No point in remaking all of this just to get rid of it later lol
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
    ) -> Result<String, error::SetupError> {
        let response = ctx
            .channel_id()
            .await_reply(&ctx.discord().shard)
            .timeout(timeout)
            .author_id(ctx.author().id)
            .await;

        let response = match response {
            Some(response) => Ok(response),
            None => Err(error::SetupError::ResponseTimedOut(timeout)),
        }?;

        Ok(response.content.clone())
    }

    async fn setup_message(
        ctx: &crate::Context<'_>,
        title: &str,
        description: &str,
    ) -> Result<(), serenity_prelude::Error> {
        ctx.send(|m| m.embed(|e| e.color(colors::GREEN).title(title).description(description)))
            .await?;

        Ok(())
    }

    let guild_id = ctx.guild_id().expect("Could not get guild ID!");

    setup_message(
        &ctx,
        "Setup",
        "Welcome! This command will guide you through the general setup of the server. \
        If at any time you would like to quit, please respond with `quit`. \
        You may also skip the option by responding with `*`.",
    )
    .await?;

    setup_message(
        &ctx,
        "Prefix",
        &format!(
            "What prefix would you like for your server? \
            \nYour prefix determines what will be used for **non-slash commands**. The default prefix is {}.",
            &ctx.data().config.prefix
        )
    ).await?;

    let prefix = get_answer(&ctx, Duration::from_secs(30)).await?;
    if prefix == "quit" {
        return Err(Box::new(error::SetupError::OperationCancelled));
    } else if prefix != "*" {
        database::set_prefix(&ctx.data().database, guild_id, &prefix).await?;
    }
    setup_message(
        &ctx,
        "Logs Channel",
        "What channel would you like to be the logs channel? \n\
        The logs channel is where **moderation actions** will be logged. \
        This can be important when it comes to knowing which actions have been done in your server. \n\
        By default, there is no logs channel.",
    ).await?;

    // This CANNOT be the best way to do this I swear to god
    let mut maybe_logs_channel: Option<serenity_prelude::ChannelId> = None;
    loop {
        let logs_channel = get_answer(&ctx, Duration::from_secs(30)).await?;
        if logs_channel == "*" {
            break;
        }

        let channel_id: serenity_prelude::ChannelId = {
            let id = serenity_prelude::ChannelId::convert(
                &ctx.discord(),
                Some(guild_id),
                None,
                &logs_channel,
            )
            .await;

            match id {
                Ok(id) => id,
                Err(_) => {
                    send_error(&ctx, "Please provide a valid channel!").await?;
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
        maybe_logs_channel = Some(channel_id);
        break;
    }

    setup_message(
        &ctx,
        "Mute Role",
        "What role would you like to use for the mute role? \
        The mute role is given to users who have been muted, as a way to change their permissions \
        (typically to remove their ability to talk). By default, there is no set mute role.",
    )
    .await?;

    let mut maybe_mute_role: Option<serenity_prelude::RoleId> = None;
    loop {
        let mute_role = get_answer(&ctx, Duration::from_secs(30)).await?;
        if mute_role == "*" {
            break;
        }

        let mute_role_id: serenity_prelude::RoleId = {
            let id =
                serenity_prelude::RoleId::convert(&ctx.discord(), Some(guild_id), None, &mute_role)
                    .await;

            match id {
                Ok(id) => id,
                Err(_) => {
                    send_error(&ctx, "Please provide a valid role!").await?;
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
        maybe_mute_role = Some(mute_role_id);
        break;
    }

    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::GREEN)
                .title("Setup finished!")
                .description(format!(
                    " \
                **Prefix**: {} \n\
                **Logs Channel**: {} \n\
                **Mute Role**: {}",
                    if prefix != "*" {
                        prefix
                    } else {
                        String::from("Skipped")
                    },
                    if let Some(channel) = maybe_logs_channel {
                        format!("<#{}>", channel.0.to_string())
                    } else {
                        String::from("Skipped")
                    },
                    if let Some(role) = maybe_mute_role {
                        format!("<@&{}>", role.0.to_string())
                    } else {
                        String::from("Skipped")
                    }
                ))
        })
    })
    .await?;

    Ok(())
}
*/
