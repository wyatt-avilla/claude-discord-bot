use super::tools;
use crate::claude::ClaudeError;
use itertools::Itertools;
use serde::Deserialize;

use anthropic::types::{ContentBlock, StopReason, Usage};
use poise::serenity_prelude as serenity;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum Action {
    SendMessage(String),
    ReactToMessage(serenity::ReactionType),
    Pass,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Response {
    pub stop_reason: Option<StopReason>,
    pub usage: Usage,
    pub content: Vec<Action>,
}

impl From<anthropic::types::MessagesResponse> for Response {
    fn from(value: anthropic::types::MessagesResponse) -> Self {
        let content = value
            .content
            .into_iter()
            .map(Action::try_from)
            .filter_map(std::result::Result::ok)
            .collect_vec();

        Response {
            stop_reason: value.stop_reason,
            usage: value.usage,
            content,
        }
    }
}

impl TryFrom<ContentBlock> for Action {
    type Error = ClaudeError;

    fn try_from(value: ContentBlock) -> Result<Self, Self::Error> {
        let ContentBlock::ToolUse { id: _, name, input } = value else {
            return Err(ClaudeError::Conversion("not tool use block".into()));
        };

        match name.as_str() {
            tools::literals::SEND_MESSAGE_NAME => {
                let message_content = input
                    .get(tools::literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ClaudeError::Conversion(format!(
                            "missing argument {}",
                            tools::literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME
                        ))
                    })?;

                Ok(Action::SendMessage(message_content.to_string()))
            }
            tools::literals::REACT_TO_MESSAGE_NAME => {
                let emoji = input
                    .get(tools::literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ClaudeError::Conversion(format!(
                            "missing argument {}",
                            tools::literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME
                        ))
                    })?;

                Ok(Action::ReactToMessage(serenity::ReactionType::Unicode(
                    emoji.to_string(),
                )))
            }
            tools::literals::SKIP_RESPONSE_NAME => Ok(Action::Pass),
            _ => Err(ClaudeError::Conversion(format!(
                "unsupported tool use '{name}'"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::claude::ClaudeError;
    use crate::claude::tools::literals;

    use super::Action;
    use anthropic::types::ContentBlock;
    use poise::serenity_prelude::ReactionType;

    fn tool_use(name: &str, input: serde_json::Value) -> ContentBlock {
        ContentBlock::ToolUse {
            id: "fake_id".to_string(),
            name: name.to_string(),
            input,
        }
    }

    #[test]
    fn into_send_message() {
        let input = json!({
          literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME: "message content here",
        });
        let cb = tool_use(literals::SEND_MESSAGE_NAME, input);
        let res = Action::try_from(cb).unwrap();

        assert_eq!(res, Action::SendMessage("message content here".to_string()));
    }

    #[test]
    fn into_react() {
        let input = json!({
          literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME: "❤️",
        });
        let cb = tool_use(literals::REACT_TO_MESSAGE_NAME, input);
        let res = Action::try_from(cb).unwrap();

        assert_eq!(
            res,
            Action::ReactToMessage(ReactionType::Unicode("❤️".to_string()))
        );
    }

    #[test]
    fn into_skip_response() {
        let input = json!({});
        let cb = tool_use(literals::SKIP_RESPONSE_NAME, input);
        let res = Action::try_from(cb).unwrap();

        assert_eq!(res, Action::Pass);
    }

    #[test]
    fn err_non_tool_use() {
        let cb = ContentBlock::Text {
            text: "hi".to_string(),
        };
        let res = Action::try_from(cb);

        assert!(matches!(res, Err(ClaudeError::Conversion(_))));
    }

    #[test]
    fn err_non_supported_tool() {
        let input = json!({});
        let cb = tool_use("other_tool_name", input);
        let res = Action::try_from(cb);

        assert!(matches!(res, Err(ClaudeError::Conversion(_))));
    }
}
