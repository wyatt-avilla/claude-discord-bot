use super::{ContentBlock, ImageBlock, TextBlock};
use crate::discord::NormalizeContent;

use poise::serenity_prelude as serenity;
use std::iter;
use std::iter::Peekable;

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

    fn contextualized_images_from(
        discord_message: &serenity::Message,
    ) -> Peekable<impl Iterator<Item = ContentBlock>> {
        let uploaded_by_context = ContentBlock::Text(TextBlock {
            text: format!(
                "*@{} uploaded the following image*",
                discord_message.author.display_name()
            ),
        });

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

        let embedded_images = discord_message.embeds.iter().filter_map(|e| {
            match (e.kind.as_deref(), e.image.as_ref()) {
                (Some("image"), _) => Some(ImageBlock {
                    url: e.url.clone()?,
                }),
                (_, Some(img)) => Some(ImageBlock {
                    url: img.url.clone(),
                }),
                _ => None,
            }
        });

        let img_blocks = attached_images.chain(embedded_images);
        img_blocks
            .flat_map(move |ib| [uploaded_by_context.clone(), ContentBlock::ImageBlock(ib)])
            .peekable()
    }

    pub fn from(discord_message: &serenity::Message, context: &serenity::Context) -> Self {
        let role = if discord_message.author.id == context.cache.current_user().id {
            Role::Assistant
        } else {
            Role::User
        };

        let message_text = Message::format_message(discord_message);

        let mut imgs = Message::contextualized_images_from(discord_message);

        if imgs.peek().is_none() {
            Message {
                role,
                content: Content::Text(message_text),
            }
        } else if discord_message.content.is_empty() {
            Message {
                role,
                content: Content::ContentBlocks(imgs.collect()),
            }
        } else {
            Message {
                role,
                content: Content::ContentBlocks(
                    imgs.chain(iter::once(ContentBlock::Text(TextBlock {
                        text: message_text,
                    })))
                    .collect(),
                ),
            }
        }
    }
}
