use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let response_time_ms = Timestamp::now().timestamp_millis() - msg.timestamp.timestamp_millis();

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e
            .color(0xa6e3a1)
            .field("Pong!", format!("Reply time: {}ms", response_time_ms), true)
        )
    }).await?;

    Ok(())
}