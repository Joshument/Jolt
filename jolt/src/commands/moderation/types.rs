use poise::serenity_prelude;

/// An error passed when an Integer is out of bounds when converting to an Enum
#[derive(Debug, Clone)]
pub struct IntEnumError;

impl std::error::Error for IntEnumError {}

impl std::fmt::Display for IntEnumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to convert integer into enum type")
    }
}

/// An error representing a missing config.
/// Contains one value, which is the config option that is not set.
#[derive(Debug, Clone)]
pub struct ConfigNotSetError(pub String);

impl std::error::Error for ConfigNotSetError {}

impl std::fmt::Display for ConfigNotSetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "config `{}` is not set!", self.0)
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)] // some values are going to be used later, no need to have useless warnings
pub enum ModerationType {
    Warning = 0,
    Kick = 1,
    Mute = 2,
    Timeout = 3,
    Ban = 4,
}

impl TryFrom<u8> for ModerationType {
    type Error = IntEnumError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ModerationType::Warning),
            1 => Ok(ModerationType::Kick),
            2 => Ok(ModerationType::Mute),
            3 => Ok(ModerationType::Timeout),
            4 => Ok(ModerationType::Ban),
            _ => Err(IntEnumError)
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
            ModerationType::Ban => "Ban"
        };
        write!(f, "{}", moderation_string)
    }
}

/// General information about an entry
pub struct ModlogEntry {
    pub id: u64,
    pub moderation_type: ModerationType,
    pub administered_at: serenity_prelude::Timestamp,
    pub expiry_date: Option<serenity_prelude::Timestamp>,
    pub reason: Option<String>,
    pub active: bool,
}

/// An error representing an out of bounds page.
/// First value is the requested page, second value is the maximum page.
#[derive(Debug, Clone)]
pub struct PageOutOfBounds(pub u64, pub u64);

impl std::error::Error for PageOutOfBounds {}

impl std::fmt::Display for PageOutOfBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "attempted to access page {} when maximum page is {}", self.0, self.1)
    }
}