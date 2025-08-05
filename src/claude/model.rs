use clap::ValueEnum;
use serde::Serialize;
use std::fmt::Display;

#[derive(Clone, Debug, ValueEnum)]
pub enum Model {
    #[value(name = "opus-4")]
    Opus4,
    #[value(name = "sonnet-4")]
    Sonnet4,
    #[value(name = "sonnet-3.7")]
    Sonnet37,
    #[value(name = "sonnet-3.5")]
    Sonnet35,
    #[value(name = "haiku-3.5")]
    Haiku35,
}

impl Model {
    pub fn id(&self) -> String {
        match self {
            Model::Opus4 => "claude-opus-4-0",
            Model::Sonnet4 => "claude-sonnet-4-0",
            Model::Sonnet37 => "claude-3-7-sonnet-latest",
            Model::Sonnet35 => "claude-3-5-sonnet-latest",
            Model::Haiku35 => "claude-3-5-haiku-latest",
        }
        .to_string()
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_possible_value().unwrap().get_name())
    }
}

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.id())
    }
}
