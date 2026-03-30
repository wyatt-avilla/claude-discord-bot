use bincode::{Decode, Encode};
use clap::ValueEnum;
use poise::ChoiceParameter;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, ValueEnum, Deserialize, Encode, Decode, ChoiceParameter)]
pub enum Model {
    #[name = "Opus 4.6"]
    #[value(name = "opus-4.6")]
    Opus46,
    #[name = "Sonnet 4.6"]
    #[value(name = "sonnet-4.6")]
    Sonnet46,

    #[name = "Opus 4.5"]
    #[value(name = "opus-4.5")]
    Opus45,
    #[name = "Sonnet 4.5"]
    #[value(name = "sonnet-4.5")]
    Sonnet45,
    #[name = "Haiku 4.5"]
    #[value(name = "haiku-4.5")]
    Haiku45,

    #[name = "Opus 4.1"]
    #[value(name = "opus-4.1")]
    Opus41,

    #[name = "Opus 4"]
    #[value(name = "opus-4")]
    Opus4,
    #[name = "Sonnet 4"]
    #[value(name = "sonnet-4")]
    Sonnet4,
}

impl Model {
    pub fn id(&self) -> String {
        match self {
            Model::Opus46 => "claude-opus-4-6",
            Model::Sonnet46 => "claude-sonnet-4-6",

            Model::Opus45 => "claude-opus-4-5",
            Model::Sonnet45 => "claude-sonnet-4-5",
            Model::Haiku45 => "claude-haiku-4-5",

            Model::Opus41 => "claude-opus-4-1",

            Model::Opus4 => "claude-opus-4-0",
            Model::Sonnet4 => "claude-sonnet-4-0",
        }
        .to_string()
    }

    pub fn pretty_name(&self) -> String {
        String::from(match self {
            Model::Opus46 => "Opus 4.6",
            Model::Sonnet46 => "Sonnet 4.6",

            Model::Opus45 => "Opus 4.5",
            Model::Sonnet45 => "Sonnet 4.5",
            Model::Haiku45 => "Haiku 4.5",

            Model::Opus41 => "Opus 4.1",

            Model::Opus4 => "Opus 4",
            Model::Sonnet4 => "Sonnet 4",
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

#[cfg(test)]
mod tests {
    use super::Model;
    use crate::claude::consts::{ANTHROPIC_API_BASE_URL, ANTHROPIC_API_VERSION};
    use clap::ValueEnum;
    use poise::serenity_prelude::futures::future::join_all;
    use reqwest::header::HeaderName;

    #[cfg_attr(not(feature = "api_key_tests"), ignore = "ig")]
    #[tokio::test]
    async fn valid_model_ids() {
        let client = reqwest::ClientBuilder::new()
            .default_headers(
                [
                    (
                        HeaderName::from_static("x-api-key"),
                        std::env::var("ANTHROPIC_API_KEY").unwrap().parse().unwrap(),
                    ),
                    (
                        HeaderName::from_static("anthropic-version"),
                        ANTHROPIC_API_VERSION.parse().unwrap(),
                    ),
                ]
                .into_iter()
                .collect(),
            )
            .build()
            .unwrap();

        let response_futures = Model::value_variants()
            .iter()
            .map(super::Model::id)
            .map(|id| {
                client
                    .get(format!("{ANTHROPIC_API_BASE_URL}/models/{id}"))
                    .send()
            })
            .collect::<Vec<_>>();

        let responses = join_all(response_futures).await;

        for res in &responses {
            match res {
                Err(e) => panic!("Error sending request: {e}"),
                Ok(res) => {
                    assert!(
                        res.status() == 200,
                        "Model ID {} is invalid (status code {})",
                        res.url().path(),
                        res.status()
                    );
                }
            }
        }
    }
}
