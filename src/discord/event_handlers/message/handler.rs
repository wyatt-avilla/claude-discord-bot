use crate::claude;

use crate::database::Record;
use crate::discord::client::CustomData;
use crate::discord::command::CommandError;
use poise::serenity_prelude::{self as serenity, GetMessages};
use rand::Rng;

fn message_in_active_channel(server_config: &Record, msg: &serenity::Message) -> bool {
    server_config
        .active_channel_ids
        .contains(&msg.channel_id.get())
}

fn random_interaction_triggered(server_config: &Record) -> bool {
    server_config
        .random_interaction_chance_denominator
        .is_some_and(|d| rand::rng().random_range(1..=d.into()) == 1)
}

async fn get_message_history(
    ctx: &serenity::Context,
    msg: &serenity::Message,
) -> Result<Vec<serenity::Message>, CommandError> {
    Ok(std::iter::once(msg.clone())
        .chain(
            msg.channel_id
                .messages(
                    ctx,
                    GetMessages::new()
                        .before(msg.id)
                        .limit(claude::MESSAGE_CONTEXT_LENGTH - 1),
                )
                .await?,
        )
        .rev()
        .collect::<Vec<_>>())
}

pub async fn handle_message(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    custom_data: &CustomData,
) -> Result<(), CommandError> {
    if msg.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    let mentioned = msg.mentions.contains(&ctx.cache.current_user());

    if msg.referenced_message.is_some() {
        if mentioned {
            msg.reply(ctx, "*Claude can't see replies. View the tracking issue* [here](<https://github.com/wyatt-avilla/claude-discord-bot/issues/18>).").await?;
        }

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

    if !message_in_active_channel(&server_config, msg) {
        if mentioned {
            msg.reply(
                ctx,
                "*Claude isn't configured to be active in this channel.*",
            )
            .await?;
        }
        return Ok(());
    }

    if !mentioned && !random_interaction_triggered(&server_config) {
        return Ok(());
    }

    let Some(api_key) = server_config.claude_api_key else {
        if mentioned {
            msg.reply(ctx, "*Anthropic API key not set.*").await?;
        }
        return Ok(());
    };

    super::action::respond_with_claude_action(
        ctx,
        msg,
        custom_data,
        &api_key,
        server_config.model,
        get_message_history(ctx, msg).await?,
    )
    .await?;

    Ok(())
}
