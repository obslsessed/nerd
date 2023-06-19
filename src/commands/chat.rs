use anyhow::Result;
use nerd2::CHARACTERS_PATH;
use nerd2::{get_characters, send_chat};
use nerd2::{Character, Context, Error};
use poise::serenity_prelude::{self as serenity, MessageId};
use std::fs::read_to_string;

/// chat
#[poise::command(slash_command, prefix_command)]
pub async fn chat(ctx: Context<'_>) -> Result<(), Error> {
    let channel = ctx.channel_id();
    let (character, message_id) = choose_character(ctx).await?;
    let thread = channel
        .create_public_thread(ctx, message_id, |thread| thread.name(&character.name))
        .await?;
    let reply = thread.await_reply(ctx).await.unwrap(); // TODO: make it not unwrap
    let text = &reply.content;
    dbg!(&reply);
    let answer = send_chat(text).await?;
    thread.say(ctx, answer).await?;
    Ok(())
}

async fn choose_character(ctx: Context<'_>) -> Result<(Character, MessageId), Error> {
    let characters = get_characters()?;
    let handle = ctx
        .send(|f| {
            f.components(|f| {
                f.create_action_row(|f| {
                    f.create_select_menu(|f| {
                        f.custom_id("hi").options(|f| {
                            for character in characters {
                                match character.emoji {
                                    Some(emoji) => f.create_option(|f| {
                                        f.value(&character.name)
                                            .label(&character.prompt.unwrap_or("no prompt".into()))
                                            .emoji(emoji)
                                    }),
                                    None => f.create_option(|f| {
                                        f.value(&character.name)
                                            .label(&character.prompt.unwrap_or("no prompt".into()))
                                    }),
                                };
                            }
                            f
                        })
                    })
                })
            })
        })
        .await?;
    let message = handle.message().await?;
    let interaction = message.await_component_interaction(ctx).await.unwrap(); // TODO: make it not unwrap
    let name = &interaction.data.values[0];
    interaction
        .create_interaction_response(ctx, |f| {
            f.kind(serenity::InteractionResponseType::UpdateMessage)
                .interaction_response_data(|f| f.content(name).components(|f| f))
        })
        .await?;
    let path = format!("{CHARACTERS_PATH}/{name}");
    let string = read_to_string(path)?;
    let character = serde_json::from_str::<Character>(&string)?;
    let message_id = message.id;
    Ok::<(Character, MessageId), Error>((character, message_id))
}
