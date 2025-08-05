use super::model::Model;
use super::response::Response;
use super::system_prompt::SYSTEM_PROMPT;
use super::tools::ToolDefinition;
use futures::future::join_all;
use std::num::NonZeroU64;
use thiserror::Error;

use poise::serenity_prelude as serenity;

const MESSAGE_API_URL: &str = "https://api.anthropic.com/v1/messages";

#[derive(Debug, Error)]
pub enum ClaudeError {
    #[error("Couldn't send HTTP request ({0})")]
    Http(reqwest::Error),
    #[error("Couldn't deserialize response ({0})")]
    Parse(reqwest::Error),
}

pub struct Client {
    http: reqwest::Client,
    system_prompt: String,
    anthropic_version: String,
    model: Model,
    max_tokens: NonZeroU64,
    tools: Vec<ToolDefinition>,
}

impl Client {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            ..Default::default()
        }
    }

    pub async fn get_response(
        &self,
        msgs: Vec<serenity::Message>,
        ctx: &serenity::Context,
        api_key: &str,
    ) -> Result<Response, ClaudeError> {
        let msgs = join_all(
            msgs.into_iter()
                .map(async move |m| super::Message::from(&m, ctx).await),
        )
        .await;

        let request = super::Request::new(
            self.model.clone(),
            self.system_prompt.clone(),
            self.max_tokens,
            self.tools.clone(),
            msgs,
        );

        self.http
            .post(MESSAGE_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", self.anthropic_version.clone())
            .json(&request)
            .send()
            .await
            .map_err(ClaudeError::Http)?
            .json()
            .await
            .map_err(ClaudeError::Parse)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            http: reqwest::Client::new(),
            anthropic_version: String::from("2023-06-01"),
            model: Model::Sonnet4,
            max_tokens: NonZeroU64::new(2048).unwrap(),
            system_prompt: SYSTEM_PROMPT.to_string(),
            tools: ToolDefinition::get_tools(),
        }
    }
}
