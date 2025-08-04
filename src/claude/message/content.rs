use serde::{Serialize, Serializer};

#[derive(Serialize)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    ContentBlocks(Vec<ContentBlock>),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ContentBlock {
    Text(TextBlock),
    ImageBlock(ImageBlock),
}

pub struct TextBlock {
    pub text: String,
}

pub struct ImageBlock {
    pub media_type: MediaType,
    // TODO: Base64T?
    pub data: String,
}

#[derive(Serialize)]
pub enum MediaType {
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/webp")]
    Webp,
}

impl Serialize for TextBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("type", "text")?;
        map.serialize_entry("text", &self.text)?;
        map.end()
    }
}

impl Serialize for ImageBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("type", "image")?;
        map.serialize_entry(
            "source",
            &serde_json::json!({
                "type": "base64",
                "media_type": &self.media_type,
                "data": &self.data,
            }),
        )?;
        map.end()
    }
}
