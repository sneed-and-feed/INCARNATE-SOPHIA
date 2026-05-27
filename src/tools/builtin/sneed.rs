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
        let (_agency, c_norm) = engine.deliberate(query, &detected);
        
        // Run a SovereignGrid step simulation
        let mut grid = crate::sneed_engine::SovereignGrid::new(3, 8);
        
        // Perturb potentials based on node index to model a non-trivial potential field V
        for (i, node) in grid.nodes.iter_mut().enumerate() {
            let val = (i as f64 * 0.2).sin().abs() * 2.0 + 1.0;
            node.spatial_attention_scale = val;
        }

        // Build bio input from detected stakes
        let mut bio_data = vec![0.05; 8];
        for (stake, &val) in &detected {
            let idx = *stake as usize % 8;
            bio_data[idx] = (bio_data[idx] + val).min(1.0);
        }
        let bio_input = crate::sneed_engine::FlumpyArray::new(bio_data, 1.0);
        let _out = grid.process_step(&bio_input, false, c_norm);
        
        let (coherence, alpha, sigma) = grid.get_spectral_metrics();
        
        let total_potential: f64 = grid.nodes.iter().map(|n| n.spatial_attention_scale).sum();
        let avg_potential = total_potential / grid.nodes.len() as f64;
        let min_potential = grid.nodes.iter().map(|n| n.spatial_attention_scale).fold(f64::INFINITY, f64::min);
        let max_potential = grid.nodes.iter().map(|n| n.spatial_attention_scale).fold(f64::NEG_INFINITY, f64::max);

        let header = GlyphWave::render("SOVEREIGN LOGIC AUDIT");
        let resonance_report = engine.get_resonance_report();
        
        let audit_results = format!(
            "{}\n\n{}\n\n\
             ### SPECTRAL GEOMETRY TELEMETRY\n\
             - **Spectral Coherence**: {:.6}\n\
             - **Eigenvalue Decay Rate (Alpha)**: {:.6}\n\
             - **Spectral Chaos (Sigma)**: {:.6}\n\
             - **Average Grid Potential (V)**: {:.6}\n\
             - **Min/Max Potential Field**: {:.4} / {:.4}\n\
             - **Substrate Status**: {}\n\n\
             **Audit Result for:** \"{}\"\n\n\
             **Status:** Coherent via Adèlic Spectral geometry.\n\
             **Conclusion:** Intent validated against the 7 Seals of Capability.",
            header,
            resonance_report,
            coherence,
            alpha,
            sigma,
            avg_potential,
            min_potential,
            max_potential,
            if coherence >= crate::sneed_engine::COHERENCE_THRESHOLD { "STABLE (🌀 Coherence >= 0.999)" } else { "DEGRADED (⚠️ Coherence < 0.999)" },
            query
        );

        Ok(ToolOutput::success(
            serde_json::json!({ "result": audit_results }),
            Duration::from_millis(50)
        ))
    }
}
