use crate::database::Record;

use super::response_intent::{ResponseIntent, classify_response};
use crate::claude;
use crate::database;
use crate::discord::client::CustomData;
use crate::discord::command::CommandError;
use crate::discord::{MessageContext, SerenityMessageContext};
use poise::serenity_prelude::{self as serenity};
use rand::Rng;
use tokio::sync::mpsc;

pub enum ResponseTrigger {
    Mention,
    RandomChance,
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

async fn handler_task(
    id: serenity::ChannelId,
    db: database::Client,
    claude: claude::Client,
    mut rx: mpsc::Receiver<SerenityMessageContext>,
) {
    while let Some(message_context) = rx.recv().await {
        let Some(Ok(server_config)) = message_context
            .server_id()
            .map(|id| db.get_config(id.into()))
        else {
            log::error!(
                "Couldn't get server config when trying to process message '{}'",
                message_context.content()
            );
            break;
        };

        let Some(response_trigger) = response_trigger(
            &message_context,
            random_interaction_triggered(&server_config),
        ) else {
            continue;
        };

        match classify_response(&response_trigger, &message_context, &server_config) {
            ResponseIntent::ShouldNotRespond => (),
            ResponseIntent::ErrorReplyWith(reply) => {
                let (ctx, msg) = message_context.into_inner();
                if msg.reply(ctx, reply.pretty_str()).await.is_err() {
                    log::error!("Unable to reply in channel id {id}");
                    break;
                }
            }
            ResponseIntent::ShouldRespondWith { api_key, model } => {
                let Ok(history) = message_context.message_history().await else {
                    log::error!("Unable to get history in channel id {id}");
                    break;
                };

                let (ctx, _) = message_context.clone().into_inner(); // :(

                let messages = claude::Message::vec_from(&history, &ctx);

                if let Err(e) = super::action::respond_with_claude_action(
                    message_context,
                    &claude,
                    api_key,
                    model.clone(),
                    messages,
                )
                .await
                {
                    log::error!("Unable respond with action in channel id {id} ({e})");
                    break;
                }
            }
        }
    }

    log::error!("Task for channel id {id} exiting...");
}

pub async fn handle_message(
    msg_ctx: SerenityMessageContext,
    custom_data: &CustomData,
) -> Result<(), CommandError> {
    let server_config = match msg_ctx
        .server_id()
        .map(|id| custom_data.db.get_config(id.into()))
    {
        Some(Ok(cfg)) => cfg,
        None => return Ok(()),
        Some(Err(e)) => {
            log::error!(
                "Couldn't get server config when trying to process message '{}' ({})",
                msg_ctx.content(),
                e,
            );
            return Ok(());
        }
    };

    let channel_id = msg_ctx.channel_id();

    if !server_config.active_channel_ids.contains(&channel_id.get()) {
        return Ok(());
    }

    let sender_entry = || custom_data.channel_senders.entry(channel_id);

    let tx_from_new_task = || {
        let (tx, rx) = mpsc::channel::<_>(128);
        log::info!("Spawning receiver task for channel id {channel_id}");

        let db = custom_data.db.clone();
        let claude = custom_data.claude.clone();

        tokio::spawn(async move { handler_task(channel_id, db, claude, rx).await });

        tx
    };

    let tx = sender_entry()
        .or_insert_with(tx_from_new_task)
        .value()
        .clone();

    if let Err(e) = tx.send(msg_ctx.clone()).await {
        log::warn!("Couldn't send message to channel id '{channel_id}' ({e})");

        log::info!("Restarting receiver task for channel id {channel_id}");
        let new_tx = sender_entry().insert(tx_from_new_task()).value().clone();

        if let Err(e) = new_tx.send(msg_ctx).await {
            log::error!(
                "Couldn't send message after task restart for channel id '{channel_id}' ({e})"
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ResponseTrigger;
    use super::response_trigger;
    use crate::discord::MockMessageContext;

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
