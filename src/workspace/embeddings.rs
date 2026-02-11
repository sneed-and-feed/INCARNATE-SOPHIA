//! Embedding providers for semantic search.
//!
//! Embeddings convert text into dense vectors that capture semantic meaning.
//! Similar concepts have similar vectors, enabling semantic search.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Error type for embedding operations.
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Rate limited, retry after {retry_after:?}")]
    RateLimited {
        retry_after: Option<std::time::Duration>,
    },

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Text too long: {length} > {max}")]
    TextTooLong { length: usize, max: usize },

    #[error("Initialization failed: {0}")]
    InitializationError(String),

    #[error("Execution failed: {0}")]
    ExecutionError(String),
}

impl From<reqwest::Error> for EmbeddingError {
    fn from(e: reqwest::Error) -> Self {
        EmbeddingError::HttpError(e.to_string())
    }
}

/// Trait for embedding providers.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Get the embedding dimension.
    fn dimension(&self) -> usize;

    /// Get the model name.
    fn model_name(&self) -> &str;

    /// Maximum input length in characters.
    fn max_input_length(&self) -> usize;

    /// Generate an embedding for a single text.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Generate embeddings for multiple texts (batched).
    ///
    /// Default implementation calls embed() for each text.
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }
}

/// OpenAI embedding provider using text-embedding-ada-002 or text-embedding-3-small.
pub struct OpenAiEmbeddings {
    client: reqwest::Client,
    api_key: String,
    model: String,
    dimension: usize,
}

impl OpenAiEmbeddings {
    /// Create a new OpenAI embedding provider with the default model.
    ///
    /// Uses text-embedding-3-small which has 1536 dimensions.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "text-embedding-3-small".to_string(),
            dimension: 1536,
        }
    }

    /// Use text-embedding-ada-002 model.
    pub fn ada_002(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "text-embedding-ada-002".to_string(),
            dimension: 1536,
        }
    }

    /// Use text-embedding-3-large model.
    pub fn large(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "text-embedding-3-large".to_string(),
            dimension: 3072,
        }
    }

    /// Use a custom model with specified dimension.
    pub fn with_model(
        api_key: impl Into<String>,
        model: impl Into<String>,
        dimension: usize,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: model.into(),
            dimension,
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAiEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OpenAiEmbeddings {
    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn max_input_length(&self) -> usize {
        // text-embedding-3-small/large: 8191 tokens (~32k chars)
        // text-embedding-ada-002: 8191 tokens
        32_000
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.len() > self.max_input_length() {
            return Err(EmbeddingError::TextTooLong {
                length: text.len(),
                max: self.max_input_length(),
            });
        }

        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InvalidResponse("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = OpenAiEmbeddingRequest {
            model: &self.model,
            input: texts,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(EmbeddingError::AuthFailed);
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(std::time::Duration::from_secs);
            return Err(EmbeddingError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::HttpError(format!(
                "Status {}: {}",
                status, error_text
            )));
        }

        let result: OpenAiEmbeddingResponse = response.json().await.map_err(|e| {
            EmbeddingError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        Ok(result.data.into_iter().map(|d| d.embedding).collect())
    }
}

/// NEAR AI embedding provider using the NEAR AI API.
///
/// Uses the same session-based auth as the LLM provider.
pub struct NearAiEmbeddings {
    client: reqwest::Client,
    base_url: String,
    session: std::sync::Arc<crate::llm::SessionManager>,
    model: String,
    dimension: usize,
}

impl NearAiEmbeddings {
    /// Create a new NEAR AI embedding provider.
    ///
    /// Uses the same session manager as the LLM provider for auth.
    pub fn new(
        base_url: impl Into<String>,
        session: std::sync::Arc<crate::llm::SessionManager>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            session,
            model: "text-embedding-3-small".to_string(),
            dimension: 1536,
        }
    }

    /// Use a specific model.
    pub fn with_model(mut self, model: impl Into<String>, dimension: usize) -> Self {
        self.model = model.into();
        self.dimension = dimension;
        self
    }
}

#[derive(Debug, Serialize)]
struct NearAiEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(Debug, Deserialize)]
struct NearAiEmbeddingResponse {
    data: Vec<NearAiEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct NearAiEmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for NearAiEmbeddings {
    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn max_input_length(&self) -> usize {
        32_000
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.len() > self.max_input_length() {
            return Err(EmbeddingError::TextTooLong {
                length: text.len(),
                max: self.max_input_length(),
            });
        }

        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InvalidResponse("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        use secrecy::ExposeSecret;

        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = NearAiEmbeddingRequest {
            model: &self.model,
            input: texts,
        };

        let token = self
            .session
            .get_token()
            .await
            .map_err(|_| EmbeddingError::AuthFailed)?;

        // Hack: Force api.near.ai for embeddings as cloud-api seems to 404
        let url = "https://api.near.ai/v1/embeddings".to_string();
        
        tracing::info!("NEAR AI Embedding Request: URL={}, Model={}", url, self.model);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token.expose_secret()))
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(EmbeddingError::AuthFailed);
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(std::time::Duration::from_secs);
            return Err(EmbeddingError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::HttpError(format!(
                "Status {}: {}",
                status, error_text
            )));
        }

        let result: NearAiEmbeddingResponse = response.json().await.map_err(|e| {
            EmbeddingError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        Ok(result.data.into_iter().map(|d| d.embedding).collect())
    }
}

/// Local embedding provider using fastembed-rs.
///
/// Runs a small embedding model locally (e.g. all-MiniLM-L6-v2).
/// This is the "sovereign" option - no API keys required.
pub struct LocalEmbeddings {
    model: std::sync::Arc<fastembed::TextEmbedding>,
    dimension: usize,
    model_name: String,
}

impl LocalEmbeddings {
    /// Create a new local embedding provider.
    ///
    /// This will download the model on first run if not present.
    pub fn new() -> Result<Self, EmbeddingError> {
        let model = fastembed::TextEmbedding::try_new(Default::default())
            .map_err(|e| EmbeddingError::InitializationError(e.to_string()))?;
            
        Ok(Self {
            model: std::sync::Arc::new(model),
            dimension: 1536, // Pad to match DB schema
            model_name: "all-MiniLM-L6-v2".to_string(),
        })
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddings {
    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn max_input_length(&self) -> usize {
        512 // Default for MiniLM
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
         let embeddings = self.embed_batch(&[text.to_string()]).await?;
         embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InvalidResponse("No embedding generated".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        // Fastembed is synchronous but fast. We wrap it in spawn_blocking to avoid blocking async runtime.
        let model = self.model.clone();
        let texts_owned = texts.to_vec();
        let target_dim = self.dimension;
        
        tracing::info!("LocalEmbeddings: dispatching batch of {} texts to dedicated thread", texts_owned.len());

        let (tx, rx) = tokio::sync::oneshot::channel();

        // Spawn a dedicated thread with increased stack size (8MB) to prevent stack overflow in ONNX Runtime
        std::thread::Builder::new()
            .name("fastembed-worker".to_string())
            .stack_size(8 * 1024 * 1024) // 8MB stack
            .spawn(move || {
                tracing::info!("LocalEmbeddings: calling model.embed inside dedicated thread");
                let res = model.embed(texts_owned, None);
                tracing::info!("LocalEmbeddings: model.embed returned");
                let _ = tx.send(res);
            })
            .map_err(|e| EmbeddingError::ExecutionError(format!("Failed to spawn thread: {}", e)))?;

        let result = rx.await
            .map_err(|e| EmbeddingError::ExecutionError(format!("Thread join error: {}", e)))?
            .map_err(|e| EmbeddingError::ExecutionError(e.to_string()))?;

        tracing::info!("LocalEmbeddings: resizing vectors");
        // Process results and apply padding
        let padded_vectors: Vec<Vec<f32>> = result.into_iter().map(|mut vec| {
            if vec.len() < target_dim {
                vec.resize(target_dim, 0.0);
            } else if vec.len() > target_dim {
                vec.truncate(target_dim);
            }
            vec
        }).collect();

        Ok(padded_vectors)
    }
}

/// A mock embedding provider for testing.
///
/// Generates deterministic embeddings based on text hash.
/// Useful for unit and integration tests.
pub struct MockEmbeddings {
    dimension: usize,
}

impl MockEmbeddings {
    /// Create a new mock embeddings provider with the given dimension.
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddings {
    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        "mock-embedding"
    }

    fn max_input_length(&self) -> usize {
        10_000
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // Generate a deterministic embedding based on text hash
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut embedding = Vec::with_capacity(self.dimension);
        let mut seed = hash;
        for _ in 0..self.dimension {
            // Simple LCG for deterministic random values
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = (seed as f32 / u64::MAX as f32) * 2.0 - 1.0;
            embedding.push(value);
        }

        // Normalize to unit length
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for x in &mut embedding {
                *x /= magnitude;
            }
        }

        Ok(embedding)
    }
}

/// Google Gemini embedding provider using OpenAI-compatible API.
pub struct GoogleEmbeddings {
    client: reqwest::Client,
    api_key: String,
    model: String,
    dimension: usize,
    base_url: String,
}

impl GoogleEmbeddings {
    /// Create a new Google embedding provider.
    ///
    /// Defaults to "text-embedding-004" which has 768 dimensions.
    /// Note: Google's default dimension is 768, but Sophia's DB expects 1536.
    /// We might need to use a model that supports 1536 or project it.
    ///
    /// However, Google's "text-embedding-004" via OpenAI adapter might support
    /// dimensions param or we just use it as is if we can re-create the DB table.
    ///
    /// WAIT: The database `memory_chunks` table has `embedding VECTOR(1536)`.
    /// If we use Google's 768-dim model, insert will fail.
    ///
    /// workaround: Use the OpenAI-compatible endpoint which often normalizes/pads
    /// or returns what we ask if the model supports it.
    ///
    /// For now, we will assume the user has a model that can support 1536 or we
    /// fail at runtime if the dimension mismatches.
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "embedding-001".to_string(), // Fallback to stable model
            dimension: 1536, // We will pad to this size
            base_url: base_url.into(),
        }
    }

    /// Use a custom model with specified dimension.
    pub fn with_model(
        mut self,
        model: impl Into<String>,
        dimension: usize,
    ) -> Self {
        self.model = model.into();
        self.dimension = dimension;
        self
    }
}

#[async_trait]
impl EmbeddingProvider for GoogleEmbeddings {
    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn max_input_length(&self) -> usize {
        2048
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
         if text.len() > self.max_input_length() {
             let truncated = &text[..self.max_input_length()];
             let embeddings = self.embed_batch(&[truncated.to_string()]).await?;
             embeddings
                 .into_iter()
                 .next()
                 .ok_or_else(|| EmbeddingError::InvalidResponse("No embedding returned".to_string()))
        } else {
             let embeddings = self.embed_batch(&[text.to_string()]).await?;
             embeddings
                 .into_iter()
                 .next()
                 .ok_or_else(|| EmbeddingError::InvalidResponse("No embedding returned".to_string()))
        }
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Native Google API: models/{model}:batchEmbedContents
        let batch_requests: Vec<serde_json::Value> = texts
            .iter()
            .map(|text| {
                serde_json::json!({
                    "model": format!("models/{}", self.model),
                    "content": {
                        "parts": [{ "text": text }]
                    }
                })
            })
            .collect();

        let request = serde_json::json!({
            "requests": batch_requests
        });

        let base_url = if self.base_url.contains("/openai") {
            self.base_url.replace("/openai", "")
        } else {
            self.base_url.clone()
        };
        
        // Ensure clean URL construction
        let url = format!(
            "{}/models/{}:batchEmbedContents", 
            base_url.trim_end_matches('/'), 
            self.model
        );

        tracing::info!(
            "Google Native Embeddings Request: URL={}, Model={} (Padded to {})", 
            url, 
            self.model,
            self.dimension
        );

        let response = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Google Embeddings Failed ({}): {}", status, error_text);
            
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(EmbeddingError::AuthFailed);
            }
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                 return Err(EmbeddingError::RateLimited { retry_after: None });
            }
            
            return Err(EmbeddingError::HttpError(format!("Status {}: {}", status, error_text)));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| {
            EmbeddingError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;
        
        let embeddings_json = result.get("embeddings").and_then(|v| v.as_array());
        
        if let Some(emb_list) = embeddings_json {
            let mut vectors = Vec::new();
            for item in emb_list {
                 if let Some(values) = item.get("values").and_then(|v| v.as_array()) {
                     let mut vec: Vec<f32> = values.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect();
                     
                     // ZERO PADDING logic to match DB dimension (1536)
                     // Google returns 768. We add zeros.
                     // Cosine similarity is preserved.
                     if vec.len() < self.dimension {
                         vec.resize(self.dimension, 0.0);
                     } else if vec.len() > self.dimension {
                         vec.truncate(self.dimension);
                     }
                     
                     vectors.push(vec);
                 } else {
                     return Err(EmbeddingError::InvalidResponse("Missing 'values' in embedding response".to_string()));
                 }
            }
            Ok(vectors)
        } else {
             Err(EmbeddingError::InvalidResponse("Missing 'embeddings' field in response".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embeddings() {
        let provider = MockEmbeddings::new(128);

        let embedding = provider.embed("hello world").await.unwrap();
        assert_eq!(embedding.len(), 128);

        // Check normalization (should be unit vector)
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_mock_embeddings_deterministic() {
        let provider = MockEmbeddings::new(64);

        let emb1 = provider.embed("test").await.unwrap();
        let emb2 = provider.embed("test").await.unwrap();

        // Same input should produce same embedding
        assert_eq!(emb1, emb2);
    }

    #[tokio::test]
    async fn test_mock_embeddings_batch() {
        let provider = MockEmbeddings::new(64);

        let texts = vec!["hello".to_string(), "world".to_string()];
        let embeddings = provider.embed_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 64);
        assert_eq!(embeddings[1].len(), 64);

        // Different texts should produce different embeddings
        assert_ne!(embeddings[0], embeddings[1]);
    }

    #[test]
    fn test_openai_embeddings_config() {
        let provider = OpenAiEmbeddings::new("test-key");
        assert_eq!(provider.dimension(), 1536);
        assert_eq!(provider.model_name(), "text-embedding-3-small");

        let provider = OpenAiEmbeddings::large("test-key");
        assert_eq!(provider.dimension(), 3072);
        assert_eq!(provider.model_name(), "text-embedding-3-large");
    }
}
