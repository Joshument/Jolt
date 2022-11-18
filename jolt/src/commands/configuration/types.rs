/// An error representing a timed out response.
/// This is used when the bot is expecting a response, but the user does not give one in time.
///
/// Value in the struct represents how long the response was left hanging for.
#[derive(Debug, Clone)]
pub struct ResponseTimedOut(pub std::time::Duration);

impl std::error::Error for ResponseTimedOut {}

impl std::fmt::Display for ResponseTimedOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Did not receieve a response in {} seconds, cancelling operation.",
            self.0.as_secs()
        )
    }
}
