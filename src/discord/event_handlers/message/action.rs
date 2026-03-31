use crate::claude;

use crate::discord::client::CustomData;
use crate::discord::command::CommandError;
use poise::serenity_prelude as serenity;

pub async fn respond_with_claude_action(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    custom_data: &CustomData,
    api_key: &str,
    model: claude::Model,
    messages: Vec<serenity::Message>,
) -> Result<(), CommandError> {
    let mentioned = msg.mentions.contains(&ctx.cache.current_user());

    let _typing = if mentioned {
        Some(msg.channel_id.start_typing(&ctx.http))
    } else {
        None
    };

    let resp = match custom_data
        .claude
        .get_response(messages, ctx, api_key, model)
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
            if resp.content.is_empty() {
                log::error!("Empty response provided for '{}'", msg.content);
            }

            for action in resp.content {
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
