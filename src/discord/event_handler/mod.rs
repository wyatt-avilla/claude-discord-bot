mod message_handler;

use super::client::CustomData;
use super::command::CommandError;
use poise::serenity_prelude as serenity;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, CustomData, CommandError>,
    custom_data: &CustomData,
) -> Result<(), CommandError> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            log::info!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            message_handler::MessageHandler::handle_message(ctx, new_message, custom_data).await?;
        }
        _ => {}
    }
    Ok(())
}
