use std::sync::Arc;

use poise::serenity_prelude;
use serenity_prelude::{GuildId, UserId, Timestamp};

use crate::commands::moderation::types::ModerationType;

// Bans, Mutes, and Timeouts should only occur once per guild per member
// This is to prevent double expiries, which could cause unexpected unban times
pub async fn clean_double_expiries(
    database: &sqlx::SqlitePool,
    guild_id: i64, 
    user_id: i64, 
    moderation_type: ModerationType,
) -> sqlx::Result<()> {
    let moderation_type_u8 = moderation_type as u8;

    if let ModerationType::Ban | ModerationType::Mute | ModerationType::Timeout = moderation_type {
        sqlx::query!(
            "UPDATE moderations SET active = FALSE WHERE guild_id = ? AND user_id = ? AND moderation_type = ?",
            guild_id,
            user_id,
            moderation_type_u8
        )
        .execute(&*database)
        .await?;
    }

    Ok(())
}

pub async fn add_moderation(
    data: &crate::Data,
    guild_id: impl Into<GuildId>, 
    user_id: impl Into<UserId>, 
    moderation_type: ModerationType,
    administered_at: Timestamp,
    expiry_date: Option<Timestamp>,
    reason: Option<&str>,
) -> sqlx::Result<()> {
    let database = &data.database;

    let guild_id = guild_id.into().0 as i64;
    let user_id = user_id.into().0 as i64;
    let moderation_type_u8 = moderation_type as u8;
    let expiry_date = expiry_date.map(|date| date.unix_timestamp());
    let administered_at = administered_at.unix_timestamp();

    clean_double_expiries(&database, guild_id, user_id, moderation_type).await?;

    sqlx::query!(
        "INSERT INTO moderations (guild_id, user_id, moderation_type, administered_at, expiry_date, reason, active) VALUES (?, ?, ?, ?, ?, ?, ?)",
        guild_id,
        user_id,
        moderation_type_u8,
        administered_at,
        expiry_date,
        reason,
        true
    )
    .execute(database)
    .await?;

    Ok(())
}