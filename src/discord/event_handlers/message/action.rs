use crate::claude;

use super::handler::ErrorReply;
use super::message_context::{MessageContext, SerenityMessageContext};
use crate::discord::command::CommandError;
use poise::serenity_prelude as serenity;

enum ChannelAction {
    ErrorReply(ErrorReply),
    ClaudeActions(Vec<claude::Action>),
}

fn channel_action_from_claude_response(
    message: &impl MessageContext,
    claude_response: Result<claude::Response, claude::ClaudeError>,
) -> Option<ChannelAction> {
    let mentioned = message.mentioned();

    let resp = match claude_response {
        Ok(r) => r,
        Err(e) => {
            log::error!("Error requesting response from Claude ({e})");
            return if mentioned {
                Some(ChannelAction::ErrorReply(ErrorReply::SomethingWentWrong))
            } else {
                None
            };
        }
    };

    match resp.stop_reason {
        claude::StopReason::MaxTokens => {
            log::error!(
                "Claude hit the max amount of tokens while trying to respond to '{}'",
                message.content()
            );
            if mentioned {
                Some(ChannelAction::ErrorReply(ErrorReply::MaxTokens))
            } else {
                None
            }
        }
        claude::StopReason::Refusal => {
            log::error!("Claude refused to respond to '{}'", message.content());
            if mentioned {
                Some(ChannelAction::ErrorReply(
                    ErrorReply::TermsOfServiceViolation,
                ))
            } else {
                None
            }
        }
        _ => {
            if resp.content.is_empty() {
                log::error!("Empty response provided for '{}'", message.content());
                None
            } else {
                Some(ChannelAction::ClaudeActions(resp.content))
            }
        }
    }
}

pub async fn respond_with_claude_action(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    claude: &impl claude::GetResponse,
    api_key: &str,
    model: claude::Model,
    messages: Vec<claude::Message>,
) -> Result<(), CommandError> {
    let message_context = SerenityMessageContext {
        message: msg,
        context: ctx,
    };

    let mentioned = message_context.mentioned();

    let _typing = if mentioned {
        Some(message_context.start_typing())
    } else {
        None
    };

    match channel_action_from_claude_response(
        &message_context,
        claude.get_response(messages, api_key, model).await,
    ) {
        None => Ok(()),
        Some(ChannelAction::ErrorReply(reply)) => {
            msg.reply(ctx, reply.pretty_str()).await?;
            Ok(())
        }
        Some(ChannelAction::ClaudeActions(actions)) => {
            for action in actions {
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
