use super::Message;
use super::ToolDefinition;
use super::model::Model;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use std::num::NonZeroU64;

#[derive(Debug, Serialize)]
pub struct Request {
    model: Model,
    system: String,
    max_tokens: NonZeroU64,
    tool_choice: Value,
    tools: Vec<ToolDefinition>,
    messages: Vec<Message>,
}

impl Request {
    pub fn new(
        model: Model,
        system_prompt: String,
        max_tokens: NonZeroU64,
        tools: Vec<ToolDefinition>,
        messages: Vec<Message>,
    ) -> Self {
        Self {
            model,
            system: system_prompt,
            max_tokens,
            tool_choice: json!({"type": "any"}),
            tools,
            messages,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::num::NonZeroU64;

    use super::Message;
    use super::Model;
    use super::Request;
    use super::ToolDefinition;
    use crate::claude::message::{Content, ContentBlock, ImageBlock, MediaType, Role, TextBlock};

    #[test]
    fn one_tool_one_message() {
        let skip_response_tool = ToolDefinition::get_tools().get(2).unwrap().clone();

        let request = serde_json::to_value(Request {
            model: Model::Sonnet4,
            system: "system prompt".to_string(),
            max_tokens: NonZeroU64::new(1024).unwrap(),
            tool_choice: json!({"type": "any"}),
            tools: vec![skip_response_tool],
            messages: vec![Message {
                role: Role::User,
                content: Content::Text("hello world".to_string()),
            }],
        })
        .unwrap();

        let json = json!({
            "model": "claude-sonnet-4-0",
            "system": "system prompt",
            "max_tokens": 1024,
            "tool_choice": {"type": "any"},
            "tools": [
              {
                "name": "skip_response",
                "description": "Don't respond to this message. Use this tool when you would not like to respond.",
                "input_schema": {
                  "type": "object",
                  "properties": null,
                  "required": [],
                },
              },
            ],
            "messages": [
              {
                "role": "user",
                "content": "hello world",
              }
            ],
        });

        assert_eq!(request, json);
    }

    #[test]
    fn two_tools_two_messages() {
        let message_and_react_tools = ToolDefinition::get_tools().into_iter().take(2);

        let request = serde_json::to_value(Request {
            model: Model::Sonnet37,
            system: "complicated system prompt".to_string(),
            max_tokens: NonZeroU64::new(1024).unwrap(),
            tool_choice: json!({"type": "any"}),
            tools: message_and_react_tools.collect(),
            messages: vec![Message {
                role: Role::User,
                content: Content::ContentBlocks(vec![
                    ContentBlock::Text(TextBlock {
                        text: "hello text block".to_string(),
                    }),
                    ContentBlock::ImageBlock(ImageBlock {
                        media_type: MediaType::Gif,
                        data: "base 64 string".to_string(),
                    }),
                ]),
            }],
        })
        .unwrap();

        let json = json!({
            "model": "claude-3-7-sonnet-latest",
            "system": "complicated system prompt",
            "max_tokens": 1024,
            "tool_choice": {"type": "any"},
            "tools": [
              {
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
              },
              {
                "name": "react_to_message",
                "description": "React to the most recent message with an emoji. Use this tool when you want to react to a Discord message. The `emoji` parameter define what emoji to use for the reaction. The `emoji` parameter should contain a single, valid, emoji like '😅' or '😅'",
                "input_schema": {
                  "type": "object",
                  "properties": {
                    "emoji": {
                      "type": "string",
                      "description": "Emoji to react with (e.g., '❤️', '👍', '🤔')",
                    },
                  },
                  "required": ["emoji"],
                },
              },
            ],
            "messages": [
              {
                "role": "user",
                "content": [
                  {
                    "type": "text",
                    "text": "hello text block",
                  },
                  {
                    "type": "image",
                    "source": {
                      "type": "base64",
                      "media_type": "image/gif",
                      "data": "base 64 string"
                    },
                  },
                ],
              },
            ],
        });

        assert_eq!(request, json);
    }
}
