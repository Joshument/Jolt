use poise::serenity_prelude;

/// Errors relating to moderation commands.
#[derive(thiserror::Error, Debug)]
pub enum ModerationError {
    /// An error representing an invalid modlog request.
    /// Contains two values, which is the id of the modlog, and guild it was meant to belong to.
    #[error("Modlog {0} does not belong to guild **{}**", .1.name)]
    ModlogNotInGuild(u64, serenity_prelude::Guild),
    /// An error representing an invalid modlog request.
    /// This warning is to specify that the modlog is not a warning.
    /// Contains one value, which is the ID of the modlog
    #[error("Modlog {0} is not a warning!")]
    NotAWarning(u64),
    /// An error representing that the member being moderated is a moderator.
    /// Contains one vaue, which is the member that is the moderator.
    #[error("User <@{0}> is a moderator!")]
    MemberIsModerator(serenity_prelude::Member),
}