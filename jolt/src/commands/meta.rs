use crate::colors;

use poise::serenity_prelude;
use serenity_prelude::Timestamp;

/// Get the latency of the bot (in milliseconds)
#[poise::command(
    prefix_command,
    slash_command,
    help_text_fn = "ping_help",
    category = "meta"
)]
pub async fn ping(ctx: crate::Context<'_>) -> Result<(), crate::DynError> {
    let response_time_ms =
        Timestamp::now().timestamp_millis() - ctx.created_at().timestamp_millis();

    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::GREEN).field(
                "Pong!",
                format!("Reply time: {}ms", response_time_ms),
                true,
            )
        })
        .ephemeral(true)
    })
    .await?;

    Ok(())
}

fn ping_help() -> String {
    "Get the latency of the bot (in milliseconds)".to_string()
}

/// Get information related to the bot
#[poise::command(
    prefix_command,
    slash_command,
    help_text_fn = "info_help",
    category = "meta"
)]
pub async fn info(ctx: crate::Context<'_>) -> Result<(), crate::DynError> {
    let uptime = &ctx.data().uptime.elapsed();

    ctx.send(|m| {
        m.embed(|e| e
            .color(colors::BLUE)
            .title("Jolt Bot")
            .fields([
                (
                    "Version",
                    crate::VERSION,
                    true,
                ),
                (
                    "GitHub",
                    "https://github.com/Joshument/Jolt",
                    true
                ),
                (
                    "Maintainer",
                    "Joshument#0001",
                    true
                ),
                (
                    "Servers",
                    &ctx.discord().cache.guild_count().to_string(),
                    true
                ),
                (
                    "Users",
                    &ctx.discord().cache.user_count().to_string(),
                    true,
                ),
                (
                    "Invite",
                    "Not yet :(",
                    true
                )
            ])
            .description("Jolt is licensed under the [BSD 3-Clause License](https://github.com/Joshument/Jolt/blob/main/LICENSE).")
            .footer(|f| {
                f.text(format!(
                    "Shard {}/{} | Uptime {} | Ping {}ms",
                    &ctx.discord().shard_id,
                    &ctx.discord().cache.shard_count(),
                    humantime::format_duration(*uptime).to_string(),
                    Timestamp::now().timestamp_millis() - ctx.created_at().timestamp_millis()
                ))
            })
        )
        .ephemeral(true)
    }).await?;

    Ok(())
}

fn info_help() -> String {
    "Get information related to the bot".to_string()
}
