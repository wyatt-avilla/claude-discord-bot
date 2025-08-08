use super::{ContentBlock, ImageBlock, TextBlock};
use crate::discord::NormalizeContent;

use poise::serenity_prelude as serenity;

use itertools::Itertools;
use serenity::ReactionType;
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

    fn with_contextualized_images(
        discord_message: &serenity::Message,
        message_text: &str,
        role: Role,
    ) -> Self {
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

        let mut content_blocks = attached_images
            .chain(embedded_images)
            .flat_map(move |ib| [uploaded_by_context.clone(), ContentBlock::ImageBlock(ib)])
            .peekable();

        if content_blocks.peek().is_none() {
            Message {
                role,
                content: Content::Text(message_text.to_string()),
            }
        } else if discord_message.content.is_empty() {
            Message {
                role,
                content: Content::ContentBlocks(content_blocks.collect()),
            }
        } else {
            Message {
                role,
                content: Content::ContentBlocks(
                    content_blocks
                        .chain(iter::once(ContentBlock::Text(TextBlock {
                            text: message_text.to_string(),
                        })))
                        .collect(),
                ),
            }
        }
    }

    fn bot_reactions(
        discord_message: &serenity::Message,
    ) -> Peekable<impl Iterator<Item = String>> {
        discord_message
            .reactions
            .iter()
            .filter(|r| r.me)
            .filter_map(|r| match &r.reaction_type {
                ReactionType::Unicode(s) => Some(s.to_string()),
                _ => None,
            })
            .peekable()
    }

    fn from(
        discord_message: &serenity::Message,
        context: &serenity::Context,
    ) -> impl Iterator<Item = Self> {
        let role = if discord_message.author.id == context.cache.current_user().id {
            Role::Assistant
        } else {
            Role::User
        };

        let message_text = Message::format_message(discord_message);
        let claude_message =
            Message::with_contextualized_images(discord_message, &message_text, role);
        let bot_reactions = Message::bot_reactions(discord_message).collect_vec();

        if bot_reactions.is_empty() {
            vec![claude_message].into_iter()
        } else {
            vec![
                claude_message,
                Message {
                    role: Role::Assistant,
                    content: Content::Text(if bot_reactions.len() == 1 {
                        format!("*Claude reacted with '{}'*", bot_reactions.join(", "))
                    } else {
                        format!(
                            "*Claude reacted with [{}]*",
                            bot_reactions
                                .into_iter()
                                .map(|r| format!("'{r}'"))
                                .join(", ")
                        )
                    }),
                },
            ]
            .into_iter()
        }
    }

    pub fn vec_from(
        discord_messages: &[serenity::Message],
        context: &serenity::Context,
    ) -> Vec<Message> {
        discord_messages
            .iter()
            .flat_map(|m| Message::from(m, context))
            .collect()
    }
}
