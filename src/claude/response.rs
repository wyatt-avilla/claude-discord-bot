use super::tools;
use const_format::concatcp;
use serde::Deserialize;

use poise::serenity_prelude as serenity;

#[derive(Debug, Eq, PartialEq)]
pub enum Action {
    SendMessage(String),
    ReactToMessage(serenity::ReactionType),
    Pass,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Response {
    pub stop_reason: StopReason,
    pub usage: Usage,
    pub content: Vec<Action>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    PauseTurn,
    Refusal,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;

        let value = Value::deserialize(deserializer)?;

        match value.get("type").and_then(|v| v.as_str()) {
            Some("tool_use") => {
                let name = value
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| D::Error::missing_field("name"))?;

                match name {
                    tools::literals::SEND_MESSAGE_NAME => {
                        let message_content = value
                            .get("input")
                            .and_then(|input| {
                                input.get(tools::literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME)
                            })
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                D::Error::missing_field(concatcp!(
                                    "input",
                                    ".",
                                    tools::literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME,
                                ))
                            })?;

                        Ok(Action::SendMessage(message_content.to_string()))
                    }
                    tools::literals::REACT_TO_MESSAGE_NAME => {
                        let emoji = value
                            .get("input")
                            .and_then(|input| {
                                input.get(tools::literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME)
                            })
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                D::Error::missing_field(concatcp!(
                                    "input",
                                    ".",
                                    tools::literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME,
                                ))
                            })?;

                        Ok(Action::ReactToMessage(serenity::ReactionType::Unicode(
                            emoji.to_string(),
                        )))
                    }
                    tools::literals::SKIP_RESPONSE_NAME => Ok(Action::Pass),
                    _ => Err(D::Error::unknown_variant(
                        name,
                        &[
                            tools::literals::SEND_MESSAGE_NAME,
                            tools::literals::REACT_TO_MESSAGE_NAME,
                            tools::literals::SKIP_RESPONSE_NAME,
                        ],
                    )),
                }
            }
            Some("text") => {
                let text = value
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| D::Error::missing_field("text"))?;

                Ok(Action::SendMessage(text.to_string()))
            }
            Some(other) => Err(D::Error::unknown_variant(other, &["tool_use", "text"])),
            None => Err(D::Error::missing_field("type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json};

    use super::{Action, Response, StopReason, Usage};
    use poise::serenity_prelude::ReactionType;

    #[test]
    fn one_tool_call() {
        let response = json!({
            "content": [
              {
                "id": "id_string_here",
                "input": {
                  "message_content": "Message content text",
                },
                "name": "send_message",
                "type": "tool_use",
              },
            ],
            "id": "id_string",
            "model": "claude-model",
            "role": "assistant",
            "stop_reason": "tool_use",
            "stop_sequence": null,
            "type": "message",
            "usage": {
              "cache_creation_input_tokens": 10,
              "cache_read_input_tokens": 0,
              "input_tokens": 33,
              "output_tokens": 44,
              "service_tier": "standard",
            },
        });

        let response_struct = Response {
            stop_reason: StopReason::ToolUse,
            usage: Usage {
                input_tokens: 33,
                output_tokens: 44,
            },
            content: vec![Action::SendMessage("Message content text".to_string())],
        };

        assert_eq!(response_struct, from_value(response).unwrap());
    }

    #[test]
    fn two_tool_calls_one_message() {
        let response = json!({
            "content": [
              {
                "id": "id_string_here",
                "input": {
                  "message_content": "Message content text",
                },
                "name": "send_message",
                "type": "tool_use",
              },
              {
                "id": "id_string_here",
                "input": {
                  "emoji": "❤️",
                },
                "name": "react_to_message",
                "type": "tool_use",
              },
              {
                "type": "text",
                "text": "hello text",
              },
              {
                "id": "id_string_here",
                "input": null,
                "name": "skip_response",
                "type": "tool_use",
              },
            ],
            "id": "id_string",
            "model": "claude-model",
            "role": "assistant",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "type": "message",
            "usage": {
              "cache_creation_input_tokens": 10,
              "cache_read_input_tokens": 0,
              "input_tokens": 33,
              "output_tokens": 44,
              "service_tier": "standard",
            },
        });

        let response_struct = Response {
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 33,
                output_tokens: 44,
            },
            content: vec![
                Action::SendMessage("Message content text".to_string()),
                Action::ReactToMessage(ReactionType::Unicode("❤️".to_string())),
                Action::SendMessage("hello text".to_string()),
                Action::Pass,
            ],
        };

        assert_eq!(response_struct, from_value(response).unwrap());
    }
}
