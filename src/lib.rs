use anyhow::Result;
use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use poise::{serenity_prelude::ReactionType, Modal};

pub const TEST_SERVER_ID: u64 = 1113998071194456195;
pub const DATABASE_PATH: &str = "nerd/";

pub type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, Error>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub struct Data {} // User data, which is stored and accessible in all command invocations

#[derive(Debug)]
pub struct Character {
    pub name: String,
    pub prompt: Option<String>,
    pub emoji: Option<ReactionType>,
}

#[derive(Debug, Modal)]
#[name = "character creator"] // Struct name by default
pub struct MyModal {
    #[name = "name"] // Field name by default
    #[placeholder = "the character's name"] // No placeholder by default
    pub name: String,
    #[name = "prompt"]
    #[placeholder = "the character's prompt"]
    #[paragraph] // Switches from single-line input to multiline text box
    pub prompt: Option<String>, // Option means optional input
}

pub async fn send_chat(input: &str) -> Result<String> {
    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo-0613")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("Answer in only 1 word.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(input)
                .build()?,
        ])
        .build()?;
    dbg!(&request);
    let response = client.chat().create(request).await?;
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
