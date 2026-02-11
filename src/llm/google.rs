//! Google Gemini Chat Completions API provider implementation.
//!
//! This provider uses the Google AI Studio / Google Cloud OpenAI-compatible 
//! chat completions API with API key authentication.
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::config::GoogleConfig;
use crate::error::LlmError;
use crate::llm::provider::{
    ChatMessage, CompletionRequest, CompletionResponse, FinishReason, LlmProvider, Role, ToolCall,
    ToolCompletionRequest, ToolCompletionResponse,
};

/// Google Gemini Chat Completions API provider.
pub struct GoogleGeminiProvider {
    client: Client,
    config: GoogleConfig,
}

impl GoogleGeminiProvider {
    /// Create a new Google Gemini provider with API key auth.
    pub fn new(config: GoogleConfig) -> Result<Self, LlmError> {
        if config.api_key.is_none() {
            return Err(LlmError::AuthFailed {
                provider: "google".to_string(),
            });
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());

        Ok(Self { client, config })
    }

    fn api_url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn api_key(&self) -> String {
        self.config
            .api_key
            .as_ref()
            .map(|k| k.expose_secret().to_string())
            .unwrap_or_default()
    }

    /// Send a request to the chat completions API.
    async fn send_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        body: &T,
    ) -> Result<R, LlmError> {
        let url = self.api_url("chat/completions");

        tracing::debug!("Sending request to Google Gemini: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("x-goog-api-key", self.api_key())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Google Gemini request failed: {}", e);
                LlmError::RequestFailed {
                    provider: "google".to_string(),
                    reason: e.to_string(),
                }
            })?;

        let status = response.status();
        let headers = response.headers().clone();
        let response_text = response.text().await.unwrap_or_default();

        tracing::debug!("Google Gemini response status: {}", status);
        tracing::debug!("Google Gemini response body: {}", response_text);

        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(LlmError::AuthFailed {
                    provider: "google".to_string(),
                });
            }
            if status.as_u16() == 429 {
                let retry_after = headers
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(Duration::from_secs);

                return Err(LlmError::RateLimited {
                    provider: "google".to_string(),
                    retry_after,
                });
            }
            return Err(LlmError::RequestFailed {
                provider: "google".to_string(),
                reason: format!("HTTP {}: {}", status, response_text),
            });
        }

        serde_json::from_str(&response_text).map_err(|e| LlmError::InvalidResponse {
            provider: "google".to_string(),
            reason: format!("JSON parse error: {}. Raw: {}", e, response_text),
        })
    }
}

#[async_trait]
impl LlmProvider for GoogleGeminiProvider {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let messages: Vec<ChatCompletionMessage> =
            req.messages.into_iter().map(|m| m.into()).collect();

        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            tools: None,
            tool_choice: None,
        };

        let response: ChatCompletionResponse = self.send_request(&request).await?;

        let choice =
            response
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| LlmError::InvalidResponse {
                    provider: "google".to_string(),
                    reason: "No choices in response".to_string(),
                })?;

        let content = choice.message.content.unwrap_or_default();
        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolUse,
            Some("content_filter") => FinishReason::ContentFilter,
            _ => FinishReason::Unknown,
        };

        Ok(CompletionResponse {
            content,
            thought: choice.message.thought,
            finish_reason,
            input_tokens: response.usage.prompt_tokens,
            output_tokens: response.usage.completion_tokens,
        })
    }

    async fn complete_with_tools(
        &self,
        req: ToolCompletionRequest,
    ) -> Result<ToolCompletionResponse, LlmError> {
        let messages: Vec<ChatCompletionMessage> =
            req.messages.into_iter().map(|m| m.into()).collect();

        let tools: Vec<ChatCompletionTool> = req
            .tools
            .into_iter()
            .map(|t| ChatCompletionTool {
                tool_type: "function".to_string(),
                function: ChatCompletionFunction {
                    name: t.name,
                    description: Some(t.description),
                    parameters: Some(t.parameters),
                },
            })
            .collect();

        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            tools: if tools.is_empty() { None } else { Some(tools) },
            tool_choice: req.tool_choice,
        };

        let response: ChatCompletionResponse = self.send_request(&request).await?;

        let choice =
            response
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| LlmError::InvalidResponse {
                    provider: "google".to_string(),
                    reason: "No choices in response".to_string(),
                })?;

        let content = choice.message.content;
        let tool_calls: Vec<ToolCall> = choice
            .message
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tc| {
                let arguments = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(serde_json::Value::Object(Default::default()));
                ToolCall {
                    id: tc.id,
                    name: tc.function.name,
                    arguments,
                    thought_signature: tc.extra_content.and_then(|e| e.google.thought_signature),
                }
            })
            .collect();

        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolUse,
            Some("content_filter") => FinishReason::ContentFilter,
            _ => {
                if !tool_calls.is_empty() {
                    FinishReason::ToolUse
                } else {
                    FinishReason::Unknown
                }
            }
        };

        Ok(ToolCompletionResponse {
            content,
            tool_calls,
            thought: choice.message.thought,
            finish_reason,
            input_tokens: response.usage.prompt_tokens,
            output_tokens: response.usage.completion_tokens,
        })
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    fn cost_per_token(&self) -> (Decimal, Decimal) {
        // Free tier for many Gemini models, or very low cost
        (dec!(0.000000), dec!(0.000000))
    }

    async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        // Direct listing not always available on the OpenAI compat endpoint
        Ok(vec![self.config.model.clone()])
    }
}

// OpenAI-compatible Chat Completions API types (re-implemented for local use)

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ChatCompletionTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ChatCompletionToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,
}

impl From<ChatMessage> for ChatCompletionMessage {
    fn from(msg: ChatMessage) -> Self {
        let role = match msg.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };
        let tool_calls = msg.tool_calls.map(|calls| {
            calls
                .into_iter()
                .map(|tc| ChatCompletionToolCall {
                    id: tc.id.clone(),
                    call_type: "function".to_string(),
                    function: ChatCompletionToolCallFunction {
                        name: tc.name.clone(),
                        arguments: tc.arguments.to_string(),
                    },
                    extra_content: tc.thought_signature.map(|sig| ExtraContent {
                        google: GoogleExtraContent {
                            thought_signature: Some(sig),
                        },
                    }),
                })
                .collect()
        });
        Self {
            role: role.to_string(),
            content: Some(msg.content),
            tool_call_id: msg.tool_call_id,
            name: msg.name,
            tool_calls,
            thought: msg.thought,
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ChatCompletionFunction,
}

#[derive(Debug, Serialize)]
struct ChatCompletionFunction {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    #[allow(dead_code)]
    id: String,
    choices: Vec<ChatCompletionChoice>,
    usage: ChatCompletionUsage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponseMessage {
    #[allow(dead_code)]
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<ChatCompletionToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionToolCall {
    id: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    call_type: String,
    function: ChatCompletionToolCallFunction,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_content: Option<ExtraContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExtraContent {
    google: GoogleExtraContent,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleExtraContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    thought_signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionToolCallFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}
