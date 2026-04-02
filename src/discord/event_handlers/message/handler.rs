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

fn random_interaction_triggered(server_config: &Record) -> bool {
    server_config
        .random_interaction_chance_denominator
        .is_some_and(|d| rand::rng().random_range(1..=d.into()) == 1)
}

fn response_trigger(
    message: &impl MessageContext,
    random_interaction_triggered: bool,
) -> Option<ResponseTrigger> {
    let mentioned = message.mentioned();
    if mentioned {
        return Some(ResponseTrigger::Mention);
    }

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

    let Some(response_trigger) = response_trigger(
        &message_context,
        random_interaction_triggered(&server_config),
    ) else {
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

#[cfg(test)]
mod tests {
    use super::super::message_context::MockMessageContext;
    use super::ResponseTrigger;
    use super::response_trigger;

    fn mentioned_message() -> MockMessageContext {
        let mut msg = MockMessageContext::new();

        msg.expect_mentioned().once().return_const(true);

        msg
    }

    fn message() -> MockMessageContext {
        let mut msg = MockMessageContext::new();

        msg.expect_mentioned().once().return_const(false);

        msg
    }

    #[test]
    fn mention_triggers() {
        let msg = mentioned_message();

        let resp = response_trigger(&msg, false);

        assert!(matches!(resp, Some(ResponseTrigger::Mention)));
    }

    #[test]
    fn random_triggers() {
        let msg = message();

        let resp = response_trigger(&msg, true);

        assert!(matches!(resp, Some(ResponseTrigger::RandomChance)));
    }

    #[test]
    fn neither_no_response() {
        let msg = message();

        let resp = response_trigger(&msg, false);

        assert!(resp.is_none());
    }

    #[test]
    fn mention_takes_priority_over_random() {
        let msg = mentioned_message();

        let resp = response_trigger(&msg, true);

        assert!(matches!(resp, Some(ResponseTrigger::Mention)));
    }
}
