use poise::serenity_prelude::{self, ChannelId};
use serenity_prelude::{GuildId, UserId, RoleId, Timestamp};

use crate::commands::moderation::types::ModerationType;

/// Sets all existing moderations of the type `moderation_type` to inactive.
/// This has to take `i64` for the guild_id and user_id as SQLite does not support unsigned 64-bit numbers.
pub async fn clear_moderations(
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
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId>, 
    user_id: impl Into<UserId>, 
    moderation_type: ModerationType,
    administered_at: Timestamp,
    expiry_date: Option<Timestamp>,
    reason: Option<&str>,
) -> sqlx::Result<()> {
    let guild_id = guild_id.into().0 as i64;
    let user_id = user_id.into().0 as i64;
    let moderation_type_u8 = moderation_type as u8;
    let expiry_date = expiry_date.map(|date| date.unix_timestamp());
    let administered_at = administered_at.unix_timestamp();

    let active = match moderation_type {
        ModerationType::Kick => false,
        _ => true
    };

    // Bans, Mutes, and Timeouts should only occur once per guild per member
    // This is to prevent double expiries, which could cause unexpected unban times
    if let ModerationType::Ban | ModerationType::Mute | ModerationType::Timeout = moderation_type {
        clear_moderations(&database, guild_id, user_id, moderation_type).await?;
    }

    sqlx::query!(
        "INSERT INTO moderations (guild_id, user_id, moderation_type, administered_at, expiry_date, reason, active) VALUES (?, ?, ?, ?, ?, ?, ?)",
        guild_id,
        user_id,
        moderation_type_u8,
        administered_at,
        expiry_date,
        reason,
        active
    )
    .execute(database)
    .await?;

    Ok(())
}

pub async fn set_mute_role(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId>,
    role_id: impl Into<RoleId>,
) -> sqlx::Result<()> {
    let guild_id_i64 = guild_id.into().0 as i64;
    let role_id_i64 = role_id.into().0 as i64;
    
    sqlx::query!(
        "INSERT INTO guild_settings (guild_id, mute_role_id) VALUES ($1, $2)
        ON CONFLICT (guild_id) DO UPDATE SET mute_role_id=excluded.mute_role_id",
        guild_id_i64,
        role_id_i64
    )
    .execute(&*database)
    .await?;

    Ok(())
}

pub async fn get_mute_role(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId>,
) -> sqlx::Result<Option<RoleId>> {
    let guild_id_i64 = guild_id.into().0 as i64;

    let entry = sqlx::query!(
        "SELECT mute_role_id FROM guild_settings WHERE guild_id=?",
        guild_id_i64
    )
        .fetch_one(database)
        .await;

    if let Err(_) = entry {
        return Ok(None);
    }

    let entry = entry?;
    
    match entry.mute_role_id {
        Some(role_id) => Ok(Some(RoleId(role_id as u64))),
        None => Ok(None)
    }
}

pub async fn set_logs_channel(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId>,
    channel_id: impl Into<ChannelId>,
) -> sqlx::Result<()> {
    let guild_id_i64 = guild_id.into().0 as i64;
    let channel_id_i64 = channel_id.into().0 as i64;
    
    sqlx::query!(
        "INSERT INTO guild_settings (guild_id, logs_channel_id) VALUES ($1, $2)
        ON CONFLICT (guild_id) DO UPDATE SET logs_channel_id=excluded.logs_channel_id",
        guild_id_i64,
        channel_id_i64
    )
    .execute(&*database)
    .await?;

    Ok(())
}

pub async fn get_logs_channel(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId>,
) -> sqlx::Result<Option<ChannelId>> {
    let guild_id_i64 = guild_id.into().0 as i64;

    let entry = sqlx::query!(
        "SELECT logs_channel_id FROM guild_settings WHERE guild_id=?",
        guild_id_i64
    )
        .fetch_one(database)
        .await;
    
    if let Err(_) = entry {
        return Ok(None);
    }

    let entry = entry?;
    
    match entry.logs_channel_id {
        Some(channel_id) => Ok(Some(ChannelId(channel_id as u64))),
        None => Ok(None)
    }
}