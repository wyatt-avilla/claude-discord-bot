use crate::claude;

use crate::discord::client::CustomData;
use crate::discord::command::CommandError;
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

    let mentioned = msg.mentions.contains(&ctx.cache.current_user());

    if msg.referenced_message.as_ref().is_some_and(|m| {
        let img_attachments = m.attachments.iter().any(|a| {
            a.content_type
                .as_ref()
                .is_some_and(|t| t.starts_with("image/"))
        });
        let img_embeds = m.embeds.iter().any(|e| e.image.is_some());

        img_attachments || img_embeds
    }) && mentioned
        && msg.content.is_empty()
    {
        msg.reply(ctx, "*Claude can't view images you reply to.*")
            .await?;
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

    if server_config
        .random_interaction_chance_denominator
        .is_some_and(|d| rand::rng().random_range(1..=d.into()) == 1)
        || mentioned
    {
        let Some(api_key) = server_config.claude_api_key else {
            return Ok(());
        };

        let messages = std::iter::once(msg.clone())
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
            .collect();

        super::action::respond_with_claude_action(
            ctx,
            msg,
            custom_data,
            &api_key,
            server_config.model,
            messages,
        )
        .await?;
    }

    Ok(())
}
