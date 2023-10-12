use crate::colors;

use poise::{
    serenity_prelude::{self, CreateEmbed},
    CreateReply,
};

pub async fn send_error(
    ctx: &crate::Context<'_>,
    error: &str,
) -> Result<(), serenity_prelude::Error> {
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .color(colors::RED)
                .title("Error!")
                .description(error),
        ),
    )
    .await?;

    Ok(())
}
