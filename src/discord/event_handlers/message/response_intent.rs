use super::handler::ResponseTrigger;
use crate::claude;
use crate::database::Record;
use crate::discord::MessageContext;
use crate::discord::error_reply::ErrorReply;

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

    // TODO: move this to sender thread basically
    if !message.in_active_channel(server_config) {
        return if mentioned {
            ResponseIntent::ErrorReplyWith(ErrorReply::InactiveChannel)
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
    use super::super::handler::ResponseTrigger;
    use super::ResponseIntent;
    use super::classify_response;
    use crate::database::Record;
    use crate::discord::MockMessageContext;
    use crate::discord::error_reply::ErrorReply;

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
    fn not_in_active_channel_no_mention_no_response() {
        let cfg = Record::default();
        let mut msg = MockMessageContext::new();

        msg.expect_authored_by_bot().once().return_const(false);
        msg.expect_is_reply().once().return_const(false);
        msg.expect_in_active_channel().once().return_const(false);

        let res = classify_response(&ResponseTrigger::RandomChance, &msg, &cfg);

        assert!(matches!(res, ResponseIntent::ShouldNotRespond));
    }

    #[test]
    fn not_in_active_channel_mention_err_msg() {
        let cfg = Record::default();
        let mut msg = MockMessageContext::new();

        msg.expect_authored_by_bot().once().return_const(false);
        msg.expect_is_reply().once().return_const(false);
        msg.expect_in_active_channel().once().return_const(false);

        let res = classify_response(&ResponseTrigger::Mention, &msg, &cfg);

        assert!(matches!(
            res,
            ResponseIntent::ErrorReplyWith(ErrorReply::InactiveChannel)
        ));
    }

    #[test]
    fn no_api_key_no_mention_no_response() {
        let cfg = Record::default();
        let mut msg = MockMessageContext::new();

        msg.expect_authored_by_bot().once().return_const(false);
        msg.expect_is_reply().once().return_const(false);
        msg.expect_in_active_channel().once().return_const(true);

        let res = classify_response(&ResponseTrigger::RandomChance, &msg, &cfg);

        assert!(matches!(res, ResponseIntent::ShouldNotRespond));
    }

    #[test]
    fn no_api_key_mention_err_msg() {
        let cfg = Record::default();
        let mut msg = MockMessageContext::new();

        msg.expect_authored_by_bot().once().return_const(false);
        msg.expect_is_reply().once().return_const(false);
        msg.expect_in_active_channel().once().return_const(true);

        let res = classify_response(&ResponseTrigger::Mention, &msg, &cfg);

        assert!(matches!(
            res,
            ResponseIntent::ErrorReplyWith(ErrorReply::MissingAPIKey)
        ));
    }
}
