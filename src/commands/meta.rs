use crate::colors;

use std::sync::Arc;
use std::time::Instant;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub struct Uptime;

impl TypeMapKey for Uptime {
    type Value = Arc<Instant>;
}

pub async fn get_uptime(data: &Arc<RwLock<TypeMap>>) -> Arc<Instant> {
    let data = data.read().await;
    data.get::<Uptime>().expect("Failed to get uptime!").clone()
}

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let response_time_ms = Timestamp::now().timestamp_millis() - msg.timestamp.timestamp_millis();

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e
            .color(colors::GREEN)
            .field("Pong!", format!("Reply time: {}ms", response_time_ms), true)
        )
    }).await?;

    Ok(())
}

#[command]
#[aliases(about)]
pub async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let uptime = get_uptime(&ctx.data).await.elapsed();

    msg.channel_id.send_message(&ctx.http, |m| {
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
                    &ctx.cache.guild_count().to_string(),
                    true
                ),
                (
                    "Users",
                    &ctx.cache.user_count().to_string(),
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
                    &ctx.shard_id,
                    &ctx.cache.shard_count(),
                    humantime::format_duration(uptime).to_string(),
                    Timestamp::now().timestamp_millis() - msg.timestamp.timestamp_millis()
                ))
            })
        )
    }).await?;

    Ok(())
}