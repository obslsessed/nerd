// these are commands/features that i want to make/use in the future

use anyhow::Result;

use nerd::{Context, Error};

// rest means use the entire rest of the message as this parameter
#[poise::command(prefix_command, slash_command)]
pub async fn say(
    ctx: Context<'_>,
    #[rest]
    #[description = "Text to say"]
    msg: String,
) -> Result<(), Error> {
    ctx.say(msg).await?;
    Ok(())
}

//paginate for retrying messages?
#[poise::command(slash_command, prefix_command)]
pub async fn paginate(ctx: Context<'_>) -> Result<(), Error> {
    let pages = &[
        "Content of first page",
        "Content of second page",
        "Content of third page",
        "Content of fourth page",
    ];

    poise::samples::paginate(ctx, pages).await?;

    Ok(())
}
