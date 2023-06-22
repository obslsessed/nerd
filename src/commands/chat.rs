use nerd::{update_webhook_channel_id, Character, Conversation};
use nerd::{Context, Error};
use nerd::{CHARACTERS_PATH, CHAT_MODEL, CONVERSATIONS_PATH};

use anyhow::Result;
use async_openai::types::{
    ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role,
};

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, GuildChannel, MessageId};

use std::fs::{read_dir, read_to_string, File};
use std::io::{Read, Write};

/// chat
#[poise::command(slash_command, prefix_command)]
pub async fn chat(ctx: Context<'_>) -> Result<(), Error> {
    let (character, message_id) = choose_character(ctx).await?;
    let thread = create_thread(&ctx, &message_id, &character.name).await?;
    new_chat(&ctx, character, thread).await?;
    Ok(())
}

pub async fn new_chat(
    ctx: &Context<'_>,
    character: Character,
    id: impl Into<ChannelId>,
) -> Result<()> {
    let id = id.into();
    let mut chat = CreateChatCompletionRequestArgs::default()
        .model(CHAT_MODEL)
        .build()?;

    if let Some(description) = &character.description {
        let description = ChatCompletionRequestMessageArgs::default()
            .role(Role::System)
            .content(description)
            .build()?;
        chat.messages.push(description);
    }

    if let Some(greeting) = &character.greeting {
        let webhook = update_webhook_channel_id(ctx.serenity_context(), &character, id).await?;
        webhook
            .execute(ctx, false, |w| w.content(&greeting))
            .await?;
        let greeting = ChatCompletionRequestMessageArgs::default()
            .role(Role::System)
            .content(greeting)
            .build()?;
        chat.messages.push(greeting);
    }

    let conversation = Conversation { character, chat };

    let path = format!("{CONVERSATIONS_PATH}/{id}");
    let mut file = File::create(path)?;

    let json = serde_json::to_string(&conversation)?;
    let bytes = json.as_bytes();
    file.write(bytes)?;
    Ok(())
}

async fn create_thread(
    ctx: &Context<'_>,
    message_id: impl Into<MessageId>,
    name: &str,
) -> Result<GuildChannel> {
    let channel = ctx.channel_id();
    let thread = channel
        .create_public_thread(ctx, message_id, |thread| thread.name(name))
        .await?;
    Ok(thread)
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
                                let mut greeting =
                                    character.greeting.unwrap_or("no greeting".into());
                                greeting.truncate(100);
                                match character.emoji {
                                    Some(emoji) => f.create_option(|f| {
                                        f.value(&character.name).label(greeting).emoji(emoji)
                                    }),
                                    None => f.create_option(|f| {
                                        f.value(&character.name).label(greeting)
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
    let character = serde_json::from_str(&string)?;
    let message_id = message.id;
    Ok((character, message_id))
}

fn get_characters() -> Result<Vec<Character>> {
    let character_names = read_dir(CHARACTERS_PATH)?;
    let characters = character_names
        .map(|entry| {
            let path = entry.unwrap().path();
            let mut file = File::open(path).unwrap();
            let mut string = String::new();
            file.read_to_string(&mut string).unwrap();
            let character = serde_json::from_str::<Character>(&string).unwrap();
            character
        })
        .collect::<Vec<Character>>();
    Ok(characters)
}
