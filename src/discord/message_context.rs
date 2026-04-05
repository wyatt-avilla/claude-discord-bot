use itertools::Itertools;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::claude;
use crate::{database::Record, discord::command::CommandError};
use poise::serenity_prelude::{self as serenity, GetMessages};

#[cfg_attr(test, automock)]
pub trait MessageContext {
    fn into_inner(self) -> (serenity::Context, serenity::Message);
    fn authored_by_bot(&self) -> bool;
    fn is_reply(&self) -> bool;
    fn mentioned(&self) -> bool;
    fn in_active_channel(&self, server_config: &Record) -> bool;
    fn start_typing(&self) -> serenity::Typing;
    fn content(&self) -> &str;
    fn server_id(&self) -> Option<serenity::GuildId>;
    fn channel_id(&self) -> serenity::ChannelId;
    async fn message_history(&self) -> Result<Vec<serenity::Message>, CommandError>;
}

#[derive(Clone)]
pub struct SerenityMessageContext {
    pub context: serenity::Context,
    pub message: serenity::Message,
}

impl MessageContext for SerenityMessageContext {
    fn into_inner(self) -> (serenity::Context, serenity::Message) {
        (self.context, self.message)
    }

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

    fn start_typing(&self) -> serenity::Typing {
        self.message.channel_id.start_typing(&self.context.http)
    }

    fn content(&self) -> &str {
        &self.message.content
    }

    fn server_id(&self) -> Option<serenity::GuildId> {
        self.message.guild_id
    }

    fn channel_id(&self) -> serenity::ChannelId {
        self.message.channel_id
    }

    async fn message_history(&self) -> Result<Vec<serenity::Message>, CommandError> {
        Ok(std::iter::once(self.message.clone())
            .chain(
                self.channel_id()
                    .messages(
                        &self.context,
                        GetMessages::new()
                            .before(self.message.id)
                            .limit(claude::MESSAGE_CONTEXT_LENGTH - 1),
                    )
                    .await?,
            )
            .rev()
            .collect_vec())
    }
}
