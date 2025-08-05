use serde::Serialize;
use serde_json::json;

#[derive(Clone, Debug, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl Tool {
    pub fn get_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: String::from("send_message"),
                description: String::from(
                    "Sends a message in the current Discord text channel. Use this tool when you want to send a message. The `message_content` defines the text that will be included in the message.",
                ),
                input_schema: json!({
                  "type": "object",
                  "properties": {
                    "message_content": {
                      "type": "string",
                      "description": "The text to use for the Discord message body",
                    },
                  },
                  "required": ["message_content"],
                }),
            },
            Tool {
                name: String::from("react_to_message"),
                description: String::from(
                    "React to the most recent message with an emoji. Use this tool when you want to react to a Discord message. The `emoji` parameter define what emoji to use for the reaction. The `emoji` parameter should contain a single, valid, emoji like '😅' or '😅'",
                ),
                input_schema: json!({
                  "type": "object",
                  "properties": {
                    "emoji": {
                      "type": "string",
                      "description": "Emoji to react with (e.g., '❤️', '👍', '🤔')",
                    },
                  },
                  "required": ["emoji"],
                }),
            },
            Tool {
                name: String::from("skip_response"),
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
    use super::Tool;
    use serde_json::json;

    #[test]
    fn tool_serialization_with_properties() {
        let send_message =
            serde_json::to_value(Tool::get_tools().first().unwrap().clone()).unwrap();

        let json = json!({
            "name": "send_message",
            "description": "Sends a message in the current Discord text channel. Use this tool when you want to send a message. The `message_content` defines the text that will be included in the message.",
            "input_schema": {
              "type": "object",
              "properties": {
                "message_content": {
                  "type": "string",
                  "description": "The text to use for the Discord message body",
                },
              },
              "required": ["message_content"],
            },
        });

        assert_eq!(send_message, json);
    }

    #[test]
    fn tool_serialization_with_no_properties() {
        let skip_response =
            serde_json::to_value(Tool::get_tools().get(2).unwrap().clone()).unwrap();

        let json = json!({
            "name": "skip_response",
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
