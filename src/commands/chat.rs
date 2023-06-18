use nerd2::send_chat;
use nerd2::{Context, Error};
use poise::serenity_prelude as serenity;

/// chat
#[poise::command(slash_command, prefix_command)]
pub async fn chat(ctx: Context<'_>) -> Result<(), Error> {
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
