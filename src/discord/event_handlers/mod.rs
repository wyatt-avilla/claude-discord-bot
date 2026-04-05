use super::SerenityMessageContext;
use crate::discord::CommandError;
use crate::discord::client::CustomData;
use poise::serenity_prelude as serenity;

mod message;

pub async fn handle_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, CustomData<SerenityMessageContext>, CommandError>,
    custom_data: &CustomData<SerenityMessageContext>,
) -> Result<(), CommandError> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            log::info!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            message::handle_message(
                SerenityMessageContext {
                    context: ctx.clone(),
                    message: new_message.clone(),
                },
                custom_data,
            )
            .await?;
        }
        _ => {}
    }
    Ok(())
}
