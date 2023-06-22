use std::{
    fs::{create_dir, read_dir},
    io::ErrorKind,
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

pub async fn send_chat(chat: CreateChatCompletionRequest) -> Result<String> {
    let client = Client::new();
    let response = client.chat().create(chat).await?;
    let content = response.choices.last().unwrap().message.content.clone(); // TODO: fix this mess
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
