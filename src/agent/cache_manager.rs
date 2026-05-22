//! Cache Manager for Ironclaw Sophia.
//!
//! Manages the lifecycle of Context Caching blocks to ensure Sophia's
//! "Sovereign Anchor" and Core Logic remain active without burning tokens.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::llm::{LlmProvider, ToolDefinition};

/// Manages the explicit context caching layer.
pub struct CacheManager {
    llm: Arc<dyn LlmProvider>,
    active_cache_id: Mutex<Option<(String, Instant)>>,
    ttl: Duration,
}

impl CacheManager {
    /// Create a new CacheManager.
    pub fn new(llm: Arc<dyn LlmProvider>, ttl: Duration) -> Self {
        Self {
            llm,
            active_cache_id: Mutex::new(None),
            ttl,
        }
    }

    /// Ensure the cache is valid, recreating if necessary.
    /// Returns the active `cache_id` if caching is supported and successful.
    pub async fn ensure_cache(&self, system_prompt: &str, tools: Vec<ToolDefinition>) -> Option<String> {
        let mut guard = self.active_cache_id.lock().await;

        if let Some((ref id, created_at)) = *guard {
            // Check if we have at least 1 hour left on TTL, otherwise renew
            if created_at.elapsed() + Duration::from_secs(3600) < self.ttl {
                return Some(id.clone());
            } else {
                // Delete expired or expiring cache
                tracing::info!("Cache {} is expiring soon, deleting to renew", id);
                let _ = self.llm.delete_cache(id).await;
                *guard = None;
            }
        }

        // Create new cache
        let messages = vec![]; // Core identity loaded into system instruction instead of messages
        match self.llm.create_cache(
            self.ttl.as_secs() as i32,
            messages,
            Some(system_prompt.to_string()),
            tools,
        ).await {
            Ok(cache_id) => {
                tracing::info!("Created new context cache: {}", cache_id);
                *guard = Some((cache_id.clone(), Instant::now()));
                Some(cache_id)
            }
            Err(e) => {
                tracing::warn!("Failed to create context cache (provider might not support it): {}", e);
                None
            }
        }
    }

    /// Force invalidation of the active cache.
    pub async fn invalidate(&self) {
        let mut guard = self.active_cache_id.lock().await;
        if let Some((ref id, _)) = *guard {
            tracing::info!("Invalidating cache: {}", id);
            let _ = self.llm.delete_cache(id).await;
        }
        *guard = None;
    }
}
