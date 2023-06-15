use anyhow::Result;
use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};

pub async fn send_chat(input: &str) -> Result<String> {
    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo-0613")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("Answer with only 5 words or less.")
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
