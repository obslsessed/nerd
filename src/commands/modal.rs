use nerd2::{ApplicationContext, Error, MyModal};
use poise::Modal;

/// create a character
#[poise::command(slash_command)]
pub async fn modal(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = MyModal::execute(ctx).await?;
    println!("Got data: {:?}", data);

    Ok(())
}
