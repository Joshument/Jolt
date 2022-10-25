use std::sync::Arc;

use serenity::prelude::*;
use serenity::model::prelude::*;

use crate::commands::moderation::ModerationType;

pub struct Database;

impl TypeMapKey for Database {
    type Value = Arc<sqlx::SqlitePool>;
}

pub async fn get_database(data: &Arc<RwLock<TypeMap>>) -> Arc<sqlx::SqlitePool> {
    let data = data.read().await;
    data.get::<Database>().expect("Failed to get database!").clone()
}

pub async fn add_temporary_moderation(
    data: &Arc<RwLock<TypeMap>>,
    guild_id: impl Into<GuildId>, 
    user_id: impl Into<UserId>, 
    moderation_type: ModerationType,
    expiry_date: Timestamp,
    reason: &str,
) -> sqlx::Result<()> {
    let database = get_database(data).await.clone();

    let guild_id = guild_id.into().0 as i64;
    let user_id = user_id.into().0 as i64;
    let moderation_type = moderation_type as u8;
    let expiry_date = expiry_date.unix_timestamp();

    sqlx::query!(
        "INSERT INTO timed_moderations (guild_id, user_id, moderation_type, expiry_date, reason) VALUES (?, ?, ?, ?, ?)",
        guild_id,
        user_id,
        moderation_type,
        expiry_date,
        reason
    )
    .execute(&*database)
    .await?;

    Ok(())
}