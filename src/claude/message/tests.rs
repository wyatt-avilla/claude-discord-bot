use super::*;

#[test]
fn one_text_message() {
    let msgs = serde_json::to_value(vec![Message {
        role: Role::User,
        content: Content::Text("hello world".to_string()),
    }])
    .unwrap();

    let json = serde_json::json!([{
        "role": "user",
        "content": "hello world",
    }]);

    assert_eq!(msgs, json);
}

#[test]
fn two_text_message() {
    let msgs = serde_json::to_value(vec![
        Message {
            role: Role::User,
            content: Content::Text("hello claude".to_string()),
        },
        Message {
            role: Role::Assistant,
            content: Content::Text("hello".to_string()),
        },
    ])
    .unwrap();

    let json = serde_json::json!([
        {
          "role": "user",
          "content": "hello claude",
        },
        {
          "role": "assistant",
          "content": "hello",
        },
    ]);

    assert_eq!(msgs, json);
}

#[test]
fn one_image_message() {
    let msgs = serde_json::to_value(vec![Message {
        role: Role::User,
        content: Content::ContentBlocks(vec![ContentBlock::ImageBlock(ImageBlock {
            media_type: MediaType::Jpeg,
            data: "base64_encoded_string".to_string(),
        })]),
    }])
    .unwrap();

    let json = serde_json::json!([
        {
          "role": "user",
          "content": [
            {
              "type": "image",
              "source": {
                "type": "base64",
                "media_type": "image/jpeg",
                "data": "base64_encoded_string",
              },
            }
          ],
        },
    ]);

    assert_eq!(msgs, json);
}

#[test]
fn two_image_messages() {
    let msgs = serde_json::to_value(vec![
        Message {
            role: Role::User,
            content: Content::ContentBlocks(vec![ContentBlock::ImageBlock(ImageBlock {
                media_type: MediaType::Jpeg,
                data: "base64_encoded_string".to_string(),
            })]),
        },
        Message {
            role: Role::User,
            content: Content::ContentBlocks(vec![ContentBlock::ImageBlock(ImageBlock {
                media_type: MediaType::Png,
                data: "base64_encoded_string2".to_string(),
            })]),
        },
    ])
    .unwrap();

    let json = serde_json::json!([
        {
          "role": "user",
          "content": [
            {
              "type": "image",
              "source": {
                "type": "base64",
                "media_type": "image/jpeg",
                "data": "base64_encoded_string",
              },
            }
          ],
        },
        {
          "role": "user",
          "content": [
            {
              "type": "image",
              "source": {
                "type": "base64",
                "media_type": "image/png",
                "data": "base64_encoded_string2",
              },
            }
          ],
        },
    ]);

    assert_eq!(msgs, json);
}

#[test]
fn one_message_two_content_blocks() {
    let msgs = serde_json::to_value(vec![Message {
        role: Role::User,
        content: Content::ContentBlocks(vec![
            ContentBlock::ImageBlock(ImageBlock {
                media_type: MediaType::Gif,
                data: "base64_encoded_string1".to_string(),
            }),
            ContentBlock::ImageBlock(ImageBlock {
                media_type: MediaType::Webp,
                data: "base64_encoded_string2".to_string(),
            }),
        ]),
    }])
    .unwrap();

    let json = serde_json::json!([
        {
          "role": "user",
          "content": [
            {
              "type": "image",
              "source": {
                "type": "base64",
                "media_type": "image/gif",
                "data": "base64_encoded_string1",
              },
            },
            {
              "type": "image",
              "source": {
                "type": "base64",
                "media_type": "image/webp",
                "data": "base64_encoded_string2",
              },
            }
          ],
        },
    ]);

    assert_eq!(msgs, json);
}

#[test]
fn mixed_content_type() {
    let msgs = serde_json::to_value(vec![Message {
        role: Role::User,
        content: Content::ContentBlocks(vec![
            ContentBlock::ImageBlock(ImageBlock {
                media_type: MediaType::Jpeg,
                data: "base64_encoded_string".to_string(),
            }),
            ContentBlock::Text(TextBlock {
                text: "this is text".to_string(),
            }),
        ]),
    }])
    .unwrap();

    let json = serde_json::json!([
        {
          "role": "user",
          "content": [
            {
              "type": "image",
              "source": {
                "type": "base64",
                "media_type": "image/jpeg",
                "data": "base64_encoded_string",
              },
            },
            {
              "type": "text",
              "text": "this is text",
            },
          ],
        },
    ]);

    assert_eq!(msgs, json);
}
