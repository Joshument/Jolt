mod commands;
mod database;
mod colors;

use std::sync::Arc;
use std::{fs, time::Instant};
use std::error::Error;

use commands::moderation::types::ModerationType;
// use commands::moderation::types::ModerationType;
use poise::{serenity_prelude, PrefixFrameworkOptions};
use serenity_prelude::GatewayIntents;
use serde::{Deserialize, Serialize};
use sqlx::sqlite;

use commands::meta::*;
use commands::moderation::*;
use commands::configuration::*;

// This gets the current git commit hash for development builds. See the build.rs file for more information on how this is obtained.
const VERSION: &str = concat!("git-", env!("GIT_HASH"));

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
    prefix: String,
    database: String,
}

pub struct Handler;

// This handler is not super necessary right now, but it helps give a bit of information.
// Later on, this will likely be used for other features.
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

// Some types that poise can use to make things a bit easier to use.
// Prevents a lot of boilerplate as things like Context will always be used with the same 2 types.
pub type DynError = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, DynError>;
pub type FrameworkError<'a> = poise::FrameworkError<'a, Data, DynError>;

async fn on_error(err: crate::FrameworkError<'_>) {
    // Very fun way to deal wtih errors
    let error_message = match &err {
        // IF THERE'S A BETTER WAY TO DO THIS PLEASE TELL ME THIS LOOKS TERRIBLE
        poise::FrameworkError::Command  {error, ..}
        | poise::FrameworkError::ArgumentParse {error, ..} => error.to_string(),
        poise::FrameworkError::MissingUserPermissions { missing_permissions, .. } => {
            format!(
                "You do not have the permission(s){} to run this command!",
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

    // Just sends an embed for the error instead of the message it's supposed to send
    match err.ctx() {
        Some(ctx) => { 
            ctx.send(|m| m
                .embed(|e| e
                    .color(colors::RED)
                        .field("Error!", error_message, false)
                )
            ).await.expect("Failed to send the error message!");
        },
        None => println!("{}", error_message),
    };
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Loads the config.json file for global settings (not server-specific)
    let raw_config = fs::read_to_string("./config.json")?;
    let config: Config = serde_json::from_str(&raw_config)?;

    // Loads up a connection to the SQLite pool.
    // This is one of the most crucial parts of the bot because this is necessary for persistent information.
    let database = sqlite::SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(
            sqlite::SqliteConnectOptions::new()
                .filename(&config.database)
                .create_if_missing(true)
        )
        .await
        .expect("Couldn't connect to the database!");

    // Migrations are just initial table declerations. You can find them in /migrations/.
    // I believe that this is specifically for binary versions of the software, but I'm not 100% sure.
    sqlx::migrate!("./../migrations").run(&database).await.expect("Couldn't run database migrations!");

    // Used in the info command to get the bot uptime. Declared here so that the timer starts ticking as the bot starts up
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

                    // We make a new thread to deal with timed moderations so that it can run async to the rest of the bot
                    tokio::spawn(async move {
                        loop {
                            // If the bot is lagging, try using a longer interval.
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                
                            // We sort the entires by using the current Unix time, and finding any moderation
                            // that has an expiry date less than the current Unix time (or earlier in time).
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

    // Sharding is used in larger instances to help spread out event handling.
    // It is also necessary when your bot is in over 2500 guilds.
    // If you are self-hosting the bot, this will likely always be only one shard.
    framework.run_autosharded().await?;

    Ok(())
}
