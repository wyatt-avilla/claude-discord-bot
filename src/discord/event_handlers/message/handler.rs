use crate::{claude, database::Record};

use super::message_context::{MessageContext, SerenityMessageContext};
use super::response_intent::{ResponseIntent, classify_response};
use crate::discord::client::CustomData;
use crate::discord::command::CommandError;
use poise::serenity_prelude::{self as serenity, GetMessages};
use rand::Rng;

pub enum ResponseTrigger {
    Mention,
    RandomChance,
}

pub enum ErrorReply {
    CantSeeReplies,
    InactiveChannel,
    MissingAPIKey,
}

impl ErrorReply {
    fn pretty_str(&self) -> &'static str {
        match self {
            ErrorReply::CantSeeReplies => {
                "*Claude can't see replies. View the tracking issue* [here](<https://github.com/wyatt-avilla/claude-discord-bot/issues/18>)."
            }
            ErrorReply::InactiveChannel => {
                "*Claude isn't configured to be active in this channel.*"
            }
            ErrorReply::MissingAPIKey => "*Anthropic API key not set.*",
        }
    }
}

async fn get_message_history(
    ctx: &serenity::Context,
    msg: &serenity::Message,
) -> Result<Vec<serenity::Message>, CommandError> {
    Ok(std::iter::once(msg.clone())
        .chain(
            msg.channel_id
                .messages(
                    ctx,
                    GetMessages::new()
                        .before(msg.id)
                        .limit(claude::MESSAGE_CONTEXT_LENGTH - 1),
                )
                .await?,
        )
        .rev()
        .collect::<Vec<_>>())
}

fn response_trigger(
    message: &impl MessageContext,
    server_config: &Record,
    mut rng: impl Rng,
) -> Option<ResponseTrigger> {
    let mentioned = message.mentioned();
    if mentioned {
        return Some(ResponseTrigger::Mention);
    }

    let random_interaction_triggered = server_config
        .random_interaction_chance_denominator
        .is_some_and(|d| rng.random_range(1..=d.into()) == 1);

    if random_interaction_triggered {
        return Some(ResponseTrigger::RandomChance);
    }

    None
}

pub async fn handle_message(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    custom_data: &CustomData,
) -> Result<(), CommandError> {
    let Some(Ok(server_config)) = msg.guild_id.map(|id| custom_data.db.get_config(id.into()))
    else {
        log::warn!(
            "Couldn't get server config when trying to process message '{}'",
            msg.content
        );
        return Ok(());
    };

    let message_context = SerenityMessageContext {
        context: ctx,
        message: msg,
    };

    let Some(response_trigger) = response_trigger(&message_context, &server_config, rand::rng())
    else {
        return Ok(());
    };

    match classify_response(&response_trigger, &message_context, &server_config) {
        ResponseIntent::ShouldNotRespond => (),
        ResponseIntent::ErrorReplyWith(reply) => {
            msg.reply(ctx, reply.pretty_str()).await?;
        }
        ResponseIntent::ShouldRespondWith { api_key, model } => {
            super::action::respond_with_claude_action(
                ctx,
                msg,
                custom_data,
                api_key,
                model.clone(),
                get_message_history(ctx, msg).await?,
            )
            .await?;
        }
    }
    Ok(())
}
