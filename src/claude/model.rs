use bincode::{Decode, Encode};
use clap::ValueEnum;
use poise::ChoiceParameter;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, ValueEnum, Deserialize, Encode, Decode, ChoiceParameter)]
pub enum Model {
    #[name = "Opus 4"]
    #[value(name = "opus-4")]
    Opus4,
    #[name = "Sonnet 4"]
    #[value(name = "sonnet-4")]
    Sonnet4,
    #[name = "Sonnet 3.7"]
    #[value(name = "sonnet-3.7")]
    Sonnet37,
    #[name = "Sonnet 3.5"]
    #[value(name = "sonnet-3.5")]
    Sonnet35,
    #[name = "Haiku 3.5"]
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

    pub fn pretty_name(&self) -> String {
        String::from(match self {
            Model::Opus4 => "Opus 4",
            Model::Sonnet4 => "Sonnet 4",
            Model::Sonnet37 => "Sonnet 3.7",
            Model::Sonnet35 => "Sonnet 3.5",
            Model::Haiku35 => "Haiku 3.5",
        })
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::Sonnet4
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
