use serde::{Serialize, Serializer};

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    ContentBlocks(Vec<ContentBlock>),
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ContentBlock {
    Text(TextBlock),
    ImageBlock(ImageBlock),
}

#[derive(Debug)]
pub struct TextBlock {
    pub text: String,
}

#[derive(Debug)]
pub struct ImageBlock {
    pub url: String,
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
                "type": "url",
                "url": &self.url,
            }),
        )?;
        map.end()
    }
}
