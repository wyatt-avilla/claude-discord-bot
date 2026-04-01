use crate::database::Record;
use poise::serenity_prelude::{self as serenity};

pub trait MessageContext {
    fn authored_by_bot(&self) -> bool;
    fn is_reply(&self) -> bool;
    fn mentioned(&self) -> bool;
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

    fn is_reply(&self) -> bool {
        self.message.referenced_message.is_some()
    }

    fn mentioned(&self) -> bool {
        self.message
            .mentions
            .contains(&self.context.cache.current_user())
    }

    fn in_active_channel(&self, server_config: &Record) -> bool {
        server_config
            .active_channel_ids
            .contains(&self.message.channel_id.get())
    }
}
