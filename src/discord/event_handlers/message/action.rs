use crate::claude;

use crate::discord::CommandError;
use crate::discord::MessageContext;
use crate::discord::error_reply::ErrorReply;

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
    message_context: impl MessageContext,
    claude: &impl claude::GetResponse,
    api_key: &str,
    model: claude::Model,
    messages: Vec<claude::Message>,
) -> Result<(), CommandError> {
    let mentioned = message_context.mentioned();

    let _typing = if mentioned {
        Some(message_context.start_typing())
    } else {
        None
    };

    match channel_action_from_claude_response(
        &message_context,
        claude.get_response(&messages, api_key, &model).await,
    ) {
        None => Ok(()),
        Some(ChannelAction::ErrorReply(reply)) => {
            message_context.error_reply(reply).await?;
            Ok(())
        }
        Some(ChannelAction::ClaudeActions(actions)) => {
            let (ctx, msg) = message_context.into_inner();
            for action in actions {
                match action {
                    claude::Action::SendMessage(txt) => {
                        msg.channel_id.say(&ctx, txt).await?;
                    }
                    claude::Action::ReactToMessage(emoji) => {
                        msg.react(&ctx, emoji.clone()).await?;
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

#[cfg(test)]
mod tests {
    use super::ChannelAction;
    use crate::discord::MockMessageContext;
    use crate::discord::error_reply::ErrorReply;
    use crate::{
        claude::{ClaudeError, Response, StopReason, Usage},
        discord::event_handlers::message::action::channel_action_from_claude_response,
    };

    async fn http_err() -> ClaudeError {
        ClaudeError::Http(reqwest::get("not a url").await.unwrap_err())
    }

    fn response(stop_reason: StopReason) -> Response {
        Response {
            stop_reason,
            usage: Usage {
                input_tokens: 0,
                output_tokens: 0,
            },
            content: vec![],
        }
    }

    #[tokio::test]
    async fn request_error_mentioned_error_reply() {
        let mut ctx = MockMessageContext::new();
        ctx.expect_mentioned().once().return_const(true);

        let resp = Err(http_err().await);

        let res = channel_action_from_claude_response(&ctx, resp);

        assert!(matches!(
            res,
            Some(ChannelAction::ErrorReply(ErrorReply::SomethingWentWrong))
        ));
    }

    #[tokio::test]
    async fn request_error_no_mention_do_nothing() {
        let mut ctx = MockMessageContext::new();
        ctx.expect_mentioned().once().return_const(false);

        let resp = Err(http_err().await);

        let res = channel_action_from_claude_response(&ctx, resp);

        assert!(res.is_none());
    }

    #[test]
    fn max_tokens_mentioned_error_reply() {
        let mut ctx = MockMessageContext::new();
        ctx.expect_mentioned().once().return_const(true);

        let resp = Ok(response(StopReason::MaxTokens));

        let res = channel_action_from_claude_response(&ctx, resp);

        assert!(matches!(
            res,
            Some(ChannelAction::ErrorReply(ErrorReply::MaxTokens))
        ));
    }

    #[test]
    fn max_tokens_no_mention_do_nothing() {
        let mut ctx = MockMessageContext::new();
        ctx.expect_mentioned().once().return_const(false);

        let resp = Ok(response(StopReason::MaxTokens));

        let res = channel_action_from_claude_response(&ctx, resp);

        assert!(res.is_none());
    }

    #[test]
    fn refusal_mentioned_error_reply() {
        let mut ctx = MockMessageContext::new();
        ctx.expect_mentioned().once().return_const(true);

        let resp = Ok(response(StopReason::Refusal));

        let res = channel_action_from_claude_response(&ctx, resp);

        assert!(matches!(
            res,
            Some(ChannelAction::ErrorReply(
                ErrorReply::TermsOfServiceViolation
            ))
        ));
    }

    #[test]
    fn refusal_no_mention_do_nothing() {
        let mut ctx = MockMessageContext::new();
        ctx.expect_mentioned().once().return_const(false);

        let resp = Ok(response(StopReason::Refusal));

        let res = channel_action_from_claude_response(&ctx, resp);

        assert!(res.is_none());
    }

    #[test]
    fn empty_content_do_nothing() {
        let mut ctx = MockMessageContext::new();
        ctx.expect_mentioned().once().return_const(false);

        let resp = Ok(response(StopReason::EndTurn));

        let res = channel_action_from_claude_response(&ctx, resp);

        assert!(res.is_none());
    }
}
