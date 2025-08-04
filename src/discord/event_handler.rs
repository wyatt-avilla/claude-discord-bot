use super::client::CustomData;
use super::command::CommandError;
use poise::serenity_prelude as serenity;

pub async fn handle_message(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    custom_data: &CustomData,
) -> Result<(), CommandError> {
    if msg.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    let Some(server_id) = msg.guild_id else {
        return Ok(());
    };

    let Ok(server_config) = custom_data.db.get_config(server_id.get()) else {
        log::warn!(
            "Couldn't get server config when trying to process message '{}'",
            msg.content
        );
        return Ok(());
    };

    if !server_config
        .active_channel_ids
        .contains(&msg.channel_id.get())
    {
        return Ok(());
    }

    msg.channel_id.say(ctx, "Non-replying message!").await?;

    Ok(())
}

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
            handle_message(ctx, new_message, custom_data).await?;
        }
        _ => {}
    }
    Ok(())
}
