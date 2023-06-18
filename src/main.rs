use nerd2::send_chat;
use nerd2::{ApplicationContext, Context, Data, Error, MyModal};
use poise::serenity_prelude as serenity;
use poise::Modal;

/// create a character
#[poise::command(slash_command)]
pub async fn modal(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = MyModal::execute(ctx).await?;
    println!("Got data: {:?}", data);

    Ok(())
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

/// chat
#[poise::command(slash_command, prefix_command)]
async fn chat(ctx: Context<'_>) -> Result<(), Error> {
    let channel = ctx.channel_id();
    // let handle = ctx.say("pick a model (in the future)").await?;
    let handle = ctx
        .send(|f| {
            f.components(|f| {
                f.create_action_row(|f| {
                    f.create_select_menu(|f| {
                        f.custom_id("hi")
                            .options(|f| f.create_option(|f| f.value("hi").label("test")))
                    })
                })
            })
        })
        .await?;
    let message = handle.message().await?;
    let interaction = message.await_component_interaction(ctx).await.unwrap(); // TODO: make it not unwrap
    let value = &interaction.data.values[0];
    interaction
        .create_interaction_response(ctx, |f| {
            f.kind(serenity::InteractionResponseType::UpdateMessage)
                .interaction_response_data(|f| f.content(value).components(|f| f))
        })
        .await?;
    let message_id = message.id;
    let thread = channel
        .create_public_thread(ctx, message_id, |thread| thread.name(value))
        .await?;
    let reply = thread.await_reply(ctx).await.unwrap(); // TODO: make it not unwrap
    let text = &reply.content;
    dbg!(&reply);
    let answer = send_chat(text).await?;
    thread.say(ctx, answer).await?;
    Ok(())
}

const TEST_SERVER_ID: u64 = 1113998071194456195;

// TODO: webhooks are fake users
// TODO: modals for creating/editing characters?

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let options = poise::FrameworkOptions {
        commands: vec![age(), chat(), modal()],
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
