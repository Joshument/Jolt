use std::fs;
use std::time::Duration;

use serenity::client::Cache;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::prelude::*;

use humantime;

use crate::Config;

#[command]
#[required_permissions("BAN_MEMBERS")]
pub async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = args.single::<UserId>()?;
    let time_string = args.single::<String>()?;
    let reason = args.rest();
    println!("{}\n{}", user_id, time_string);
    
    let time = humantime::parse_duration(&time_string)?;
    let now = Timestamp::now();
    let unban_time = Timestamp::from_unix_timestamp(now.unix_timestamp() + time.as_secs() as i64)?;

    let http = &ctx.http;
    let cache = &ctx.cache;

    let guild = msg.guild(cache).expect("wa");
    guild.ban(http, user_id, 0).await?;

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| { e
            .color(0xa6e3a1)
            .field("Zap!", format!("User <@{}> has been banned until <t:{}:F>", user_id.as_u64(), unban_time.unix_timestamp()), true);

            if reason.len() > 0 {
                e.field("Reason:", reason, false);
            }

            e
        })
    }).await?;

    Ok(())
}