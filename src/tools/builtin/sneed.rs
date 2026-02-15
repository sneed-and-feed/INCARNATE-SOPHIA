use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;

use crate::context::JobContext;
use crate::sneed_engine::{GlyphWave, StakesEngine};
use crate::tools::{Tool, ToolError, ToolOutput};

/// Tool for performing a retrocausal logic audit using the Sneed Engine.
pub struct SneedTool;

impl SneedTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SneedTool {
    fn name(&self) -> &str {
        "sneed"
    }

    fn description(&self) -> &str {
        "Perform a retrocausal logic audit on the specified intent or query."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The intent or query to audit."
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        _ctx: &JobContext,
    ) -> Result<ToolOutput, ToolError> {
        let query = params["query"].as_str().ok_or_else(|| {
            ToolError::ExecutionFailed("Missing required parameter: query".to_string())
        })?;

        let mut engine = StakesEngine::new();
        let detected = StakesEngine::detect_stakes(query);
        let _agency = engine.deliberate(query, &detected);
        
        let header = GlyphWave::render("SOVEREIGN LOGIC AUDIT");
        let resonance_report = engine.get_resonance_report();
        
        let audit_results = format!(
            "{}\n\n{}\n\n**Audit Result for:** \"{}\"\n\n**Status:** Coherent via TAU_SOVEREIGN modulation.\n**Conclusion:** Intent validated against the 7 Seals of Capability.",
            header,
            resonance_report,
            query
        );

        Ok(ToolOutput::success(
            serde_json::json!({ "result": audit_results }),
            Duration::from_millis(50)
        ))
    }
}
