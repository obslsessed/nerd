use std::fs::File;
use std::io::Write;

use nerd2::CHARACTERS_PATH;
use nerd2::{
    create_character_from_modal, set_emoji_from_reaction, ApplicationContext, Error, MyModal,
};
use poise::Modal;

/// create a character
#[poise::command(slash_command)]
pub async fn create(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let modal = MyModal::execute(ctx).await?;
    let maybe_character = create_character_from_modal(modal);
    if let Some(mut character) = maybe_character {
        let emoji = set_emoji_from_reaction(ctx).await?;
        if let Some(new_emoji) = emoji {
            character.emoji = Some(new_emoji);
        }
        let path = format!("{}/{}", CHARACTERS_PATH, character.name);
        let mut file = File::create(path)?;
        let json = serde_json::to_string(&character)?;
        let bytes = json.as_bytes();
        file.write_all(bytes)?;
        ctx.say(json).await?;
    }
    Ok(())
}
