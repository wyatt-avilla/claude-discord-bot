use crate::claude;
use itertools::Itertools;

use super::client::CustomData;
use super::command::CommandError;
use poise::serenity_prelude::{self as serenity, GetMessages};
use rand::Rng;

pub async fn handle_message(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    custom_data: &CustomData,
) -> Result<(), CommandError> {
    if msg.author.id == ctx.cache.current_user().id || msg.referenced_message.is_some() {
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
        let Some(api_key) = server_config.claude_api_key else {
            return Ok(());
        };

        let messages = std::iter::once(msg.clone())
            .chain(
                msg.channel_id
                    .messages(ctx, GetMessages::new().before(msg.id).limit(15))
                    .await?,
            )
            .collect_vec();

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

        match resp {
            claude::Response::Message(text) => {
                msg.channel_id.say(ctx, text).await?;
            }
            claude::Response::Reaction(reaction) => {
                msg.react(ctx, reaction).await?;
            }
            claude::Response::Pass => {
                log::warn!("Claude chose not to response to '{}'", msg.content);
            }
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
