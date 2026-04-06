use serde::Serialize;
use serde_json::json;

pub mod literals {
    pub const SEND_MESSAGE_NAME: &str = "send_message";
    pub const SEND_MESSAGE_CONTENT_ARGUMENT_NAME: &str = "message_content";
    pub const REACT_TO_MESSAGE_NAME: &str = "react_to_message";
    pub const REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME: &str = "emoji";
    pub const SKIP_RESPONSE_NAME: &str = "skip_response";
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl ToolDefinition {
    pub fn get_tools() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: String::from(literals::SEND_MESSAGE_NAME),
                description: String::from(
                    "Sends a message in the current Discord text channel. Use this tool when you want to send a message. The `message_content` defines the text that will be included in the message.",
                ),
                input_schema: json!({
                  "type": "object",
                  "properties": {
                    literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME: {
                      "type": "string",
                      "description": "The text to use for the Discord message body",
                    },
                  },
                  "required": [literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME],
                }),
            },
            ToolDefinition {
                name: String::from(literals::REACT_TO_MESSAGE_NAME),
                description: String::from(
                    "React to the most recent message with an emoji. Use this tool when you want to react to a Discord message. The `emoji` parameter define what emoji to use for the reaction. The `emoji` parameter should contain a single, valid, emoji like '😅' or '😅'",
                ),
                input_schema: json!({
                  "type": "object",
                  "properties": {
                    literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME: {
                      "type": "string",
                      "description": "Emoji to react with (e.g., '❤️', '👍', '🤔')",
                    },
                  },
                  "required": [literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME],
                }),
            },
            ToolDefinition {
                name: String::from(literals::SKIP_RESPONSE_NAME),
                description: String::from(
                    "Don't respond to this message. Use this tool when you would not like to respond.",
                ),
                input_schema: json!({
                  "type": "object",
                  "properties": null,
                  "required": [],
                }),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::ToolDefinition;
    use super::literals;
    use serde_json::json;

    #[test]
    fn tool_serialization_with_properties() {
        let send_message =
            serde_json::to_value(ToolDefinition::get_tools().first().unwrap().clone()).unwrap();

        let json = json!({
            "name": literals::SEND_MESSAGE_NAME,
            "description": "Sends a message in the current Discord text channel. Use this tool when you want to send a message. The `message_content` defines the text that will be included in the message.",
            "input_schema": {
              "type": "object",
              "properties": {
                literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME: {
                  "type": "string",
                  "description": "The text to use for the Discord message body",
                },
              },
              "required": [literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME],
            },
        });

        assert_eq!(send_message, json);
    }

    #[test]
    fn tool_serialization_with_no_properties() {
        let skip_response =
            serde_json::to_value(ToolDefinition::get_tools().get(2).unwrap().clone()).unwrap();

        let json = json!({
            "name": literals::SKIP_RESPONSE_NAME,
            "description": "Don't respond to this message. Use this tool when you would not like to respond.",
            "input_schema": {
              "type": "object",
              "properties": null,
              "required": [],
            },
        });

        assert_eq!(skip_response, json);
    }
}
