use crate::claude;

use super::handler::ErrorReply;
use crate::database::Record;
use poise::serenity_prelude::{self as serenity};
use rand::Rng;

pub trait MessageContext {
    fn authored_by_bot(&self) -> bool;
    fn mentioned(&self) -> bool;
    fn is_reply(&self) -> bool;
    fn in_active_channel(&self, server_config: &Record) -> bool;
}

pub struct SerenityMessageContext<'a> {
    pub context: &'a serenity::Context,
    pub message: &'a serenity::Message,
}

impl MessageContext for SerenityMessageContext<'_> {
    fn authored_by_bot(&self) -> bool {
        self.message.author.id == self.context.cache.current_user().id
    }

    fn mentioned(&self) -> bool {
        self.message
            .mentions
            .contains(&self.context.cache.current_user())
    }

    fn is_reply(&self) -> bool {
        self.message.referenced_message.is_some()
    }

    fn in_active_channel(&self, server_config: &Record) -> bool {
        server_config
            .active_channel_ids
            .contains(&self.message.channel_id.get())
    }
}

pub enum ResponseIntent<'a> {
    ShouldNotRespond,
    ErrorReplyWith(ErrorReply),
    ShouldRespondWith {
        api_key: &'a str,
        model: &'a claude::Model,
    },
}

pub fn classify_response<'a>(
    message: &impl MessageContext,
    server_config: &'a Record,
) -> ResponseIntent<'a> {
    if message.authored_by_bot() {
        return ResponseIntent::ShouldNotRespond;
    }

    let mentioned = message.mentioned();

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

    let random_interaction_triggered = server_config
        .random_interaction_chance_denominator
        .is_some_and(|d| rand::rng().random_range(1..=d.into()) == 1);

    if !mentioned && !random_interaction_triggered {
        return ResponseIntent::ShouldNotRespond;
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
