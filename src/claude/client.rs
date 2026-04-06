use super::model::Model;
use super::response::Response;
use super::system_prompt::SYSTEM_PROMPT;
use super::tools;
use crate::claude;
use anthropic::AnthropicError;
use anthropic::types::Message as AnthropicMessage;
use anthropic::types::SystemPrompt;
use anthropic::types::Tool;
use std::num::NonZeroU32;
use thiserror::Error;

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
        todo!("Convert `GetResponse` to anthropic types")
    }
}

#[derive(Debug, Error)]
pub enum ClaudeError {
    #[error("Couldn't build Client ({0})")]
    ClientBuilder(AnthropicError),
    #[error("Couldn't build Request ({0})")]
    RequestBuilder(AnthropicError),
    #[error("Couldn't send Request ({0})")]
    RequestFailed(AnthropicError),
    #[error("Couldn't convert from Anthropic type ({0})")]
    Conversion(String),
}

#[derive(Clone)]
pub struct Client {
    system_prompt: SystemPrompt,
    max_tokens: NonZeroU32,
    tools: Vec<Tool>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub async fn get_response(
        &self,
        messages: &[AnthropicMessage],
        api_key: &str,
        model: &Model,
    ) -> Result<Response, ClaudeError> {
        let client = anthropic::ClientBuilder::new()
            .api_key(api_key)
            .build()
            .map_err(ClaudeError::ClientBuilder)?;

        let request = anthropic::types::MessagesRequestBuilder::new(
            model.id(),
            messages.into(),
            self.max_tokens.into(),
        )
        .system(self.system_prompt.clone())
        .tools(self.tools.clone())
        .build()
        .map_err(ClaudeError::RequestBuilder)?;

        Ok(client
            .messages(request)
            .await
            .map_err(ClaudeError::RequestFailed)?
            .into())
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            max_tokens: NonZeroU32::new(2048).unwrap(),
            system_prompt: SystemPrompt::Text(SYSTEM_PROMPT.to_string()),
            tools: tools::get_tools(),
        }
    }
}
