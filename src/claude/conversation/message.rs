use super::{ContentBlock, ImageBlock, TextBlock};
use crate::discord::NormalizeContent;

use itertools::Itertools;
use poise::serenity_prelude as serenity;

use super::{Content, Role};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Message {
    pub role: Role,
    pub content: Content,
}

impl Message {
    fn format_message(msg: &serenity::Message) -> String {
        let fmt = |msg: &serenity::Message, reply_context: String| {
            let time = msg
                .timestamp
                .with_timezone(&chrono::Local)
                .format("%-m-%-d-%Y %-I:%M%p")
                .to_string();

            format!(
                "[{}] {}:{} {}",
                time,
                msg.author.display_name(),
                reply_context,
                msg.normalize_content(),
            )
        };

        match &msg.referenced_message {
            Some(reply_msg) if !reply_msg.content.is_empty() => {
                let reply_context = format!("(Replying to: '{}')", fmt(reply_msg, String::new()));
                fmt(msg, reply_context)
            }
            _ => fmt(msg, String::new()),
        }
    }

    pub fn from(discord_message: &serenity::Message, context: &serenity::Context) -> Self {
        let role = if discord_message.author.id == context.cache.current_user().id {
            Role::Assistant
        } else {
            Role::User
        };

        let message_text = Message::format_message(discord_message);

        let attached_images = discord_message.attachments.iter().filter_map(|a| {
            if a.content_type
                .as_ref()
                .is_some_and(|t| t.starts_with("image/"))
            {
                Some(ImageBlock { url: a.url.clone() })
            } else {
                None
            }
        });

        let embedded_images = discord_message
            .embeds
            .iter()
            .filter_map(|e| e.image.as_ref())
            .map(|ei| ImageBlock {
                url: ei.url.clone(),
            });

        let imgs = attached_images.chain(embedded_images).collect_vec();

        if imgs.is_empty() {
            Message {
                role,
                content: Content::Text(message_text),
            }
        } else {
            let cbs = imgs.into_iter().map(ContentBlock::ImageBlock);

            if discord_message.content.is_empty() {
                Message {
                    role,
                    content: Content::ContentBlocks(cbs.collect()),
                }
            } else {
                Message {
                    role,
                    content: Content::ContentBlocks(
                        cbs.chain(std::iter::once(ContentBlock::Text(TextBlock {
                            text: message_text,
                        })))
                        .collect(),
                    ),
                }
            }
        }
    }
}
