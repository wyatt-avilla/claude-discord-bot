mod client;
mod command;
mod error_reply;
mod event_handlers;
mod message;
mod message_context;

pub use client::Bot;
pub use message::NormalizeContent;
pub use message_context::{MessageContext, SerenityMessageContext};

#[cfg(test)]
pub use message_context::MockMessageContext;

type CommandError = Box<dyn std::error::Error + Send + Sync>;
type PoiseContext<'a> =
    poise::Context<'a, client::CustomData<SerenityMessageContext>, CommandError>;
