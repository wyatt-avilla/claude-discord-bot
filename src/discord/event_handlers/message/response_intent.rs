use super::handler::{ErrorReply, ResponseTrigger};
use super::message_context::MessageContext;
use crate::claude;
use crate::database::Record;

pub enum ResponseIntent<'a> {
    ShouldNotRespond,
    ErrorReplyWith(ErrorReply),
    ShouldRespondWith {
        api_key: &'a str,
        model: &'a claude::Model,
    },
}

pub fn classify_response<'a>(
    trigger: &ResponseTrigger,
    message: &impl MessageContext,
    server_config: &'a Record,
) -> ResponseIntent<'a> {
    if message.authored_by_bot() {
        return ResponseIntent::ShouldNotRespond;
    }

    let mentioned = matches!(trigger, ResponseTrigger::Mention);

    if message.is_reply() {
        return if mentioned {
            ResponseIntent::ErrorReplyWith(ErrorReply::CantSeeReplies)
        } else {
            ResponseIntent::ShouldNotRespond
        };
    }

    let Some(api_key) = &server_config.claude_api_key else {
        return if mentioned {
            ResponseIntent::ErrorReplyWith(ErrorReply::MissingAPIKey)
        } else {
            ResponseIntent::ShouldNotRespond
        };
    };

    ResponseIntent::ShouldRespondWith {
        api_key,
        model: &server_config.model,
    }
}

#[cfg(test)]
mod tests {
    use super::super::handler::ErrorReply;
    use super::super::handler::ResponseTrigger;
    use super::super::message_context::MockMessageContext;
    use super::ResponseIntent;
    use super::classify_response;
    use crate::database::Record;

    #[test]
    fn authored_by_bot_no_response() {
        let cfg = Record::default();
        let mut msg = MockMessageContext::new();

        msg.expect_authored_by_bot().once().return_const(true);

        let res = classify_response(&ResponseTrigger::Mention, &msg, &cfg);

        assert!(matches!(res, ResponseIntent::ShouldNotRespond));
    }

    #[test]
    fn reply_no_mention_no_response() {
        let cfg = Record::default();
        let mut msg = MockMessageContext::new();

        msg.expect_authored_by_bot().once().return_const(false);
        msg.expect_is_reply().once().return_const(true);

        let res = classify_response(&ResponseTrigger::RandomChance, &msg, &cfg);

        assert!(matches!(res, ResponseIntent::ShouldNotRespond));
    }

    #[test]
    fn reply_no_mention_err_msg() {
        let cfg = Record::default();
        let mut msg = MockMessageContext::new();

        msg.expect_authored_by_bot().once().return_const(false);
        msg.expect_is_reply().once().return_const(true);

        let res = classify_response(&ResponseTrigger::Mention, &msg, &cfg);

        assert!(matches!(
            res,
            ResponseIntent::ErrorReplyWith(ErrorReply::CantSeeReplies)
        ));
    }

    #[test]
    fn no_api_key_no_mention_no_response() {
        let cfg = Record::default();
        let mut msg = MockMessageContext::new();

        msg.expect_authored_by_bot().once().return_const(false);
        msg.expect_is_reply().once().return_const(false);

        let res = classify_response(&ResponseTrigger::RandomChance, &msg, &cfg);

        assert!(matches!(res, ResponseIntent::ShouldNotRespond));
    }

    #[test]
    fn no_api_key_mention_err_msg() {
        let cfg = Record::default();
        let mut msg = MockMessageContext::new();

        msg.expect_authored_by_bot().once().return_const(false);
        msg.expect_is_reply().once().return_const(false);

        let res = classify_response(&ResponseTrigger::Mention, &msg, &cfg);

        assert!(matches!(
            res,
            ResponseIntent::ErrorReplyWith(ErrorReply::MissingAPIKey)
        ));
    }
}
