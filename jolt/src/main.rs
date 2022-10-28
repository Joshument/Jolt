mod commands;
mod database;
mod colors;

use std::sync::Arc;
use std::{fs, time::Instant};
use std::error::Error;

use commands::moderation::types::ModerationType;
// use commands::moderation::types::ModerationType;
use poise::{serenity_prelude, PrefixFrameworkOptions, BoxFuture};
use serenity_prelude::GatewayIntents;
use serde::{Deserialize, Serialize};
use sqlx::sqlite;

use commands::meta::*;
use commands::moderation::*;
use commands::configuration::*;

// const GIT_HASH: &str = env!("GIT_HASH");
const VERSION: &str = concat!("git-", env!("GIT_HASH"));

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
    prefix: String,
    database: String,
}

pub struct Handler;

#[poise::async_trait]
impl serenity_prelude::EventHandler for Handler {
    async fn ready(&self, _: serenity_prelude::Context, ready: serenity_prelude::Ready) {
        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            println!("{} is connected on shard {}/{}!", ready.user.name, shard[0], shard[1],);
        }
    }

    async fn resume(&self, _: serenity_prelude::Context, _: serenity_prelude::ResumedEvent) {
        println!("Resumed!");
    }
}

pub struct Data {
    database: Arc<sqlx::SqlitePool>,
    uptime: Instant,
}

pub type DynError = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, DynError>;
pub type FrameworkError<'a> = poise::FrameworkError<'a, Data, DynError>;

async fn on_error(err: crate::FrameworkError<'_>) {
    let error_message = match &err {
        poise::FrameworkError::Command  {error, ..}
        | poise::FrameworkError::ArgumentParse {error, ..} => error.to_string(),
        poise::FrameworkError::MissingUserPermissions { missing_permissions, .. } => {
            format!(
                "You do not have the permissions{} to run this command!",
                format!(
                    " `{}` ",
                    if let Some(permissions) = missing_permissions {
                        permissions.to_string()
                    } else {
                        String::new()
                    }
                )
            )
        }
        _ => String::from("error is not intentional; please send this to the developers (/info)")
    };

    err.ctx().expect("Failed to get context of error!").send(|m| m
        .embed(|e| e
            .color(colors::RED)
                .field("Error!", error_message, false)
        )
    ).await.expect("Failed to send the error message!");
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let raw_config = fs::read_to_string("./config.json")?;
    let config: Config = serde_json::from_str(&raw_config)?;

    let database = sqlite::SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(
            sqlite::SqliteConnectOptions::new()
                .filename(&config.database)
                .create_if_missing(true)
        )
        .await
        .expect("Couldn't connect to the database!");

    sqlx::migrate!("./../migrations").run(&database).await.expect("Couldn't run database migrations!");
    /*
    let framework =
        StandardFramework::new()
        .configure(|c| c.owners(owners).prefix(config.prefix))
        .group(&GENERAL_GROUP)
        .group(&MODERATORS_GROUP)
        .on_dispatch_error(dispatch_error)
        .after(after);
    */

    let uptime = Instant::now();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                // Meta
                ping(),
                info(),

                // Moderation
                warn(),
                delwarn(),
                warnings(),
                ban(),
                unban(),
                kick(),
                timeout(),
                untimeout(),
                mute(),
                unmute(),
                modlogs(),

                // Configuration
                mute_role(),
                logs_channel(),
            ],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(config.prefix),
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .token(config.token)
        .intents(GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .user_data_setup(
            move |ctx, _ready, framework| {
                Box::pin(async move { 
                    let commands = &framework.options().commands;
                    let create_commands = poise::builtins::create_application_commands(commands);
    
                    serenity_prelude::Command::set_global_application_commands(&ctx.http, |b| {
                        *b = create_commands;
                        b
                    }).await?;

                    /*
                    let guild_id = serenity_prelude::GuildId(1033905219257516032);
                    guild_id.set_application_commands(&ctx.http, |b| b).await?;
                    */
                    let database = std::sync::Arc::new(database);
                    let moderations_database = database.clone();
                    let moderations_ctx = ctx.clone();
                    tokio::spawn(async move {
                        loop {
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                
                            let current_time = serenity_prelude::Timestamp::now().unix_timestamp();
                
                            let entries = sqlx::query!(
                                "SELECT guild_id, user_id, moderation_type FROM moderations WHERE expiry_date < ? AND active = TRUE",
                                current_time
                            )
                                .fetch_all(&*moderations_database) 
                                .await
                                .expect("Failed to get current moderations!");
                
                            for entry in entries {
                                let guild_id = serenity_prelude::GuildId(entry.guild_id as u64);
                                let user_id = serenity_prelude::UserId(entry.user_id as u64);
                                let moderation_type: ModerationType = (entry.moderation_type as u8).try_into()
                                    .expect("Failed to convert moderation_type into ModerationType enum!");
                
                                match moderation_type {
                                    ModerationType::Ban => guild_id.unban(&moderations_ctx.http, user_id).await
                                        .expect(format!("Failed to unban user {} from {}", user_id, guild_id).as_str()),
                                    ModerationType::Mute => {
                                        let role = database::get_mute_role(&moderations_database, guild_id)
                                            .await.expect("Failed to open database!");
                                        if let Some(role_id) = role {
                                            let mut member = guild_id.member(&moderations_ctx, user_id).await.expect("Failed to get member!");
                                            member.remove_role(&moderations_ctx.http, role_id).await.expect("Failed to remove role from member!");
                                        }
                                    },
                                    _ => () // Either there is no timed event, or the event has a built-in expiry (timeout)
                                }
                            }

                            sqlx::query!(
                                "UPDATE moderations SET active = FALSE WHERE expiry_date < ?",
                                current_time
                            )
                                .execute(&*moderations_database)
                                .await
                                .expect("Failed to write to database!");
                        }
                    });

                    Ok(Data {
                        database,
                        uptime,
                    }
                )})
            }
        );

    framework.run_autosharded().await?;

    /*
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
        data.insert::<commands::meta::Uptime>(Arc::new(uptime));
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
    */

    Ok(())
}
