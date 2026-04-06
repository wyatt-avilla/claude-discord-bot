use super::consts;
use super::model::Model;
use super::response::Response;
use super::system_prompt::SYSTEM_PROMPT;
use super::tools;
use crate::claude;
use anthropic::types::Tool;
use std::num::NonZeroU64;
use std::sync::Arc;
use thiserror::Error;

use consts::ANTHROPIC_API_BASE_URL;

pub trait GetResponse {
    async fn get_response(
        &self,
        msgs: &[claude::Message],
        api_key: &str,
        model: &claude::Model,
    ) -> Result<claude::Response, ClaudeError>;
}

impl GetResponse for Client {
    async fn get_response(
        &self,
        msgs: &[claude::Message],
        api_key: &str,
        model: &claude::Model,
    ) -> Result<claude::Response, ClaudeError> {
        self.get_response(msgs, api_key, model).await
    }
}

#[derive(Debug, Error)]
pub enum ClaudeError {
    #[error("Couldn't send HTTP request ({0})")]
    Http(reqwest::Error),
    #[error("Couldn't deserialize response ({0})")]
    Parse(reqwest::Error),
}

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    system_prompt: Arc<String>,
    anthropic_version: Arc<String>,
    max_tokens: NonZeroU64,
    tools: Arc<Vec<Tool>>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub async fn get_response(
        &self,
        msgs: &[claude::Message],
        api_key: &str,
        model: &Model,
    ) -> Result<Response, ClaudeError> {
        let request = super::Request::new(
            model,
            &self.system_prompt,
            self.max_tokens,
            &self.tools,
            msgs,
        );

        self.http
            .post(format!("{ANTHROPIC_API_BASE_URL}/messages"))
            .header("x-api-key", api_key)
            .header("anthropic-version", self.anthropic_version.to_string())
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
            anthropic_version: consts::ANTHROPIC_API_VERSION.to_string().into(),
            max_tokens: NonZeroU64::new(2048).unwrap(),
            system_prompt: SYSTEM_PROMPT.to_string().into(),
            tools: tools::get_tools().into(),
        }
    }
}
