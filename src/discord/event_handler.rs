use super::client::CustomData;
use super::command::CommandError;
use poise::serenity_prelude::{self as serenity, GetMessages};
use rand::Rng;

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

    if msg.mentions.contains(&ctx.cache.current_user())
        || server_config
            .random_interaction_chance_denominator
            .is_some_and(|d| rand::rng().random_range(1..=d.into()) == 1)
    {
        let builder = GetMessages::new().before(msg.id).limit(15);
        let messages = msg.channel_id.messages(ctx, builder).await?;
        let Some(api_key) = server_config.claude_api_key else {
            return Ok(());
        };

        let resp = match custom_data
            .claude
            .get_response(messages, ctx, &api_key)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::error!("Error requesting response from Claude {e}");
                return Ok(());
            }
        };

        if let Some(resp) = resp {
            msg.channel_id.say(ctx, resp).await?;
        } else {
            log::warn!("Claude chose not to respond");
        }
    }

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
