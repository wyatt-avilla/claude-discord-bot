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
            url: "url goes here".to_string(),
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
                "type": "url",
                "url": "url goes here",
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
                url: "url goes here".to_string(),
            })]),
        },
        Message {
            role: Role::User,
            content: Content::ContentBlocks(vec![ContentBlock::ImageBlock(ImageBlock {
                url: "url goes here".to_string(),
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
                "type": "url",
                "url": "url goes here",
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
                "type": "url",
                "url": "url goes here",
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
                url: "url goes here".to_string(),
            }),
            ContentBlock::ImageBlock(ImageBlock {
                url: "url goes here".to_string(),
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
                "type": "url",
                "url": "url goes here",
              },
            },
            {
              "type": "image",
              "source": {
                "type": "url",
                "url": "url goes here",
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
                url: "url goes here".to_string(),
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
                "type": "url",
                "url": "url goes here",
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
