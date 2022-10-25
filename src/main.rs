mod commands;
mod groups;
mod hooks;
mod database;
mod colors;

use std::fs;
use std::error::Error;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use commands::moderation::ModerationType;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::sqlite;

use groups::*;
use hooks::*;

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
    prefix: String,
    database: String,
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

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


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let raw_config = fs::read_to_string("./config.json")?;
    let config: Config = serde_json::from_str(&raw_config)?;

    let database = sqlite::SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(
            sqlite::SqliteConnectOptions::new()
                .filename(config.database)
                .create_if_missing(true)
        )
        .await
        .expect("Couldn't connect to the database!");

    sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations!");

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
        .on_dispatch_error(dispatch_error)
        .after(after);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;
    let mut client = Client::builder(&config.token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating client!");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<database::Database>(Arc::new(database));
    }

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

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

    let moderations_database = database::get_database(&client.data).await;
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let current_time = Timestamp::now().unix_timestamp();

            let entries = sqlx::query!(
                "SELECT guild_id, user_id, moderation_type FROM moderations WHERE expiry_date < ? AND active = TRUE",
                current_time
            )
                .fetch_all(&*moderations_database) 
                .await
                .expect("Failed to get current moderations!");

            sqlx::query!(
                "UPDATE moderations SET active = FALSE WHERE expiry_date < ?",
                current_time
            )
                .execute(&*moderations_database)
                .await
                .expect("Failed to write to database!");

            for entry in entries {
                let guild_id = GuildId(entry.guild_id as u64);
                let user_id = UserId(entry.user_id as u64);
                let moderation_type: ModerationType = (entry.moderation_type as u8).try_into()
                    .expect("Failed to convert moderation_type into ModerationType enum!");

                match moderation_type {
                    ModerationType::Ban => guild_id.unban(&http, user_id).await
                        .expect(format!("Failed to unban user {} from {}", user_id, guild_id).as_str()),
                    ModerationType::Mute => unimplemented!(),
                    _ => () // There should be no other timed events
                }
            }
        }
    });

    if let Err(why) = client.start_autosharded().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}
