#[cfg(test)]
use mockall::{automock, predicate::*};

use poise::serenity_prelude::{self as serenity};

#[cfg_attr(test, automock)]
pub trait MessageContext {
    fn authored_by_bot(&self) -> bool;
    fn is_reply(&self) -> bool;
    fn mentioned(&self) -> bool;
    fn start_typing(&self) -> serenity::Typing;
    fn content(&self) -> &str;
}

pub struct SerenityMessageContext<'a> {
    pub context: &'a serenity::Context,
    pub message: &'a serenity::Message,
}

impl MessageContext for SerenityMessageContext<'_> {
    fn authored_by_bot(&self) -> bool {
        self.message.author.id == self.context.cache.current_user().id
    }

    fn is_reply(&self) -> bool {
        self.message.referenced_message.is_some()
    }

    fn mentioned(&self) -> bool {
        self.message
            .mentions
            .contains(&self.context.cache.current_user())
    }

    fn start_typing(&self) -> serenity::Typing {
        self.message.channel_id.start_typing(&self.context.http)
    }

    fn content(&self) -> &str {
        &self.message.content
    }
}
