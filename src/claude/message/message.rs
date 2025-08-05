use super::MediaType;
use crate::{
    claude::message::{ContentBlock, ImageBlock, TextBlock},
    discord::NormalizeContent,
};
use base64::{Engine as _, engine::general_purpose};
use futures::future::join_all;

use poise::serenity_prelude as serenity;

use super::{Content, Role};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Message {
    role: Role,
    content: Content,
}

impl Message {
    pub async fn from(discord_message: &serenity::Message, context: &serenity::Context) -> Self {
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

        let image_futures = discord_message.attachments.iter().map(|a| async move {
            let media_type: MediaType = a.content_type.clone()?.as_str().try_into().ok()?;
            let img_bytes = a.download().await.ok()?;
            let b64 = general_purpose::STANDARD.encode(&img_bytes);

            Some(ImageBlock {
                media_type,
                data: b64,
            })
        });

        let imgs: Vec<ImageBlock> = join_all(image_futures)
            .await
            .into_iter()
            .flatten()
            .collect();

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
