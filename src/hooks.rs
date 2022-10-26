use crate::colors;

use serenity::framework::standard::{DispatchError, CommandResult};
use serenity::framework::standard::macros::hook;
use serenity::model::prelude::Message;
use serenity::prelude::*;
use serenity::http::error::Error as HttpError;
use serenity::framework::standard::ArgError;
use serenity::model::misc::UserIdParseError;

use humantime::DurationError;

pub fn parse_error_to_english<'a>(error: Box<dyn std::error::Error + Send + Sync>) -> Option<&'a str> {
    if let Some(serenity_error) = error.downcast_ref::<SerenityError>() {
        match serenity_error {
            SerenityError::Http(http_error) => {
                match http_error.as_ref() {
                    HttpError::UnsuccessfulRequest(error_response) => {
                        match error_response.error.code {
                            50033 => {
                                Some("That user does not exist!")
                            }
                            _ => None
                        }
                    }
                    _ => None
                }
            }
            _ => None
        }
    } else if let Some(_) = error.downcast_ref::<ArgError<UserIdParseError>>() {
        Some("That user does not exist!")
    } else if let Some(duration_error) = error.downcast_ref::<DurationError>() {
        match duration_error {
            _ => Some("Invalid time duration!")
        }
    } else {
        None
    }
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    } else {
        println!("{:?}", error)
    }
}

#[hook]
pub async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => {
            println!("Command '{}' returned error {:?}", command_name, why);

            // Message to send on failure
            let error_message = parse_error_to_english(why);

            if let Some(message) = error_message {
                msg.channel_id.send_message(&ctx.http, |m| {
                    m.add_embed(|e| {
                        e.color(colors::RED)
                            .description(message)
                    })
                }).await.expect("Failed to send error message to channel!");
            }
        }
    }

    ()
}