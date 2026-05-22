//! Google Gemini Native API provider implementation.
//!
//! This provider uses the native Google Gemini API (v1beta/models/{model}:generateContent)
//! and supports Context Caching and File Uploads.
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use secrecy::ExposeSecret;
use serde_json::json;
use crate::config::GoogleConfig;
use crate::error::LlmError;
use crate::llm::provider::{
    ChatMessage, CompletionRequest, CompletionResponse, FinishReason, LlmProvider, Role, ToolCall,
    ToolCompletionRequest, ToolCompletionResponse, ToolDefinition,
};

/// Google Gemini API provider.
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

    fn api_key(&self) -> String {
        self.config
            .api_key
            .as_ref()
            .map(|k| k.expose_secret().to_string())
            .unwrap_or_default()
    }

    /// Convert generic ChatMessage to Gemini Part
    fn to_gemini_content(msg: &ChatMessage) -> serde_json::Value {
        let role = match msg.role {
            Role::System => "user", // Gemini system instructions are separate, but if inline it's user
            Role::User => "user",
            Role::Assistant => "model",
            Role::Tool => "function", // Tool response
        };

        if msg.role == Role::Tool {
            let func_name = msg.name.as_deref().unwrap_or("unknown");
            let response_val = serde_json::from_str(&msg.content)
                .unwrap_or_else(|_| json!({"result": msg.content}));
            
            return json!({
                "role": "function",
                "parts": [{
                    "functionResponse": {
                        "name": func_name,
                        "response": response_val
                    }
                }]
            });
        }

        if let Some(ref tool_calls) = msg.tool_calls {
            let parts: Vec<serde_json::Value> = tool_calls.iter().map(|tc| {
                json!({
                    "functionCall": {
                        "name": tc.name,
                        "args": tc.arguments
                    }
                })
            }).collect();
            return json!({
                "role": "model",
                "parts": parts
            });
        }

        let mut parts = Vec::new();
        if !msg.content.is_empty() {
            parts.push(json!({"text": msg.content}));
        }
        if let Some(uri) = &msg.file_uri {
            let mime = msg.mime_type.as_deref().unwrap_or("text/plain");
            parts.push(json!({
                "fileData": {
                    "fileUri": uri,
                    "mimeType": mime
                }
            }));
        }
        
        // Fallback: Gemini requires at least one part in the content block
        if parts.is_empty() {
            parts.push(json!({"text": ""}));
        }

        json!({
            "role": role,
            "parts": parts
        })
    }

    async fn send_generate_content(&self, body: serde_json::Value) -> Result<serde_json::Value, LlmError> {
        // Assume base_url is https://generativelanguage.googleapis.com
        let base_url = "https://generativelanguage.googleapis.com";
        let url = format!("{}/v1beta/models/{}:generateContent?key={}", base_url, self.config.model, self.api_key());

        tracing::debug!("Sending request to Google Gemini generateContent");

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed {
                provider: "google".to_string(),
                reason: e.to_string(),
            })?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(LlmError::AuthFailed { provider: "google".to_string() });
            }
            if status.as_u16() == 429 {
                return Err(LlmError::RateLimited {
                    provider: "google".to_string(),
                    retry_after: Some(Duration::from_secs(60)),
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

    fn parse_generate_response(&self, json: serde_json::Value) -> Result<(String, Option<String>, Vec<ToolCall>, FinishReason, u32, u32), LlmError> {
        let candidates = json.get("candidates").and_then(|c| c.as_array());
        if candidates.is_none() || candidates.unwrap().is_empty() {
            let prompt_feedback = json.get("promptFeedback");
            tracing::error!("Gemini safety block: {:?}", prompt_feedback);
            return Err(LlmError::InvalidResponse {
                provider: "google".to_string(),
                reason: "[Content Blocked by Safety Filter]".to_string(),
            });
        }

        let candidate = &candidates.unwrap()[0];
        let content = candidate.get("content");
        let finish_reason_str = candidate.get("finishReason").and_then(|s| s.as_str()).unwrap_or("");
        
        let mut text = String::new();
        let mut tool_calls = Vec::new();

        if let Some(content) = content {
            if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                for part in parts {
                    if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                        text.push_str(t);
                    }
                    if let Some(fc) = part.get("functionCall") {
                        if let Some(name) = fc.get("name").and_then(|n| n.as_str()) {
                            let args = fc.get("args").cloned().unwrap_or(json!({}));
                            tool_calls.push(ToolCall {
                                id: uuid::Uuid::new_v4().to_string(),
                                name: name.to_string(),
                                arguments: args,
                                thought_signature: None,
                            });
                        }
                    }
                }
            }
        }

        let usage = json.get("usageMetadata");
        let input_tokens = usage.and_then(|u| u.get("promptTokenCount")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let output_tokens = usage.and_then(|u| u.get("candidatesTokenCount")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let finish_reason = match finish_reason_str {
            "STOP" => FinishReason::Stop,
            "MAX_TOKENS" => FinishReason::Length,
            "SAFETY" => FinishReason::ContentFilter,
            "RECITATION" => FinishReason::ContentFilter,
            "OTHER" => FinishReason::Unknown,
            _ => {
                if !tool_calls.is_empty() { FinishReason::ToolUse } else { FinishReason::Stop }
            }
        };

        Ok((text, None, tool_calls, finish_reason, input_tokens, output_tokens))
    }
}

#[async_trait]
impl LlmProvider for GoogleGeminiProvider {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let contents: Vec<serde_json::Value> = req.messages.iter().map(Self::to_gemini_content).collect();
        
        let mut body = json!({
            "contents": contents,
        });

        if let Some(cache_id) = req.cache_id {
            body.as_object_mut().unwrap().insert("cachedContent".to_string(), json!(cache_id));
        }

        let response_json = self.send_generate_content(body).await?;
        let (content, thought, _, finish_reason, input_tokens, output_tokens) = self.parse_generate_response(response_json)?;

        Ok(CompletionResponse {
            content,
            thought,
            finish_reason,
            input_tokens,
            output_tokens,
        })
    }

    async fn complete_with_tools(
        &self,
        req: ToolCompletionRequest,
    ) -> Result<ToolCompletionResponse, LlmError> {
        let contents: Vec<serde_json::Value> = req.messages.iter().map(Self::to_gemini_content).collect();
        
        let tools_json: Vec<serde_json::Value> = req.tools.into_iter().map(|t| {
            json!({
                "functionDeclarations": [{
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters
                }]
            })
        }).collect();

        let mut body = json!({
            "contents": contents,
        });

        if let Some(cache_id) = req.cache_id {
            body.as_object_mut().unwrap().insert("cachedContent".to_string(), json!(cache_id));
        } else {
            body.as_object_mut().unwrap().insert("tools".to_string(), json!(tools_json));
        }

        let response_json = self.send_generate_content(body).await?;
        let (content, thought, tool_calls, finish_reason, input_tokens, output_tokens) = self.parse_generate_response(response_json)?;

        Ok(ToolCompletionResponse {
            content: if content.is_empty() { None } else { Some(content) },
            tool_calls,
            thought,
            finish_reason,
            input_tokens,
            output_tokens,
        })
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    fn cost_per_token(&self) -> (Decimal, Decimal) {
        (dec!(0.000000), dec!(0.000000))
    }

    async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        Ok(vec![self.config.model.clone()])
    }

    async fn create_cache(
        &self,
        ttl_seconds: i32,
        messages: Vec<ChatMessage>,
        system_instruction: Option<String>,
        tools: Vec<ToolDefinition>,
    ) -> Result<String, LlmError> {
        let base_url = "https://generativelanguage.googleapis.com";
        let url = format!("{}/v1beta/cachedContents?key={}", base_url, self.api_key());

        let contents: Vec<serde_json::Value> = messages.iter().map(Self::to_gemini_content).collect();
        
        let mut body = json!({
            "model": format!("models/{}", self.config.model),
            "contents": contents,
            "ttl": format!("{}s", ttl_seconds)
        });

        if let Some(sys_instr) = system_instruction {
            body.as_object_mut().unwrap().insert("systemInstruction".to_string(), json!({
                "parts": [{"text": sys_instr}]
            }));
        }

        if !tools.is_empty() {
            let tools_json: Vec<serde_json::Value> = tools.into_iter().map(|t| {
                json!({
                    "functionDeclarations": [{
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters
                    }]
                })
            }).collect();
            body.as_object_mut().unwrap().insert("tools".to_string(), json!(tools_json));
        }

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed {
                provider: "google".to_string(),
                reason: e.to_string(),
            })?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(LlmError::RequestFailed {
                provider: "google".to_string(),
                reason: format!("Cache creation failed: HTTP {}: {}", status, response_text),
            });
        }

        let json_resp: serde_json::Value = serde_json::from_str(&response_text).unwrap_or(json!({}));
        if let Some(name) = json_resp.get("name").and_then(|n| n.as_str()) {
            Ok(name.to_string())
        } else {
            Err(LlmError::InvalidResponse {
                provider: "google".to_string(),
                reason: format!("No name field in cache response: {}", response_text),
            })
        }
    }

    async fn delete_cache(&self, cache_id: &str) -> Result<(), LlmError> {
        let base_url = "https://generativelanguage.googleapis.com";
        let url = format!("{}/v1beta/{}?key={}", base_url, cache_id, self.api_key());

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed {
                provider: "google".to_string(),
                reason: e.to_string(),
            })?;

        if !response.status().is_success() {
            tracing::warn!("Failed to delete cache {}: {}", cache_id, response.status());
        }
        Ok(())
    }

    async fn upload_file(&self, path: &std::path::Path, mime_type: &str) -> Result<String, LlmError> {
        let base_url = "https://generativelanguage.googleapis.com";
        let url = format!("{}/upload/v1beta/files?key={}", base_url, self.api_key());

        let file_data = tokio::fs::read(path).await.map_err(|e| LlmError::RequestFailed {
            provider: "google".to_string(),
            reason: format!("Failed to read file: {}", e),
        })?;
        let file_size = file_data.len();

        let response = self
            .client
            .post(&url)
            .header("X-Goog-Upload-Protocol", "raw")
            .header("X-Goog-Upload-Command", "upload, finalize")
            .header("X-Goog-Upload-Header-Content-Length", file_size.to_string())
            .header("X-Goog-Upload-Header-Content-Type", mime_type)
            .header("Content-Type", mime_type)
            .body(file_data)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed {
                provider: "google".to_string(),
                reason: e.to_string(),
            })?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(LlmError::RequestFailed {
                provider: "google".to_string(),
                reason: format!("File upload failed: HTTP {}: {}", status, response_text),
            });
        }

        let json_resp: serde_json::Value = serde_json::from_str(&response_text).unwrap_or(json!({}));
        if let Some(uri) = json_resp.get("file").and_then(|f| f.get("uri")).and_then(|u| u.as_str()) {
            Ok(uri.to_string())
        } else {
            Err(LlmError::InvalidResponse {
                provider: "google".to_string(),
                reason: format!("No file URI in upload response: {}", response_text),
            })
        }
    }
}
