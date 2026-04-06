use super::definitions::literals;
use const_format::concatcp;
use serde::Deserialize;

#[derive(Debug, Eq, PartialEq)]
enum ToolUse {
    SendMessage(String),
    ReactToMessage(String),
    SkipResponse,
}

impl<'de> Deserialize<'de> for ToolUse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;

        let value = Value::deserialize(deserializer)?;

        let Some(type_str) = value.get("type") else {
            return Err(D::Error::missing_field("type"));
        };

        if type_str.ne("tool_use") {
            return Err(D::Error::unknown_field(
                &type_str.to_string(),
                &["tool_use"],
            ));
        }

        let Some(name) = value.get("name").and_then(|n| n.as_str()) else {
            return Err(D::Error::missing_field("name"));
        };

        match name {
            literals::SEND_MESSAGE_NAME => {
                let field = concatcp!("input", ".", literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME,);

                Ok(ToolUse::SendMessage(
                    value
                        .get("input")
                        .and_then(|input| input.get(literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME))
                        .and_then(|v| v.as_str())
                        .filter(|v| !v.is_empty())
                        .ok_or_else(|| D::Error::missing_field(field))?
                        .to_string(),
                ))
            }
            literals::REACT_TO_MESSAGE_NAME => {
                let field =
                    concatcp!("input", ".", literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME,);

                Ok(ToolUse::ReactToMessage(
                    value
                        .get("input")
                        .and_then(|input| input.get(literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME))
                        .and_then(|v| v.as_str())
                        .filter(|v| !v.is_empty())
                        .ok_or_else(|| D::Error::missing_field(field))?
                        .to_string(),
                ))
            }
            literals::SKIP_RESPONSE_NAME => Ok(ToolUse::SkipResponse),
            _ => Err(D::Error::unknown_variant(
                name,
                &[
                    literals::SEND_MESSAGE_NAME,
                    literals::REACT_TO_MESSAGE_NAME,
                    literals::SKIP_RESPONSE_NAME,
                ],
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::{from_value, json};

    #[test]
    fn parses_send_message() {
        let v = json!(
          {
            "id": "id_string_here",
            "input": {
              "message_content": "Message content text",
            },
            "name": literals::SEND_MESSAGE_NAME,
            "type": "tool_use",
          }
        );

        assert_eq!(
            from_value::<ToolUse>(v).unwrap(),
            ToolUse::SendMessage("Message content text".into())
        );
    }
    #[test]
    fn err_on_empty_send_message_argument() {
        let v = json!(
          {
            "id": "id_string_here",
            "input": {
              literals::SEND_MESSAGE_CONTENT_ARGUMENT_NAME: "",
            },
            "name": literals::SEND_MESSAGE_NAME,
            "type": "tool_use",
          }
        );

        let e = from_value::<ToolUse>(v).err().unwrap();
        assert!(e.to_string().contains("missing field"));
        assert_eq!(e.classify(), serde_json::error::Category::Data);
    }

    #[test]
    fn parses_react_to_message() {
        let v = json!(
          {
            "id": "id_string_here",
            "input": {
              literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME: "❤️",
            },
            "name": literals::REACT_TO_MESSAGE_NAME,
            "type": "tool_use",
          }
        );

        assert_eq!(
            from_value::<ToolUse>(v).unwrap(),
            ToolUse::ReactToMessage("❤️".into())
        );
    }

    #[test]
    fn err_on_empty_react_to_message_argument() {
        let v = json!(
          {
            "id": "id_string_here",
            "input": {
              literals::REACT_TO_MESSAGE_EMOJI_ARGUMENT_NAME: "",
            },
            "name": literals::REACT_TO_MESSAGE_NAME,
            "type": "tool_use",
          }
        );

        let e = from_value::<ToolUse>(v).err().unwrap();
        assert!(e.to_string().contains("missing field"));
        assert_eq!(e.classify(), serde_json::error::Category::Data);
    }

    #[test]
    fn parses_skip_response() {
        let v = json!(
          {
            "id": "id_string_here",
            "input": null,
            "name": literals::SKIP_RESPONSE_NAME,
            "type": "tool_use",
          }
        );

        assert_eq!(from_value::<ToolUse>(v).unwrap(), ToolUse::SkipResponse);
    }

    #[test]
    fn err_on_type_field_missing() {
        let v = json!(
          {
            "id": "id_string_here",
            "input": null,
            "name": literals::SKIP_RESPONSE_NAME,
          }
        );

        let e = from_value::<ToolUse>(v).err().unwrap();
        assert!(e.to_string().contains("missing field"));
        assert_eq!(e.classify(), serde_json::error::Category::Data);
    }

    #[test]
    fn err_on_type_field_mismatch() {
        let v = json!(
          {
            "id": "id_string_here",
            "input": null,
            "name": literals::SKIP_RESPONSE_NAME,
            "type": "other_type",
          }
        );

        let e = from_value::<ToolUse>(v).err().unwrap();
        assert!(e.to_string().contains("unknown field"));
        assert_eq!(e.classify(), serde_json::error::Category::Data);
    }
}
