use serde::Serialize;

#[cfg(test)]
mod tests;

mod content;
mod role;

pub use content::{Content, ContentBlock, ImageBlock, MediaType, TextBlock};
pub use role::Role;

#[derive(Serialize, Debug)]
pub struct Message {
    role: Role,
    content: Content,
}

impl From<poise::serenity_prelude::Message> for Message {
    fn from(value: poise::serenity_prelude::Message) -> Self {
        todo!()
    }
}
