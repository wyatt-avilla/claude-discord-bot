use crate::discord::NormalizeContent;

use poise::serenity_prelude as serenity;

use super::{Content, Role};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Message {
    role: Role,
    content: Content,
}

impl Message {
    pub fn from(discord_message: &serenity::Message, context: &serenity::Context) -> Self {
        let role = if discord_message.author.id == context.cache.current_user().id {
            Role::Assistant
        } else {
            Role::User
        };

        let time = discord_message
            .timestamp
            .with_timezone(&chrono::Local)
            .format("%-m-%-d-%Y %-I:%M%p")
            .to_string();

        let message_text = format!(
            "[{}] {}: {}",
            time,
            discord_message.author.display_name(),
            discord_message.normalize_content(),
        );

        Message {
            role,
            content: Content::Text(message_text),
        }
    }
}
