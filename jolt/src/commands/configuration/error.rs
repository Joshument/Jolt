/// Errors related to setup related problems
#[derive(thiserror::Error, Debug)]
pub enum SetupError {
    /// An error representing a timed out response.
    /// This is used when the bot is expecting a response, but the user does not give one in time.
    ///
    /// Value in the enum variant represents how long the response was left hanging for.
    #[error("Did not recieve a response in {} seconds, cancelling operation", .0.as_secs())]
    ResponseTimedOut(std::time::Duration),
    /// An error representing a cancelled operation.
    /// This is typically used when an action is cancelled by the user.
    #[error("Operation cancelled by user!")]
    OperationCancelled
}