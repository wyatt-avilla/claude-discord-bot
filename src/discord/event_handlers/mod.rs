use crate::discord::client::CustomData;
use crate::discord::command::CommandError;
use poise::serenity_prelude as serenity;

mod message;

pub async fn handle_event(
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
            message::handle_message(ctx, new_message, custom_data).await?;
        }
        _ => {}
    }
    Ok(())
}
