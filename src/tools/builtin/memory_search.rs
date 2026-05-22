use std::sync::Arc;

use async_trait::async_trait;

use crate::context::JobContext;
use crate::llm::LlmProvider;
use crate::tools::tool::{Tool, ToolError, ToolOutput};

/// Tool for uploading a file from the host filesystem directly to the LLM context.
/// This utilizes Gemini's native File API for Multimodal RAG analysis.
pub struct MemoryUploadTool {
    llm: Arc<dyn LlmProvider>,
}

impl MemoryUploadTool {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Tool for MemoryUploadTool {
    fn name(&self) -> &str {
        "memory_upload"
    }

    fn description(&self) -> &str {
        "Upload a local file or asset (like an image, PDF, or large document) to the LLM context for multimodal analysis. \
         Returns the attached file URI which the LLM will automatically include in the next turn's context. \
         Only use this when you explicitly need to inspect the contents of a file that exceeds normal text bounds."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute or relative path to the local file to upload (e.g., 'assets/diagram.png', 'docs/manual.pdf')"
                },
                "mime_type": {
                    "type": "string",
                    "description": "Optional: The MIME type of the file (e.g., 'image/png'). If omitted, it will be guessed from the extension."
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &JobContext,
    ) -> Result<ToolOutput, ToolError> {
        let start = std::time::Instant::now();

        let path_str = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("missing 'path' parameter".to_string()))?;

        let path = std::path::Path::new(path_str);
        if !path.exists() {
            return Err(ToolError::ExecutionFailed(format!("File does not exist: {}", path_str)));
        }

        let mime_type = params
            .get("mime_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                // Map common text/code extensions to text/plain to ensure Gemini reads them as text.
                // Note: .ts defaults to video/mp2t in mime_guess, so we must explicitly map it to text/plain.
                match ext.as_str() {
                    "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "go" | "c" | "cpp" | "h" | "hpp" 
                    | "cs" | "java" | "sh" | "bat" | "ps1" | "md" | "toml" | "yaml" | "yml" 
                    | "json" | "txt" | "sql" | "graphql" | "proto" | "dockerfile" | "lock" => {
                        "text/plain".to_string()
                    }
                    _ => mime_guess::from_path(path)
                        .first_raw()
                        .unwrap_or("application/octet-stream")
                        .to_string(),
                }
            });

        // Upload using the LLM provider
        let file_uri = self
            .llm
            .upload_file(path, &mime_type)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to upload file to Gemini: {}", e)))?;

        // Return a ToolOutput containing the success message and attaching the file_uri
        let mut output = ToolOutput::success(serde_json::json!({
            "status": "success",
            "file_uri": file_uri.clone(),
            "mime_type": mime_type.to_string(),
            "message": "File successfully uploaded and attached to the context. You can now analyze it."
        }), start.elapsed());
        
        output = output.with_file_attachment(file_uri, mime_type.to_string());
        
        Ok(output)
    }
}
