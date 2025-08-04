use serde::Serialize;

#[cfg(test)]
mod tests;

mod content;
mod role;

pub use content::{Content, ContentBlock, ImageBlock, MediaType, TextBlock};
pub use role::Role;

#[derive(Serialize)]
pub struct Message {
    role: Role,
    content: Content,
}
