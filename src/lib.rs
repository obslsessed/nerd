use std::{
    fs::{create_dir, read_dir, read_to_string, write},
    io::ErrorKind,
    str::FromStr,
};

use anyhow::Result;
use async_openai::{
    types::{
        ChatChoice, ChatCompletionRequestMessageArgs, ChatCompletionResponseMessage,
        CreateChatCompletionRequest, Role,
    },
    Client,
};
use poise::serenity_prelude::json::json;
use poise::serenity_prelude::{ChannelId, Message, ReactionType};
use poise::serenity_prelude::{Context as SerenityContext, Webhook};
use poise::Modal;

use serde::{Deserialize, Serialize};

pub const TEST_SERVER_ID: u64 = 1113998071194456195;
pub const BRAZIL_SERVER_ID: u64 = 849378682741194752;
pub const NERD_BOT_ID: u64 = 1118700646791647262;
pub const RYY_BOT_ID: u64 = 672957277032153108;
pub const OWNER_USER_ID: u64 = 284834416633577472;
pub const CHAT_MODEL: &str = "gpt-3.5-turbo-0613";
pub const DATABASE_PATH: &str = "nerd";
pub const CHARACTERS_PATH: &str = "nerd/characters";
pub const CONVERSATIONS_PATH: &str = "nerd/conversations";

pub type ApplicationContext<'a> = poise::ApplicationContext<'a, (), Error>;
pub type Context<'a> = poise::Context<'a, (), Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub struct Data {} // User data, which is stored and accessible in all command invocations

#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub character: Character,
    pub chat: CreateChatCompletionRequest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub webhook: Option<Webhook>,
    pub emoji: Option<ReactionType>,
    pub description: Option<String>,
    pub greeting: Option<String>,
    pub examples: Option<Vec<String>>,
}

#[derive(Debug, Modal)]
#[name = "character creator"]
pub struct MyModal {
    #[name = "name"]
    #[placeholder = "the character's name"]
    pub name: String,
    #[name = "description"]
    #[placeholder = "the character's description"]
    #[paragraph]
    pub description: Option<String>,
    #[name = "greeting"]
    #[placeholder = "the character's greeting"]
    #[paragraph]
    pub greeting: Option<String>,
}

pub fn create_directories() -> Result<()> {
    let paths = vec![DATABASE_PATH, CHARACTERS_PATH, CONVERSATIONS_PATH];
    for path in paths {
        if let Err(error) = create_dir(path) {
            if error.kind() != ErrorKind::AlreadyExists {
                return Err(error.into());
            }
        };
    }
    Ok(())
}

pub async fn remember_and_add_to_chat(ctx: &SerenityContext, message: &Message) -> Result<()> {
    let channel = message.channel_id;
    let path = format!("{CONVERSATIONS_PATH}/{channel}");
    let string = read_to_string(&path)?;
    let mut conversation = serde_json::from_str::<Conversation>(&string)?;
    let input = ChatCompletionRequestMessageArgs::default()
        .role(Role::User)
        .content(&message.content)
        .build()?;
    conversation.chat.messages.push(input);
    let response = send_chat(conversation.chat.clone()).await?;
    let output = ChatCompletionRequestMessageArgs::default()
        .role(Role::User)
        .content(&response)
        .build()?;
    conversation.chat.messages.push(output);
    let json = serde_json::to_string(&conversation)?;
    write(&path, json)?;

    let webhook =
        update_webhook_channel_id(&ctx, &conversation.character, message.channel_id).await?;
    webhook.execute(ctx, false, |w| w.content(response)).await?;
    Ok(())
}

pub async fn update_webhook_channel_id(
    ctx: &SerenityContext,
    character: &Character,
    id: impl Into<ChannelId>,
) -> Result<Webhook> {
    let id = id.into();
    let webhook = character.webhook.clone().unwrap();
    let webhook_id = webhook.id.into();
    let token = &webhook.token.unwrap();
    let value = json!({
        "channel_id": id,
    });
    let map = value.as_object().unwrap();
    let webhook = ctx
        .http
        .edit_webhook_with_token(webhook_id, token, &map)
        .await?;
    dbg!(&webhook);
    Ok(webhook)
}

pub async fn send_chat(chat: CreateChatCompletionRequest) -> Result<String> {
    let error_text = "NEJ!!!!! DET HÄNDE NÅGONTING FEL!!!!!!!!! `response.choices.pop()` gav None i `send_chat()`".into();
    let error_message = ChatCompletionResponseMessage {
        role: async_openai::types::Role::Assistant,
        content: error_text,
    };
    let error_response = ChatChoice {
        message: error_message,
        index: 0,
        finish_reason: None,
    };

    let client = Client::new();
    let mut response = client.chat().create(chat).await?;
    let last = response.choices.pop().unwrap_or_else(|| {
        dbg!(&response);
        error_response
    });
    let content = last.message.content;
    Ok(content)
}

pub fn get_thread_ids() -> Result<Vec<ChannelId>> {
    // todo: fix  this entire function
    let threads = read_dir(CONVERSATIONS_PATH)?;
    let thread_ids = threads
        .map(|entry| {
            let os_string = entry.unwrap().file_name();
            let str = os_string.to_str().unwrap();
            let channel_id = ChannelId::from_str(str).unwrap();
            channel_id
        })
        .collect::<Vec<ChannelId>>();
    Ok(thread_ids)
}
