mod commands;

use commands::chat::chat;
use commands::create::create;

use anyhow::Result;
use commands::todo::paginate;
use commands::todo::say;
use nerd2::create_directories;
use nerd2::Error;
use nerd2::NERD_BOT_ID;
use nerd2::RYY_BOT_ID;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Context;
use poise::serenity_prelude::Interaction;
use poise::serenity_prelude::Reaction;

// TODO: webhooks are fake users
// TODO: modals for creating/editing characters?

struct Handler {
    options: poise::FrameworkOptions<(), Error>,
    shard_manager:
        std::sync::Mutex<Option<std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>>>>,
}
#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: serenity::Message) {
        // FrameworkContext contains all data that poise::Framework usually manages
        let shard_manager = (*self.shard_manager.lock().unwrap()).clone().unwrap();
        let framework_data = poise::FrameworkContext {
            bot_id: serenity::UserId(NERD_BOT_ID),
            options: &self.options,
            user_data: &(),
            shard_manager: &shard_manager,
        };

        poise::dispatch_event(framework_data, &ctx, &poise::Event::Message { new_message }).await;
    }

    async fn reaction_add(&self, ctx: Context, rct: Reaction) {
        dbg!(&rct);
        if rct.member.unwrap().user.unwrap().id == RYY_BOT_ID {
            let cid = rct.channel_id.as_u64().to_owned();
            let mid = rct.message_id.as_u64().to_owned();
            let rtype = &rct.emoji;
            if let Err(err) = ctx.http.create_reaction(cid, mid, rtype).await {
                println!("could not react with ryybot: {err}");
            };
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // FrameworkContext contains all data that poise::Framework usually manages
        let shard_manager = (*self.shard_manager.lock().unwrap()).clone().unwrap();
        let framework_data = poise::FrameworkContext {
            bot_id: serenity::UserId(NERD_BOT_ID),
            options: &self.options,
            user_data: &(),
            shard_manager: &shard_manager,
        };

        poise::dispatch_event(
            framework_data,
            &ctx,
            &poise::Event::InteractionCreate { interaction },
        )
        .await;
    }

    // For slash commands or edit tracking to work, forward interaction_create and message_update
}

#[tokio::main]
async fn main() -> Result<()> {
    create_directories()?;
    let commands = vec![chat(), create(), paginate(), say()];
    let token = std::env::var("DISCORD_TOKEN")?;
    let mut handler = Handler {
        options: poise::FrameworkOptions {
            commands,
            ..Default::default()
        },
        shard_manager: std::sync::Mutex::new(None),
    };
    poise::set_qualified_names(&mut handler.options.commands);

    let handler = std::sync::Arc::new(handler);

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let mut client = serenity::Client::builder(token, intents)
        .event_handler_arc(handler.clone())
        .await?;
    *handler.shard_manager.lock().unwrap() = Some(client.shard_manager.clone());
    client.start().await?;
    Ok(())
}
