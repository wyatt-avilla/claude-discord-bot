use crate::claude;

use super::super::client::CustomData;
use super::super::command::CommandError;
use poise::serenity_prelude::{self as serenity, GetMessages};
use rand::Rng;

pub struct MessageHandler {}

impl MessageHandler {
    async fn respond_with_claude_action(
        ctx: &serenity::Context,
        msg: &serenity::Message,
        custom_data: &CustomData,
        api_key: &str,
        messages: Vec<serenity::Message>,
    ) -> Result<(), CommandError> {
        let mentioned = msg.mentions.contains(&ctx.cache.current_user());

        let resp = match custom_data
            .claude
            .get_response(messages, ctx, api_key)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::error!("Error requesting response from Claude ({e})");
                if mentioned {
                    msg.reply(ctx, "*An error occurred while Claude tried to respond*")
                        .await?;
                }
                return Ok(());
            }
        };

        match resp.stop_reason {
            claude::StopReason::MaxTokens => {
                log::error!(
                    "Claude hit the max amount of tokens while trying to respond to '{}'",
                    msg.content
                );
                if mentioned {
                    msg.reply(
                        ctx,
                        "*Claude hit the max amount of tokens while trying to respond*",
                    )
                    .await?;
                }
                Ok(())
            }
            claude::StopReason::Refusal => {
                log::error!("Claude refused to respond to '{}'", msg.content);
                if mentioned {
                    msg.reply(
                        ctx,
                        "*Content in this interaction violates Anthropic's terms of service*",
                    )
                    .await?;
                }
                Ok(())
            }
            _ => {
                if resp.content.len() > 1 {
                    log::warn!(
                        "Multiple actions were provided, using the last one.\nMessage: '{}'",
                        msg.content
                    );
                }

                if let Some(action) = resp.content.last() {
                    match action {
                        claude::Action::SendMessage(txt) => {
                            msg.channel_id.say(ctx, txt).await?;
                        }
                        claude::Action::ReactToMessage(emoji) => {
                            msg.react(ctx, emoji.clone()).await?;
                        }
                        claude::Action::Pass => {
                            log::warn!("Claude chose not to respond to '{}'", msg.content);
                        }
                    }
                }
                Ok(())
            }
        }
    }

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

            MessageHandler::respond_with_claude_action(ctx, msg, custom_data, &api_key, messages)
                .await?;
        }

        Ok(())
    }
}
