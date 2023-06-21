use anyhow::Result;
use async_openai::types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequest, Role};
use nerd2::{get_characters, new_chat, send_chat};
use nerd2::{Character, Context, Error};
use nerd2::{CHARACTERS_PATH, CONVERSATIONS_PATH};
use poise::serenity_prelude::{self as serenity, CacheHttp, ChannelId, GuildChannel, MessageId};
use std::fs::{read_to_string, write};

/// chat
#[poise::command(slash_command, prefix_command)]
pub async fn chat(ctx: Context<'_>) -> Result<(), Error> {
    let (character, message_id) = choose_character(ctx).await?;
    let thread = create_thread(&ctx, &message_id, &character.name).await?;
    let thread_id = thread.id;
    new_chat(&character, thread_id)?;
    let reply = thread.await_reply(ctx).await.unwrap(); // TODO: make it not unwrap
    let reply_text = &reply.content;
    dbg!(&reply);
    handle_chat(ctx, reply_text, thread, character).await?;

    Ok(())
}

async fn handle_chat(
    http: impl CacheHttp,
    input: &str,
    thread: impl Into<ChannelId>,
    character: Character,
) -> Result<()> {
    let id = thread.into();
    let ctx = http.http();

    let path = format!("{CONVERSATIONS_PATH}/{id}");
    let string = read_to_string(&path)?;
    let mut chat = serde_json::from_str::<CreateChatCompletionRequest>(&string)?;
    let input = ChatCompletionRequestMessageArgs::default()
        .role(Role::User)
        .content(input)
        .build()?;
    chat.messages.push(input);
    dbg!(&chat);
    let answer = send_chat(chat.clone(), character).await?;
    id.say(ctx, &answer).await?;
    let output = ChatCompletionRequestMessageArgs::default()
        .role(Role::Assistant)
        .content(answer)
        .build()?;
    chat.messages.push(output);
    let output_json = serde_json::to_string(&chat)?;
    write(&path, output_json)?;
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
    let character = serde_json::from_str(&string)?;
    let message_id = message.id;
    Ok((character, message_id))
}
