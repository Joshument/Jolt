use crate::commands::configuration::error as configuration_error;
use crate::commands::moderation::error as moderation_error;

pub use configuration_error::SetupError;
pub use moderation_error::ModerationError;
pub use poise::serenity_prelude::SerenityError;
pub use sqlx::error::Error as SqlxError;
pub use poise::FrameworkError;

/// General error struct for all variants in the program
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error passed when an Integer is out of bounds when converting to an Enum.
    #[error("Failed to convert integer into enum type")]
    IntEnumError,
    /// An error representing a missing config.
    /// Contains one value, which is the config option that is not set.
    #[error("Config `{0}` is not set!")]
    ConfigNotSetError(String),
    /// An error representing an out of bounds page.
    /// First value is the requested page, second value is the maximum page.
    #[error("Attempted to access page {0} when maximum page is {1}")]
    PageOutOfBounds(usize, usize),
    /// Errors relating to the `serenity` crate (re-exported by poise).
    #[error(transparent)]
    SerenityError(#[from] SerenityError),
    /// Errors relating to the `sqlx` crate.
    #[error(transparent)]
    SqlxError(#[from] SqlxError),
    /// Errors related to setup related problems.
    #[error("transparent")]
    SetupError(#[from] SetupError),
    /// Errors related to moderation command problems
    #[error(transparent)]
    ModerationError(#[from] ModerationError),
}