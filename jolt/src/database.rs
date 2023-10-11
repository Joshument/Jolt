/*
This file provides a lot of syntatic sugar around database access to make it easier to work with the SQLite database.
In all technicality, you don't even need to know SQL if you don't intend to touch this file. You're welcome
*/

use poise::serenity_prelude::{self, ChannelId};
use serenity_prelude::{GuildId, RoleId, Timestamp, UserId};

use crate::commands::moderation::types::{ModerationType, ModlogEntry};
use crate::error::Error;

/// Sets all existing moderations of the type `ModerationType` to inactive.
pub async fn clear_moderations(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId> + Clone,
    user_id: impl Into<UserId> + Clone,
    moderation_type: ModerationType,
) -> sqlx::Result<()> {
    let moderation_type_u8 = moderation_type as u8;
    let guild_id = guild_id.into().0 as i64;
    let user_id = user_id.into().0 as i64;

    if let ModerationType::Ban | ModerationType::Mute | ModerationType::Timeout = moderation_type {
        sqlx::query!(
            "UPDATE moderations SET active = FALSE WHERE guild_id = ? AND user_id = ? AND moderation_type = ?",
            guild_id,
            user_id,
            moderation_type_u8
        )
        .execute(database)
        .await?;
    }

    Ok(())
}

/// Clear a specific function using the ID of the moderation.
/// Mostly used to get rid of warnings.
///
/// **Make sure you are not letting users remove moderations from other guilds!**
pub async fn clear_single_moderation(database: &sqlx::SqlitePool, id: u64) -> sqlx::Result<()> {
    let id = id as i64;

    sqlx::query!("UPDATE moderations SET active = FALSE WHERE id = ?", id)
        .execute(database)
        .await?;

    Ok(())
}

pub async fn add_moderation(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId> + Clone,
    user_id: impl Into<UserId> + Clone,
    moderator_id: impl Into<UserId>,
    moderation_type: ModerationType,
    administered_at: Timestamp,
    expiry_date: Option<Timestamp>,
    reason: Option<&str>,
) -> sqlx::Result<()> {
    let guild_id_i64 = guild_id.clone().into().0 as i64;
    let user_id_i64 = user_id.clone().into().0 as i64;
    let moderator_id_64 = moderator_id.into().0 as i64;
    let moderation_type_u8 = moderation_type as u8;
    let expiry_date = expiry_date.map(|date| date.unix_timestamp());
    let administered_at = administered_at.unix_timestamp();

    let active = match moderation_type {
        ModerationType::Kick
        | ModerationType::Unban
        | ModerationType::Unmute
        | ModerationType::Untimeout => false,
        _ => true,
    };

    // Bans, Mutes, and Timeouts should only occur once per guild per member
    // This is to prevent double expiries, which could cause unexpected unban times
    if let ModerationType::Ban | ModerationType::Mute | ModerationType::Timeout = moderation_type {
        clear_moderations(
            &database,
            guild_id.clone(),
            user_id.clone(),
            moderation_type,
        )
        .await?;
    }

    // Get the highest moderation id to increment for new id
    let id: i64 = sqlx::query!(
        "SELECT MAX(id) AS max_id FROM moderations WHERE guild_id = ?",
        guild_id_i64,
    )
        .fetch_one(database)
        .await?.max_id.unwrap_or(0) + 1;

    sqlx::query!(
        "INSERT INTO moderations \
        (guild_id, id, user_id, moderator_id, moderation_type, administered_at, expiry_date, reason, active) \
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        guild_id_i64,
        id,
        user_id_i64,
        moderator_id_64,
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
        None => Ok(None),
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
    .execute(database)
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
    .fetch_optional(database)
    .await?;

    Ok(entry.and_then(|some| {
        some.logs_channel_id
            .map(|unwrapped| ChannelId(unwrapped as u64))
    }))
}

pub async fn get_prefix(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId>,
) -> Result<Option<String>, sqlx::Error> {
    let guild_id_i64 = guild_id.into().0 as i64;

    let entry = sqlx::query!(
        "SELECT prefix FROM guild_settings WHERE guild_id=?",
        guild_id_i64
    )
    .fetch_optional(database)
    .await?;

    Ok(entry.and_then(|some| some.prefix))
}

pub async fn set_prefix(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId>,
    prefix: &str,
) -> sqlx::Result<()> {
    let guild_id_i64 = guild_id.into().0 as i64;

    sqlx::query!(
        "INSERT INTO guild_settings (guild_id, prefix) VALUES ($1, $2)
        ON CONFLICT (guild_id) DO UPDATE SET prefix=excluded.prefix",
        guild_id_i64,
        prefix
    )
    .execute(database)
    .await?;

    Ok(())
}

pub async fn get_modlog_count(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId>,
    user_id: impl Into<UserId>,
) -> Result<usize, sqlx::Error> {
    let guild_id_i64 = guild_id.into().0 as i64;
    let user_id_i64 = user_id.into().0 as i64;

    let query_count = sqlx::query!(
        "SELECT * FROM moderations WHERE guild_id=? AND user_id=?",
        guild_id_i64,
        user_id_i64
    )
    .fetch_all(database)
    .await?
    .len();

    Ok(query_count)
}

pub async fn get_modlog_page(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId> + Copy,
    user_id: impl Into<UserId> + Copy,
    page: usize,
    page_size: usize,
) -> Result<Vec<ModlogEntry>, Error> {
    let guild_id_i64 = guild_id.into().0 as i64;
    let user_id_i64 = user_id.into().0 as i64;
    let query_count = get_modlog_count(database, guild_id, user_id).await?;
    let page_size_i64 = page_size as i64;
    // I'm such a mathematician
    let offset: i64 = (page * page_size - page_size)
        .try_into()
        .expect("how the hell did you go over the 64-bit integer limit in modlogs");

    // stay confused because I don't remember at a glance either
    if query_count < page * page_size - page_size {
        return Err(Error::PageOutOfBounds(page, page_size / 10 + 1));
    }

    let modlogs = sqlx::query_as(
        &format!(
            "SELECT * FROM moderations WHERE guild_id={} AND user_id={} ORDER BY id DESC LIMIT {} OFFSET {}",
            guild_id_i64,
            user_id_i64,
            page_size_i64,
            offset
        )
    )
    .fetch_all(database)
    .await?;

    Ok(modlogs)
}

pub async fn get_warning_count(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId>,
    user_id: impl Into<UserId>,
) -> Result<usize, sqlx::Error> {
    let guild_id_i64 = guild_id.into().0 as i64;
    let user_id_i64 = user_id.into().0 as i64;

    let query_count = sqlx::query!(
        "SELECT * FROM moderations WHERE guild_id=? AND user_id=? AND moderation_type=? AND active=TRUE",
        guild_id_i64,
        user_id_i64,
        ModerationType::Warning as u8
    )
        .fetch_all(database)
        .await?
        .len();

    Ok(query_count)
}

pub async fn get_warning_page(
    database: &sqlx::SqlitePool,
    guild_id: impl Into<GuildId> + Copy,
    user_id: impl Into<UserId> + Copy,
    page: usize,
    page_size: usize,
) -> Result<Vec<ModlogEntry>, Error> {
    let guild_id_i64 = guild_id.into().0 as i64;
    let user_id_i64 = user_id.into().0 as i64;
    let query_count = get_warning_count(database, guild_id, user_id).await?;
    let page_size_i64 = page_size as i64;
    // I loooove math
    let offset: i64 = (page * page_size - page_size)
        .try_into()
        .expect("how the hell did you get over the 64-bit integer limit in warnings");

    // It works so who cares about making it readable !!!
    if query_count < page * page_size - page_size {
        return Err(Error::PageOutOfBounds(page, page_size / 10 + 1));
    }

    let modlogs = sqlx::query_as(
        &format!(
            "SELECT * FROM moderations WHERE guild_id={} AND user_id={} AND moderation_type={} AND active=TRUE ORDER BY id DESC LIMIT {} OFFSET {}",
            guild_id_i64,
            user_id_i64,
            ModerationType::Warning as u8,
            page_size_i64,
            offset
        )
    )
    .fetch_all(database)
    .await?;

    Ok(modlogs)
}

pub async fn get_single_modlog(database: &sqlx::SqlitePool, id: u64) -> sqlx::Result<ModlogEntry> {
    let id = id as i64;

    let modlog = sqlx::query!("SELECT * FROM moderations WHERE id=?", id)
        .fetch_one(database)
        .await?;
    println!("{:?}", modlog.expiry_date);
    Ok(ModlogEntry {
        id: modlog.id as u64,
        guild_id: GuildId(modlog.guild_id as u64),
        moderation_type: (modlog.moderation_type as u8).try_into().unwrap(),
        user_id: UserId(modlog.user_id as u64),
        moderator_id: UserId(modlog.moderator_id as u64),
        administered_at: serenity_prelude::Timestamp::from_unix_timestamp(modlog.administered_at)
            .unwrap(),
        expiry_date: modlog
            .expiry_date
            .map(|some| serenity_prelude::Timestamp::from_unix_timestamp(some).unwrap()),
        reason: modlog.reason.clone(),
        active: modlog.active,
    })
}
