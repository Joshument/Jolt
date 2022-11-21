use crate::error::Error;
use poise::serenity_prelude::{self, GuildId, Timestamp, UserId};
use sqlx::{sqlite::SqliteRow, FromRow, Row, Sqlite};

#[derive(Copy, Clone, PartialEq, PartialOrd)]
#[repr(u8)]
#[allow(dead_code)] // some values are going to be used later, no need to have useless warnings
pub enum ModerationType {
    Warning = 0,
    Kick = 1,
    Mute = 2,
    Timeout = 3,
    Ban = 4,
    Unmute = 5,
    Untimeout = 6,
    Unban = 7,
}

impl TryFrom<u8> for ModerationType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ModerationType::Warning),
            1 => Ok(ModerationType::Kick),
            2 => Ok(ModerationType::Mute),
            3 => Ok(ModerationType::Timeout),
            4 => Ok(ModerationType::Ban),
            5 => Ok(ModerationType::Unmute),
            6 => Ok(ModerationType::Untimeout),
            7 => Ok(ModerationType::Unban),
            _ => Err(Error::IntEnumError),
        }
    }
}

impl std::fmt::Display for ModerationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let moderation_string = match self {
            ModerationType::Warning => "Warning",
            ModerationType::Kick => "Kick",
            ModerationType::Mute => "Mute",
            ModerationType::Timeout => "Timeout",
            ModerationType::Ban => "Ban",
            ModerationType::Unmute => "Unmute",
            ModerationType::Untimeout => "Untimeout",
            ModerationType::Unban => "Unban",
        };
        write!(f, "{}", moderation_string)
    }
}

/// General information about an entry
// #[derive(sqlx::FromRow)]
pub struct ModlogEntry {
    // #[sqlx(try_from = "i64")]
    pub id: u64,
    // #[sqlx(try_from = "i64")]
    pub guild_id: GuildId,
    // #[sqlx(try_from = "u8")]
    pub moderation_type: ModerationType,
    // #[sqlx(try_from = "i64")]
    pub user_id: UserId,
    // #[sqlx(try_from = "i64")]
    pub moderator_id: UserId,
    pub administered_at: Timestamp,
    // #[sqlx(default)]
    pub expiry_date: Option<Timestamp>,
    // #[sqlx(default)]
    pub reason: Option<String>,
    pub active: bool,
}

impl FromRow<'_, SqliteRow> for ModlogEntry {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get::<i64, &str>("id")? as u64,
            guild_id: GuildId(row.try_get::<i64, &str>("guild_id")? as u64),
            moderation_type: (row.try_get::<i64, &str>("moderation_type")? as u8)
                .try_into()
                .unwrap(),
            user_id: UserId(row.try_get::<i64, &str>("user_id")? as u64),
            moderator_id: UserId(row.try_get::<i64, &str>("moderator_id")? as u64),
            administered_at: Timestamp::from_unix_timestamp(row.try_get("administered_at")?)
                .unwrap(),
            expiry_date: row
                .try_get("expiry_date")
                .ok()
                .map(|date| Timestamp::from_unix_timestamp(date).unwrap()),
            reason: row.try_get("reason").ok(),
            active: row.try_get("active")?,
        })
    }
}
