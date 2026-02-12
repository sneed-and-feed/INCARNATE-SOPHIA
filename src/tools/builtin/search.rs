//! DuckDuckGo search tool.
//!
//! Provides web search capabilities by executing a Python helper script.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use crate::context::JobContext;
use crate::tools::tool::{Tool, ToolError, ToolOutput};

/// Maximum output size before truncation (64KB).
const MAX_OUTPUT_SIZE: usize = 64 * 1024;

/// Default search timeout.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// DuckDuckGo search tool.
pub struct SearchTool {
    /// Path to the Python helper script.
    script_path: PathBuf,
    /// Search timeout.
    timeout: Duration,
}

impl SearchTool {
    /// Create a new search tool with default settings.
    pub fn new() -> Self {
        Self {
            script_path: PathBuf::from("tools-src/search_ddg.py"),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Set the script path.
    pub fn with_script_path(mut self, path: PathBuf) -> Self {
        self.script_path = path;
        self
    }

    /// Set the search timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Execute the search via the Python helper.
    async fn execute_search(
        &self,
        query: &str,
        max_results: u64,
    ) -> Result<serde_json::Value, ToolError> {
        let max_results_str = max_results.to_string();
        
        #[cfg(target_os = "windows")]
        let (program, args) = ("cmd", vec!["/C", "python.bat", self.script_path.to_str().unwrap(), query, &max_results_str]);

        #[cfg(not(target_os = "windows"))]
        let (program, args) = ("python3", vec![self.script_path.to_str().unwrap(), query, &max_results_str]);

        let mut command = Command::new(program);
        command
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn process
        let mut child = command
            .spawn()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to spawn search process: {}", e)))?;

        // Wait with timeout
        let result = tokio::time::timeout(self.timeout, async {
            let status = child.wait().await?;

            // Read stdout
            let mut stdout = Vec::new();
            if let Some(mut out) = child.stdout.take() {
                out.read_to_end(&mut stdout).await?;
            }

            // Read stderr
            let mut stderr = Vec::new();
            if let Some(mut err) = child.stderr.take() {
                err.read_to_end(&mut stderr).await?;
            }

            if !status.success() {
                let err_msg = String::from_utf8_lossy(&stderr);
                return Err(ToolError::ExecutionFailed(format!(
                    "Search process failed (exit code {}): {}",
                    status.code().unwrap_or(-1),
                    err_msg
                )));
            }

            // Parse JSON output
            let results: serde_json::Value = serde_json::from_slice(&stdout)
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse search results: {}", e)))?;

            Ok(results)
        })
        .await;

        match result {
            Ok(res) => res,
            Err(_) => {
                let _ = child.kill().await;
                Err(ToolError::Timeout(self.timeout))
            }
        }
    }
}

impl Default for SearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Perform a web search using DuckDuckGo. Highly effective for finding current information, documentation, and news."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return (optional, default 5)"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &JobContext,
    ) -> Result<ToolOutput, ToolError> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("missing 'query' parameter".into()))?;

        let max_results = params
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(5);

        let start = std::time::Instant::now();
        let results = self.execute_search(query, max_results).await?;
        let duration = start.elapsed();

        // Check for error objects in the result list (from Python script)
        if let Some(arr) = results.as_array() {
            if let Some(first) = arr.first() {
                if let Some(err) = first.get("error").and_then(|v| v.as_str()) {
                    return Err(ToolError::ExecutionFailed(err.to_string()));
                }
            }
        }

        Ok(ToolOutput::success(results, duration))
    }

    fn requires_approval(&self) -> bool {
        true // Web search involves external network access
    }

    fn requires_sanitization(&self) -> bool {
        true // Search results could contain anything
    }
}
