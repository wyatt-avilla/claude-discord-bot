use super::handler::{ErrorReply, ResponseTrigger};
use super::message_context::MessageContext;
use crate::claude;
use crate::database::Record;

pub enum ResponseIntent<'a> {
    ShouldNotRespond,
    ErrorReplyWith(ErrorReply),
    ShouldRespondWith {
        api_key: &'a str,
        model: &'a claude::Model,
    },
}

pub fn classify_response<'a>(
    trigger: &ResponseTrigger,
    message: &impl MessageContext,
    server_config: &'a Record,
) -> ResponseIntent<'a> {
    if message.authored_by_bot() {
        return ResponseIntent::ShouldNotRespond;
    }

    let mentioned = matches!(trigger, ResponseTrigger::Mention);

    if message.is_reply() {
        return if mentioned {
            ResponseIntent::ErrorReplyWith(ErrorReply::CantSeeReplies)
        } else {
            ResponseIntent::ShouldNotRespond
        };
    }

    if !message.in_active_channel(server_config) {
        return if mentioned {
            ResponseIntent::ErrorReplyWith(ErrorReply::InactiveChannel)
        } else {
            ResponseIntent::ShouldNotRespond
        };
    }

    let Some(api_key) = &server_config.claude_api_key else {
        return if mentioned {
            ResponseIntent::ErrorReplyWith(ErrorReply::MissingAPIKey)
        } else {
            ResponseIntent::ShouldNotRespond
        };
    };

    ResponseIntent::ShouldRespondWith {
        api_key,
        model: &server_config.model,
    }
}
