use nerd2::CHARACTERS_PATH;
use nerd2::{ApplicationContext, Error};
use nerd2::{Character, MyModal};
use poise::serenity_prelude::ReactionType;

use std::fs::File;
use std::io::Write;

use poise::Modal;

use anyhow::Result;

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

fn create_character_from_modal(modal: Option<MyModal>) -> Option<Character> {
    match modal {
        None => None,
        Some(data) => Some(Character {
            name: data.name,
            emoji: None,
            description: data.description,
            greeting: data.greeting,
            examples: None,
        }),
    }
}

async fn set_emoji_from_reaction(
    application_context: ApplicationContext<'_>,
) -> Result<Option<ReactionType>> {
    let serenity_context = application_context.serenity_context;
    let poise_context = poise::Context::Application(application_context);

    let cancel = ReactionType::Unicode("❌".into());

    let handle = application_context
        .say("react to this message with the character's emoji or x for none")
        .await?;
    let message = handle.message().await?;
    message.react(serenity_context, cancel.clone()).await?;
    let action = message.await_reaction(serenity_context).await.unwrap(); //TODO: make it not unwrap
    handle.delete(poise_context).await?;
    let reaction = &action.as_inner_ref().emoji;
    let no_emoji = reaction == &cancel;

    match no_emoji {
        true => Ok(None),
        false => Ok(Some(reaction.to_owned())),
    }
}
