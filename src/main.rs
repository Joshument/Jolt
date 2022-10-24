mod messagehandler;
mod slashcommands;
mod commands;

use std::fs;
use std::error::Error;

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use serenity::async_trait;
use serenity::client::Cache;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::DispatchError;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::framework::standard::macros::hook;
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::prelude::*;

use serde::{Deserialize, Serialize};

use commands::meta::*;
use commands::moderation::*;
use tokio::time::sleep;

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
    prefix: String,
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(ban)]
struct Moderators;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            println!("{} is connected on shard {}/{}!", ready.user.name, shard[0], shard[1],);
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        println!("Resumed!");
    }
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    } else {
        println!("{:?}", error)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let raw_config = fs::read_to_string("./config.json")?;
    let config: Config = serde_json::from_str(&raw_config)?;

    let http = Http::new(&config.token);

    // Get bot owners and bot id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why)
    };

    let framework =
        StandardFramework::new()
        .configure(|c| c.owners(owners).prefix(config.prefix))
        .group(&GENERAL_GROUP)
        .group(&MODERATORS_GROUP)
        .on_dispatch_error(dispatch_error);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&config.token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating client!");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone())
    }

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;

            let lock = shard_manager.lock().await;
            let shard_runners = lock.runners.lock().await;

            for (id, runner) in shard_runners.iter() {
                println!(
                    "Shard ID {} is {} with a latency of {:?}",
                    id, runner.stage, runner.latency,
                );
            }
        }
    });

    if let Err(why) = client.start_shards(1).await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}
