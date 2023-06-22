use std::{
    fs::{create_dir, read_dir, File},
    io::{ErrorKind, Read},
    str::FromStr,
};

use anyhow::Result;
use async_openai::{types::CreateChatCompletionRequest, Client};
use poise::{
    serenity_prelude::{ChannelId, ReactionType},
    Modal,
};
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
pub struct Character {
    pub name: String,
    pub prompt: Option<String>,
    pub emoji: Option<ReactionType>,
}

#[derive(Debug, Modal)]
#[name = "character creator"]
pub struct MyModal {
    #[name = "name"]
    #[placeholder = "the character's name"]
    pub name: String,
    #[name = "prompt"]
    #[placeholder = "the character's prompt"]
    #[paragraph]
    pub prompt: Option<String>,
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

pub async fn send_chat(chat: CreateChatCompletionRequest) -> Result<String> {
    let client = Client::new();
    let response = client.chat().create(chat).await?;
    dbg!(&response);
    let content = response.choices.last().unwrap().message.content.clone(); // TODO: fix this mess
    Ok(content)
}

pub fn create_character_from_modal(modal: Option<MyModal>) -> Option<Character> {
    match modal {
        None => None,
        Some(data) => Some(Character {
            name: data.name,
            prompt: data.prompt,
            emoji: None,
        }),
    }
}

pub async fn set_emoji_from_reaction(
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

pub fn get_characters() -> Result<Vec<Character>> {
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
