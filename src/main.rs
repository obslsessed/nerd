mod commands;

use crate::commands::chat::chat;
use crate::commands::modal::create;

use nerd2::Data;
use nerd2::TEST_SERVER_ID;
use poise::serenity_prelude as serenity;

// TODO: webhooks are fake users
// TODO: modals for creating/editing characters?

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let options = poise::FrameworkOptions {
        commands: vec![chat(), create()],
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!("Got an event in event handler: {:?}", event.name());
                Ok(())
            })
        },
        ..Default::default()
    };
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let framework = poise::Framework::builder()
        .options(options)
        .token(token)
        .intents(intents)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // don't need to register globally, but maybe eventually lol
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    poise::serenity_prelude::GuildId(TEST_SERVER_ID),
                )
                .await?;
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
