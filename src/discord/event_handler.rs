use super::client::CustomData;
use super::command::CommandError;
use poise::serenity_prelude as serenity;

pub async fn handle_message(
    ctx: &serenity::Context,
    msg: &serenity::Message,
) -> Result<(), CommandError> {
    if msg.author.id != ctx.cache.current_user().id {
        msg.channel_id.say(ctx, "Non-replying message!").await?;
    }

    Ok(())
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, CustomData, CommandError>,
    _data: &CustomData,
) -> Result<(), CommandError> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            log::info!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => handle_message(ctx, new_message).await?,
        _ => {}
    }
    Ok(())
}
