use super::model::Model;
use super::system_prompt::SYSTEM_PROMPT;
use std::num::NonZeroU64;

pub struct Client {
    http: reqwest::Client,
    system_prompt: String,
    anthropic_version: String,
    model: Model,
    max_tokens: NonZeroU64,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            http: reqwest::Client::new(),
            anthropic_version: String::from("2023-06-01"),
            model: Model::Sonnet4,
            max_tokens: NonZeroU64::new(2048).unwrap(),
            system_prompt: SYSTEM_PROMPT.to_string(),
        }
    }
}
