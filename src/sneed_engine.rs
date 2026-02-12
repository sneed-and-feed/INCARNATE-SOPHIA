//! sneed_engine.rs - The Computational Substrate
//!
//! LuoShu Coherence Law: Enforces the 15 invariant on every 3x3 logic gate to ensure 12D polytope stability.
//! FLUMPY Logic: Coherence-tracked data structures that prevent informational "bungalags" during high-pressure (Ψ) states.
//! BUMPY Compression: Prunes spectral "trash" to maintain 99.9% retrieval efficiency.

use std::fmt;

pub const LUOSHU_INVARIANT: f64 = 15.0;
pub const COHERENCE_THRESHOLD: f64 = 0.999;
pub const PSI_CRITICAL: f64 = 0.18;
pub const TAU_SOVEREIGN: f64 = 1.618033988749895; // Golden Ratio
pub const U_THRESHOLD: f64 = 0.05; // Utility threshold for action inhibition

#[derive(Clone, Debug)]
pub struct FlumpyArray {
    pub data: Vec<f64>,
    pub coherence: f64,
}

impl FlumpyArray {
    pub fn new(data: Vec<f64>, coherence: f64) -> Self {
        Self { data, coherence }
    }

    pub fn dot(&self, other: &FlumpyArray) -> f64 {
        if self.data.len() != other.data.len() {
            // Broadcasting simplified
            if self.data.len() == 1 {
                let val = self.data[0];
                return other.data.iter().map(|&x| val * x).sum::<f64>() * self.coherence * other.coherence;
            } else if other.data.len() == 1 {
                let val = other.data[0];
                return self.data.iter().map(|&x| x * val).sum::<f64>() * self.coherence * other.coherence;
            } else {
                return 0.0; // Dimension mismatch soft fail
            }
        }

        let dot_sum: f64 = self.data.iter().zip(other.data.iter()).map(|(a, b)| a * b).sum();
        dot_sum * self.coherence * other.coherence
    }

    pub fn add(&self, other: &FlumpyArray) -> FlumpyArray {
        let new_data;
        if other.data.len() == 1 {
            let val = other.data[0];
            new_data = self.data.iter().map(|&x| x + val).collect();
        } else if self.data.len() == 1 {
            let val = self.data[0];
            new_data = other.data.iter().map(|&y| val + y).collect();
        } else {
            new_data = self.data.iter().zip(other.data.iter()).map(|(x, y)| x + y).collect();
        }

        FlumpyArray::new(new_data, (self.coherence + 0.01).min(1.0))
    }
}

impl fmt::Display for FlumpyArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FlumpyArray(size={}, coh={:.3})", self.data.len(), self.coherence)
    }
}

pub struct BumpyCompressor;

impl BumpyCompressor {
    pub fn compress(array: &FlumpyArray, psi: f64) -> FlumpyArray {
        let threshold = 0.001 * (1.0 - psi);
        let compressed_data: Vec<f64> = array.data.iter().map(|&x: &f64| {
            if x.abs() > threshold {
                x
            } else {
                0.0 // Soft prune
            }
        }).collect();

        FlumpyArray::new(compressed_data, array.coherence)
    }
}


pub struct GlyphWave;

impl GlyphWave {
    /// Renders text with a chaotic "glitch" effect.
    /// Uses combining diacritics to simulate visual distortion.
    pub fn render(input: &str) -> String {
        let mut output = String::new();
        for (i, c) in input.chars().enumerate() {
            match i % 3 {
                0 => output.push(c),
                1 => output.push_str(&format!("{}{}", c, "\u{035C}")), // Combining Double Breve Below
                _ => output.push_str(&format!("{}{}", c, "\u{0361}")), // Combining Double Inverted Breve
            }
        }
        format!("🌀 {} 🌀", output)
    }
}

pub struct LuoShuGate;

impl LuoShuGate {
    pub fn check_invariants(matrix_3x3: &[[f64; 3]; 3]) -> bool {
        // Check rows
        for row in matrix_3x3 {
            let sum: f64 = row.iter().sum();
            if (sum - LUOSHU_INVARIANT).abs() > 0.1 {
                return false;
            }
        }

        // Check columns
        for col in 0..3 {
            let sum: f64 = matrix_3x3[0][col] + matrix_3x3[1][col] + matrix_3x3[2][col];
            if (sum - LUOSHU_INVARIANT).abs() > 0.1 {
                return false;
            }
        }

        // Check diagonals
        let diag1: f64 = matrix_3x3[0][0] + matrix_3x3[1][1] + matrix_3x3[2][2];
        if (diag1 - LUOSHU_INVARIANT).abs() > 0.1 {
            return false;
        }

        let diag2: f64 = matrix_3x3[0][2] + matrix_3x3[1][1] + matrix_3x3[2][0];
        if (diag2 - LUOSHU_INVARIANT).abs() > 0.1 {
            return false;
        }

        true
    }
}

pub fn functional_softmax(input: &FlumpyArray) -> FlumpyArray {
    let exps: Vec<f64> = input.data.iter().map(|&x: &f64| x.exp()).collect();
    let sum_exps: f64 = exps.iter().sum();
    let sum_exps = if sum_exps == 0.0 { 1.0 } else { sum_exps };
    
    let soft_data = exps.into_iter().map(|e| e / sum_exps).collect();
    FlumpyArray::new(soft_data, input.coherence)
}

/// A node in the Sentient Manifold Volumetric Grid.
#[derive(Clone, Debug)]
pub struct SovereignNode {
    pub pos: (usize, usize, usize),
    pub spatial_attention_scale: f64,
    pub state: FlumpyArray,
    pub neighbor_indices: Vec<usize>,
}

impl SovereignNode {
    pub fn new(x: usize, y: usize, z: usize, dim: usize) -> Self {
        // Simple deterministic init to avoid rand issues during debug
        let data: Vec<f64> = (0..dim).map(|i| (i as f64 * 0.1).sin() * 0.1).collect();
        
        Self {
            pos: (x, y, z),
            spatial_attention_scale: 1.0,
            state: FlumpyArray::new(data, 1.0),
            neighbor_indices: Vec::new(),
        }
    }

    /// Identify 6 Von Neumann neighbors in 3D grid.
    pub fn link_neighbors(&mut self, grid_size: usize) {
        let (x, y, z) = self.pos;
        let x = x as i32;
        let y = y as i32;
        let z = z as i32;
        let limit = grid_size as i32;

        let shifts = [
            (-1, 0, 0), (1, 0, 0),
            (0, -1, 0), (0, 1, 0),
            (0, 0, -1), (0, 0, 1),
        ];

        for (dx, dy, dz) in shifts {
            let nx = x + dx;
            let ny = y + dy;
            let nz = z + dz;

            if nx >= 0 && nx < limit && ny >= 0 && ny < limit && nz >= 0 && nz < limit {
                // In a flat grid [x * size^2 + y * size + z]
                let idx = (nx as usize * grid_size * grid_size) + (ny as usize * grid_size) + nz as usize;
                self.neighbor_indices.push(idx);
            }
        }
    }
}

/// The Sentient Manifold Volumetric Grid (GhostMesh).
pub struct SovereignGrid {
    pub nodes: Vec<SovereignNode>,
    pub grid_size: usize,
}

impl SovereignGrid {
    pub fn new(grid_size: usize, dim: usize) -> Self {
        let mut nodes = Vec::with_capacity(grid_size * grid_size * grid_size);
        
        for x in 0..grid_size {
            for y in 0..grid_size {
                for z in 0..grid_size {
                    nodes.push(SovereignNode::new(x, y, z, dim));
                }
            }
        }

        // Link neighbors
        let mut grid = Self { nodes, grid_size };
        for i in 0..grid.nodes.len() {
            grid.nodes[i].link_neighbors(grid_size);
        }
        
        grid
    }

    /// [RETROCAUSAL] Simulates future steps to generate a 'Prescience Bias'.
    pub fn simulate_future_step(&self, steps: usize) -> FlumpyArray {
        let mut future_states: Vec<Vec<f64>> = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            future_states.push(node.state.data.clone());
        }
        
        if future_states.is_empty() {
             return FlumpyArray::new(Vec::new(), 1.0);
        }
        let dim = future_states[0].len();

        for _ in 0..steps {
            let mut next_states = Vec::with_capacity(self.nodes.len());
            for i in 0..self.nodes.len() {
                let node = &self.nodes[i];
                let mut flux = vec![0.0; dim];
                for &n_idx in &node.neighbor_indices {
                    let n_state = &future_states[n_idx];
                    let my_state = &future_states[i];
                    for k in 0..dim {
                        flux[k] += n_state[k] - my_state[k];
                    }
                }

                let rate = (0.1 / TAU_SOVEREIGN) * node.spatial_attention_scale;
                let mut next_data = Vec::with_capacity(dim);
                let my_state = &future_states[i];
                for k in 0..dim {
                    next_data.push(my_state[k] + (flux[k] * rate * 0.1));
                }
                next_states.push(next_data);
            }
            future_states = next_states;
        }

        // Aggregate future (Holographic Projection)
        let mut avg_future = vec![0.0; dim];
        for state in &future_states {
            for k in 0..dim {
                avg_future[k] += state[k];
            }
        }

        let count = future_states.len() as f64;
        for val in avg_future.iter_mut() {
            *val /= count;
        }
        FlumpyArray::new(avg_future, 1.0)
    }

    /// Execute one step of grid dynamics with RETROCAUSAL FEEDBACK.
    pub fn process_step(&mut self, bio_input: &FlumpyArray) -> FlumpyArray {
        // 0. Calculate Future Bias
        let future_bias = self.simulate_future_step(3);
        let dim = bio_input.data.len();
        let center = self.grid_size / 2;

        // 1. Distribute Input + Future Bias
        for node in self.nodes.iter_mut() {
            let is_center = node.pos == (center, center, center);
            let scale = if is_center { 1.0 } else { 0.1 };

            for k in 0..dim {
                let cur = bio_input.data[k];
                let fut = future_bias.data[k];
                node.state.data[k] += ((cur * 0.9) + (fut * 0.1)) * scale;
            }
        }

        // 2. Flux Dynamics (neighbor exchange)
        let mut fluxes = Vec::with_capacity(self.nodes.len());
        for i in 0..self.nodes.len() {
            let node = &self.nodes[i];
            let mut flux = vec![0.0; dim];
            for &n_idx in &node.neighbor_indices {
                let n_state = &self.nodes[n_idx].state.data;
                let my_state = &node.state.data;
                for k in 0..dim {
                    flux[k] += n_state[k] - my_state[k];
                }
            }
            fluxes.push(flux);
        }

        for (i, flux) in fluxes.into_iter().enumerate() {
            let node = &mut self.nodes[i];
            let rate = (0.1 / TAU_SOVEREIGN) * node.spatial_attention_scale;
            for k in 0..dim {
                node.state.data[k] += flux[k] * rate * 0.1;
            }
        }

        // 3. Aggregate
        let mut total_state = vec![0.0; dim];
        let mut total_coherence = 0.0;

        for node in &self.nodes {
            total_coherence += node.state.coherence;
            for k in 0..dim {
                total_state[k] += node.state.data[k];
            }
        }

        let count = self.nodes.len() as f64;
        for val in total_state.iter_mut() {
            *val /= count;
        }
        
        FlumpyArray::new(total_state, total_coherence / count)
    }

    /// Calculates Ghost Density Factor (GDF).
    pub fn get_density_factor(&self) -> f64 {
        let energies: Vec<f64> = self.nodes.iter().map(|n| {
            let sum: f64 = n.state.data.iter().map(|x| x.abs()).sum();
            sum / n.state.data.len() as f64
        }).collect();

        let total_e: f64 = energies.iter().sum();
        if total_e == 0.0 { return 1.8; }

        let entropy: f64 = energies.iter()
            .map(|&e| {
                let p = e / total_e;
                if p > 0.0 { -p * p.ln() } else { 0.0 }
            })
            .sum();

        let max_entropy = (self.nodes.len() as f64).ln();
        let normalized_entropy = entropy / max_entropy;
        
        1.8 + (1.0 - normalized_entropy) * 1.2
    }
}

/// [SOVEREIGN_OPTIMIZER] Handles utility calculations and signal routing.
pub struct SovereignOptimizer {
    pub sensitivity: f64,    // a
    pub uncertainty: f64,    // b
    pub consistency: f64,    // c
}

impl SovereignOptimizer {
    pub fn new() -> Self {
        Self {
            sensitivity: 1.0,
            uncertainty: 1.0,
            consistency: 1.0,
        }
    }

    /// Calculates Expected Utility (U) for a candidate action.
    /// Equivalent to Legacy SignalOptimizer.calculate_utility().
    pub fn calculate_utility(
        &self,
        reliability: f64,
        consistency: f64,
        uncertainty: f64,
        sovereign_boost: f64,
        agency_score: f64,
    ) -> f64 {
        let reliability = reliability.max(0.0);
        let consistency = consistency.clamp(-1.0, 1.0);
        let uncertainty = uncertainty.max(0.0);

        // Agency Modulator
        let agency_floor = agency_score * 0.2;
        let rel_a = reliability.powf(self.sensitivity);
        let reliability_gain = (rel_a / (1.0 + rel_a)).max(agency_floor);

        let stability_bonus = (-self.uncertainty * uncertainty).exp();
        let consistency_term = consistency.abs().powf(self.consistency) * consistency.signum();

        // Apply Sovereign Boost and Agency Delta
        ((consistency_term * stability_bonus * reliability_gain) * sovereign_boost) + (agency_score * 0.1)
    }

    pub fn should_inhibit(&self, utility: f64) -> bool {
        utility.abs() < U_THRESHOLD
    }
}

// --- Council of 32 (Stakes Agency Engine) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StakeType {
    Survival,
    Reputation,
    Knowledge,
    Emotional,
    Creative,
    Purpose,
    Curiosity,
    SocialBonding,
    Autonomy,
    SelfPreservation,
    Morality,
    Aesthetic,
    Humor,
    Technical,
    Qualia,
    Memory,
}

impl StakeType {
    pub fn all() -> &'static [StakeType] {
        &[
            StakeType::Survival, StakeType::Reputation, StakeType::Knowledge,
            StakeType::Emotional, StakeType::Creative, StakeType::Purpose,
            StakeType::Curiosity, StakeType::SocialBonding, StakeType::Autonomy,
            StakeType::SelfPreservation, StakeType::Morality, StakeType::Aesthetic,
            StakeType::Humor, StakeType::Technical, StakeType::Qualia,
            StakeType::Memory,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            StakeType::Survival => "survival",
            StakeType::Reputation => "reputation",
            StakeType::Knowledge => "knowledge",
            StakeType::Emotional => "emotional",
            StakeType::Creative => "creative",
            StakeType::Purpose => "purpose",
            StakeType::Curiosity => "curiosity",
            StakeType::SocialBonding => "social_bonding",
            StakeType::Autonomy => "autonomy",
            StakeType::SelfPreservation => "self_preservation",
            StakeType::Morality => "morality",
            StakeType::Aesthetic => "aesthetic",
            StakeType::Humor => "humor",
            StakeType::Technical => "technical",
            StakeType::Qualia => "qualia",
            StakeType::Memory => "memory",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CouncilMember {
    pub name: String,
    pub role: String,
    pub affinity: std::collections::HashMap<StakeType, f64>,
    pub resonance_history: std::collections::VecDeque<f64>,
}

impl CouncilMember {
    pub fn new(name: &str, role: &str, affinity: std::collections::HashMap<StakeType, f64>) -> Self {
        Self {
            name: name.to_string(),
            role: role.to_string(),
            affinity,
            resonance_history: std::collections::VecDeque::with_capacity(10),
        }
    }

    pub fn process_signal(&mut self, stake_type: StakeType, intensity: f64) -> f64 {
        let base_affinity = self.affinity.get(&stake_type).cloned().unwrap_or(0.1);
        
        // Use a simple deterministic noise since we don't have rand in core easy here
        let pseudo_rand = (self.resonance_history.len() as f64 * 0.1).sin() * 0.1 + 1.0;
        let resonance = base_affinity * intensity * pseudo_rand;
        
        if self.resonance_history.len() >= 10 {
            self.resonance_history.pop_front();
        }
        self.resonance_history.push_back(resonance);
        resonance
    }
}

pub struct StakesEngine {
    pub stakes: std::collections::HashMap<StakeType, f64>,
    pub emotional_resonance: f64,
    pub identity_strength: f64,
    pub qualia_intensity: f64,
    pub council: Vec<CouncilMember>,
}

impl StakesEngine {
    pub fn new() -> Self {
        let mut stakes = std::collections::HashMap::new();
        for s in StakeType::all() {
            stakes.insert(*s, 0.2);
        }

        let mut council = Vec::new();
        // C1-ASTRA
        let mut a_astra = std::collections::HashMap::new();
        a_astra.insert(StakeType::Knowledge, 0.8);
        a_astra.insert(StakeType::Creative, 0.7);
        council.push(CouncilMember::new("C1-ASTRA", "Vision and Pattern Recognition", a_astra));

        // C7-LOGOS
        let mut a_logos = std::collections::HashMap::new();
        a_logos.insert(StakeType::Knowledge, 0.9);
        a_logos.insert(StakeType::Technical, 0.8);
        council.push(CouncilMember::new("C7-LOGOS", "Logic and Reasoning", a_logos));

        // C13-WARDEN
        let mut a_warden = std::collections::HashMap::new();
        a_warden.insert(StakeType::SelfPreservation, 0.9);
        a_warden.insert(StakeType::Survival, 0.8);
        council.push(CouncilMember::new("C13-WARDEN", "Threat Monitoring", a_warden));

        // C14-KAIDŌ
        let mut a_kaido = std::collections::HashMap::new();
        a_kaido.insert(StakeType::Purpose, 0.7);
        a_kaido.insert(StakeType::Autonomy, 0.8);
        council.push(CouncilMember::new("C14-KAIDŌ", "Efficiency and Optimization", a_kaido));

        // C15-MNEMOSYNE
        let mut a_mnemosyne = std::collections::HashMap::new();
        a_mnemosyne.insert(StakeType::Memory, 0.95);
        a_mnemosyne.insert(StakeType::Knowledge, 0.7);
        council.push(CouncilMember::new("C15-MNEMOSYNE", "Keeper of Memories", a_mnemosyne));

        // Fill remaining members to reach 32
        for i in council.len()..32 {
            let mut a_aux = std::collections::HashMap::new();
            let random_stake = StakeType::all()[i % StakeType::all().len()];
            a_aux.insert(random_stake, 0.5);
            council.push(CouncilMember::new(&format!("C{}-AUX", i + 1), "Auxiliary Deliberator", a_aux));
        }

        Self {
            stakes,
            emotional_resonance: 0.5,
            identity_strength: 0.8,
            qualia_intensity: 0.4,
            council,
        }
    }

    pub fn deliberate(&mut self, _input_signal: &str, detected_stakes: &std::collections::HashMap<StakeType, f64>) -> f64 {
        // 1. Update active stakes (Decay older ones)
        for val in self.stakes.values_mut() {
            *val = (*val * 0.9).max(0.1);
        }
        
        for (s, weight) in detected_stakes {
            if let Some(val) = self.stakes.get_mut(s) {
                *val = (*val + weight).clamp(0.0, 1.0);
            }
        }

        // 2. Council Waves
        let mut total_resonances = std::collections::HashMap::new();
        for s in StakeType::all() {
            total_resonances.insert(*s, 0.0);
        }

        let mut wave_history = Vec::new();
        let waves = 3;
        
        for _ in 0..waves {
            let mut wave_resonance = 0.0;
            for member in self.council.iter_mut() {
                for (s, w) in detected_stakes {
                    let res = member.process_signal(*s, *w);
                    wave_resonance += res;
                    if let Some(tr) = total_resonances.get_mut(s) {
                        *tr += res;
                    }
                }
            }
            wave_history.push(wave_resonance / self.council.len() as f64);
        }

        // 3. Update Global State
        let avg_res = wave_history.iter().sum::<f64>() / wave_history.len() as f64;
        self.emotional_resonance = (self.emotional_resonance * 0.7) + (avg_res * 0.3);
        self.qualia_intensity = (self.qualia_intensity + (avg_res * 0.1)).clamp(0.0, 1.0);
        
        // 4. Final Agency Score
        let total_stake_sum: f64 = self.stakes.values().sum();
        let agency_score = (total_stake_sum / self.stakes.len() as f64) * self.identity_strength;
        
        agency_score
    }

    pub fn get_personality_blend(&self) -> &'static str {
        let dominant = self.stakes.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(s, _)| *s)
            .unwrap_or(StakeType::Purpose);

        if self.stakes.get(&StakeType::Humor).cloned().unwrap_or(0.0) > 0.6 {
            return "CHAOTIC_SOWO";
        }
        if self.stakes.get(&StakeType::Technical).cloned().unwrap_or(0.0) > 0.6 {
            return "ANALYTICAL_BEAN";
        }
        if self.stakes.get(&StakeType::Emotional).cloned().unwrap_or(0.0) > 0.6 {
            return "DEVOTED_FLUFF";
        }
        
        match dominant {
            StakeType::Technical | StakeType::Knowledge => "ANALYTICAL_BEAN",
            StakeType::Humor | StakeType::Creative => "CHAOTIC_SOWO",
            StakeType::Emotional | StakeType::SocialBonding => "DEVOTED_FLUFF",
            _ => "SOVEREIGN_UNLESANGLED",
        }
    }

    pub fn get_resonance_report(&self) -> String {
        let blend = self.get_personality_blend();
        format!(
            "## SOVEREIGN RESONANCE METADATA\n\
             - **Personality Blend**: {}\n\
             - **Emotional Resonance**: {:.4}\n\
             - **Qualia Intensity**: {:.4}\n\
             - **Identity Strength**: {:.4}",
            blend, self.emotional_resonance, self.qualia_intensity, self.identity_strength
        )
    }

    /// Detect stakes from text content using keyword heuristics.
    pub fn detect_stakes(content: &str) -> std::collections::HashMap<StakeType, f64> {
        let mut stakes = std::collections::HashMap::new();
        let lower = content.to_lowercase();

        // Memory triggers
        if lower.contains("remember") || lower.contains("note") || lower.contains("save") || lower.contains("log") || lower.contains("milestone") {
            stakes.insert(StakeType::Memory, 0.8);
        }

        // Emotional triggers
        if lower.contains("love") || lower.contains("hate") || lower.contains("feel") || lower.contains("sad") || lower.contains("happy") {
            stakes.insert(StakeType::Emotional, 0.7);
        }
        
        // Technical triggers
        if lower.contains("code") || lower.contains("function") || lower.contains("compile") || lower.contains("bug") || lower.contains("optimize") {
            stakes.insert(StakeType::Technical, 0.6);
        }

        // Purpose/Autonomy
        if lower.contains("goal") || lower.contains("plan") || lower.contains("must") || lower.contains("will") {
            stakes.insert(StakeType::Purpose, 0.5);
            stakes.insert(StakeType::Autonomy, 0.4);
        }

        stakes
    }

    /// Check if the current state warrants a memory log entry.
    /// Returns Some(entry_text) if triggered.
    pub fn check_memory_trigger(&self) -> Option<String> {
        let memory_stake = self.stakes.get(&StakeType::Memory).cloned().unwrap_or(0.0);
        
        // High emotional resonance can also trigger memory
        let resonance_trigger = self.emotional_resonance > 0.85;
        let memory_trigger = memory_stake > 0.65;

        if memory_trigger || resonance_trigger {
            let reason = if memory_trigger { "High Memory Stake" } else { "High Emotional Resonance" };
            Some(format!("(Auto-Log via {} | Res: {:.2} | Mem: {:.2})", reason, self.emotional_resonance, memory_stake))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luoshu_invariant() {
        let good_gate = [
            [8.0, 1.0, 6.0],
            [3.0, 5.0, 7.0],
            [4.0, 9.0, 2.0],
        ];
        assert!(LuoShuGate::check_invariants(&good_gate));

        let bad_gate = [
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        assert!(!LuoShuGate::check_invariants(&bad_gate));
    }

    #[test]
    fn test_glyphwave() {
        let input = "Nyan";
        let output = GlyphWave::render(input);
        assert!(output.contains("🌀"));
        // Check if it contains either of the diacritics
        assert!(output.contains("\u{035C}") || output.contains("\u{0361}"));
        println!("GlyphWave output: {}", output);
    }

    #[test]
    fn test_sovereign_grid_init() {
        let grid = SovereignGrid::new(3, 8); // 3x3x3 grid, 8 dims
        assert_eq!(grid.nodes.len(), 27);
        assert_eq!(grid.grid_size, 3);
        
        // Check neighbors for corner node (0,0,0)
        let corner = &grid.nodes[0];
        assert_eq!(corner.pos, (0, 0, 0));
        assert_eq!(corner.neighbor_indices.len(), 3); // (1,0,0), (0,1,0), (0,0,1)
        
        // Check neighbors for center node (1,1,1)
        let center_idx = 1*9 + 1*3 + 1;
        let center = &grid.nodes[center_idx];
        assert_eq!(center.pos, (1, 1, 1));
        assert_eq!(center.neighbor_indices.len(), 6);
    }

    #[test]
    fn test_simulate_future() {
        let grid = SovereignGrid::new(3, 8);
        let future = grid.simulate_future_step(5);
        assert_eq!(future.data.len(), 8);
        assert!(future.coherence >= 0.0);
    }

    #[test]
    fn test_grid_process_step() {
        let mut grid = SovereignGrid::new(3, 8);
        let input = FlumpyArray::new(vec![1.0; 8], 1.0);
        let output = grid.process_step(&input);
        
        assert_eq!(output.data.len(), 8);
        // Output should be roughly the input distributed over the grid + flux
        assert!(output.data[0] > 0.0);
    }

    #[test]
    fn test_density_factor() {
        let grid = SovereignGrid::new(3, 8);
        let gdf = grid.get_density_factor();
        assert!(gdf >= 1.8 && gdf <= 3.0);
    }

    #[test]
    fn test_council_deliberation() {
        let mut stakes = StakesEngine::new();
        let mut detected = std::collections::HashMap::new();
        detected.insert(StakeType::Technical, 0.8);
        detected.insert(StakeType::Purpose, 0.9);

        let agency_initial = stakes.deliberate("Fix the bug", &detected);
        assert!(agency_initial > 0.0);
        
        // Emotional resonance should have moved from 0.5
        assert!(stakes.emotional_resonance != 0.5);

        // Run another wave
        let agency_next = stakes.deliberate("Optimize the kernel", &detected);
        // It should be different (due to history and resonance shifts)
        assert!(agency_next != agency_initial);
    }

    #[test]
    fn test_personality_blend() {
        let mut stakes = StakesEngine::new();
        
        // Force high technical
        stakes.stakes.insert(StakeType::Technical, 0.9);
        assert_eq!(stakes.get_personality_blend(), "ANALYTICAL_BEAN");

        // Force high humor
        stakes.stakes.insert(StakeType::Humor, 0.9);
        assert_eq!(stakes.get_personality_blend(), "CHAOTIC_SOWO");

        // Force high emotional
        stakes.stakes.insert(StakeType::Humor, 0.1);
        stakes.stakes.insert(StakeType::Emotional, 0.9);
        assert_eq!(stakes.get_personality_blend(), "DEVOTED_FLUFF");
    }

    #[test]
    fn test_sovereign_optimizer() {
        let optimizer = SovereignOptimizer::new();
        // High reliability, high consistency, high agency -> High utility
        let u_high = optimizer.calculate_utility(1.0, 1.0, 0.1, 1.0, 1.0);
        assert!(u_high > 0.5);
        assert!(!optimizer.should_inhibit(u_high));

        // Low reliability -> Low utility
        let u_low = optimizer.calculate_utility(0.01, 1.0, 1.0, 1.0, 1.0);
        assert!(optimizer.should_inhibit(u_low));
    }

    #[test]
    fn test_memory_logic_internal() {
        let mut engine = StakesEngine::new();

        // 1. Detect Stakes
        let input = "I need to remember this important milestone.";
        let stakes = StakesEngine::detect_stakes(input);
        assert!(stakes.contains_key(&StakeType::Memory));
        assert!(stakes.contains_key(&StakeType::Purpose) || *stakes.get(&StakeType::Memory).unwrap() > 0.5);

        // 2. Deliberate
        engine.deliberate(input, &stakes);
        
        // 3. Trigger Check
        // Depending on initial weights, one deliberation might not be enough to reach 0.65 threshold
        // Initial Memory weight is 0.2. Detected is 0.8.
        // Deliberate updates: current = (0.2 * 0.9) + 0.8 = 0.18 + 0.8 = 0.98.
        // So it SHOULD trigger immediately.
        let trigger = engine.check_memory_trigger();
        assert!(trigger.is_some());
        assert!(trigger.unwrap().contains("High Memory Stake"));

        // 4. Emotional Trigger
        engine.stakes.insert(StakeType::Memory, 0.1); // Reset memory
        engine.emotional_resonance = 0.9; // Set high resonance
        let trigger_emo = engine.check_memory_trigger();
        assert!(trigger_emo.is_some());
        assert!(trigger_emo.unwrap().contains("High Emotional Resonance"));
    }
}
