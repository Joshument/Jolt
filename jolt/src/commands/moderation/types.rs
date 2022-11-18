use crate::error::Error;
use poise::serenity_prelude;

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
pub struct ModlogEntry {
    pub id: u64,
    pub guild_id: serenity_prelude::GuildId,
    pub moderation_type: ModerationType,
    pub user_id: serenity_prelude::UserId,
    pub moderator_id: serenity_prelude::UserId,
    pub administered_at: serenity_prelude::Timestamp,
    pub expiry_date: Option<serenity_prelude::Timestamp>,
    pub reason: Option<String>,
    pub active: bool,
}
