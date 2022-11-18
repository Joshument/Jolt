use crate::colors;

use poise::serenity_prelude;

pub async fn send_error(
    ctx: &crate::Context<'_>,
    error: &str
) -> Result<(), serenity_prelude::Error> {
    ctx.send(|m| {
        m.embed(|e| {
            e.color(colors::RED)
                .title("Error!")
                .description(error)
        })
    })
    .await?;

    Ok(())
}