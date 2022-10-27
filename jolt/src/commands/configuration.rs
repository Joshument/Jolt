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

    let guild_id_i64 = guild_id.0 as i64;
    let role_id_i64 = role_id.0 as i64;
    
    sqlx::query!(
        "INSERT INTO guild_settings (guild_id, mute_role_id) VALUES ($1, $2)
        ON CONFLICT (guild_id) DO UPDATE SET mute_role_id=excluded.mute_role_id",
        guild_id_i64,
        role_id_i64
    )
    .execute(&*database)
    .await?;

    Ok(())
}

fn mute_role_help() -> String {
    String::from("Set the mute role in the server
Example: %muterole @Muted
    ")
}