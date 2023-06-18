use anyhow::Result;
use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use poise::Modal;

pub type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, Error>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub struct Data {} // User data, which is stored and accessible in all command invocations

#[derive(Debug, Modal)]
#[name = "Modal title"] // Struct name by default
pub struct MyModal {
    #[name = "First input label"] // Field name by default
    #[placeholder = "Your first input goes here"] // No placeholder by default
    #[min_length = 5] // No length restriction by default (so, 1-4000 chars)
    #[max_length = 500]
    first_input: String,
    #[name = "Second input label"]
    #[paragraph] // Switches from single-line input to multiline text box
    second_input: Option<String>, // Option means optional input
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
