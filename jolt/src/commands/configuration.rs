// guild-specific configuration
use poise::serenity_prelude;

use crate::database;
use crate::colors;

/// Set or change the mute role of the server
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "ADMINISTRATOR",
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
                "This action does *not* change the permissions of any channels, make sure you set them up before using the mute commands.", 
                false
            )
        )
    ).await?;

    Ok(())
}

fn mute_role_help() -> String {
    String::from("Set the mute role in the server
**NOTE**: This does *not* change the permissions of channels, you will have to set them up yourself.
Example: %muterole @Muted
    ")
}