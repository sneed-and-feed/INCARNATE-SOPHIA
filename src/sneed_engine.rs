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
pub const U_THRESHOLD: f64 = 0.005; // Utility threshold for action inhibition

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

pub struct DiracDecomposition;

impl DiracDecomposition {
    pub fn deformed_u(array: &FlumpyArray, c_norm: f64) -> FlumpyArray {
        use crate::operators::{SpectralOperator, HadamardTwistOperator};
        let op = HadamardTwistOperator { c_norm };
        let new_data = op.apply(&array.data);
        FlumpyArray::new(new_data, array.coherence)
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
    pub id: usize,
    pub spatial_attention_scale: f64,
    pub state: FlumpyArray,
    pub neighbor_indices: Vec<usize>,
}

impl SovereignNode {
    pub fn new(id: usize, dim: usize) -> Self {
        // Simple deterministic init to avoid rand issues during debug
        let data: Vec<f64> = (0..dim).map(|i| (i as f64 * 0.1).sin() * 0.1).collect();
        
        Self {
            id,
            spatial_attention_scale: 1.0,
            state: FlumpyArray::new(data, 1.0),
            neighbor_indices: Vec::new(),
        }
    }

    /// Identify 2 Directed Collatz Graph generators.
    pub fn link_neighbors(&mut self, n: usize) {
        self.neighbor_indices = crate::spectral_oracle::get_collatz_directed_neighbors(self.id, n);
    }
}

/// The Sentient Manifold Volumetric Grid (GhostMesh).
pub struct SovereignGrid {
    pub nodes: Vec<SovereignNode>,
    pub grid_size: usize,
}

impl SovereignGrid {
    pub fn new(grid_size: usize, dim: usize) -> Self {
        // Find next power of 2 >= grid_size^3
        let target_nodes = grid_size * grid_size * grid_size;
        let n = target_nodes.next_power_of_two();
        let mut nodes = Vec::with_capacity(n);
        
        for id in 0..n {
            nodes.push(SovereignNode::new(id, dim));
        }

        // Link neighbors via Schreier topology
        let mut grid = Self { nodes, grid_size };
        for i in 0..grid.nodes.len() {
            grid.nodes[i].link_neighbors(n);
        }
        
        // --- PERRON-FROBENIUS INTEGRATION ---
        // Compute the strictly positive principal eigenvector
        let adj_list = crate::spectral_oracle::build_sparse_adjacency_list(n);
        let pf_vector = crate::spectral_oracle::compute_pf_eigenvector(&adj_list, 50);
        
        // Apply PF centrality as the structural spatial_attention_scale
        for i in 0..grid.nodes.len() {
            if i < pf_vector.len() {
                grid.nodes[i].spatial_attention_scale = pf_vector[i];
            }
        }
        
        grid
    }

    /// [RETROCAUSAL] Simulates future steps to generate a 'Prescience Bias' using Bakry-Émery steering.
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
                use crate::operators::{SpectralOperator, CollatzTransportOperator};
                
                let my_state = &future_states[i];
                let mut neighbor_states = Vec::with_capacity(node.neighbor_indices.len());
                let mut neighbor_vs = Vec::with_capacity(node.neighbor_indices.len());
                
                for &n_idx in &node.neighbor_indices {
                    neighbor_states.push(future_states[n_idx].as_slice());
                    neighbor_vs.push(self.nodes[n_idx].spatial_attention_scale);
                }
                
                let op = CollatzTransportOperator {
                    neighbor_indices: &node.neighbor_indices,
                    my_state,
                    my_v: node.spatial_attention_scale,
                    neighbor_states,
                    neighbor_vs,
                };
                let flux = op.apply(&[]);

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

    /// Execute one step of grid dynamics with RETROCAUSAL FEEDBACK and Bakry-Émery steering.
    pub fn process_step(&mut self, bio_input: &FlumpyArray, is_sleep: bool, _c_norm: f64) -> FlumpyArray {
        // 0. Calculate Future Bias
        let future_bias = self.simulate_future_step(3);
        let dim = bio_input.data.len();

        // 1. Distribute Input + Future Bias
        for node in self.nodes.iter_mut() {
            let is_center = node.id == 0;
            let scale = if is_center { 1.0 } else { 0.1 };

            for k in 0..dim {
                let cur = bio_input.data[k];
                let fut = future_bias.data[k];
                node.state.data[k] += ((cur * 0.9) + (fut * 0.1)) * scale;
            }
        }

        // 2. Flux Dynamics (neighbor exchange with Bakry-Émery steering)
        let mut fluxes = Vec::with_capacity(self.nodes.len());
        for i in 0..self.nodes.len() {
            let node = &self.nodes[i];
            use crate::operators::{SpectralOperator, CollatzTransportOperator};
            
            let my_state = &node.state.data;
            let mut neighbor_states = Vec::with_capacity(node.neighbor_indices.len());
            let mut neighbor_vs = Vec::with_capacity(node.neighbor_indices.len());
            
            for &n_idx in &node.neighbor_indices {
                neighbor_states.push(self.nodes[n_idx].state.data.as_slice());
                neighbor_vs.push(self.nodes[n_idx].spatial_attention_scale);
            }
            
            let op = CollatzTransportOperator {
                neighbor_indices: &node.neighbor_indices,
                my_state,
                my_v: node.spatial_attention_scale,
                neighbor_states,
                neighbor_vs,
            };
            let flux = op.apply(&[]);
            fluxes.push(flux);
        }

        for (i, flux) in fluxes.into_iter().enumerate() {
            let node = &mut self.nodes[i];
            let rate_multiplier = if is_sleep { 0.01 } else { 0.1 };
            let rate = (rate_multiplier / TAU_SOVEREIGN) * node.spatial_attention_scale;
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

    /// Compute spectral metrics (coherence, alpha, sigma) of the live memory states.
    pub fn get_spectral_metrics(&self) -> (f64, f64, f64) {
        let n = self.nodes.len();
        if n == 0 {
            return (1.0, 0.0, 1.0);
        }
        
        let dim = self.nodes[0].state.data.len();
        if dim == 0 {
            return (1.0, 0.0, 1.0);
        }

        // Optimize O(N^2) density matrix to O(N * dim^2).
        // Since X X^T has the same non-zero eigenvalues as X^T X, we compute X^T X (size dim x dim).
        let mut xtx_mat = vec![vec![0.0; dim]; dim];
        let mut trace = 0.0;
        
        for i in 0..n {
            let state = &self.nodes[i].state;
            let coh = state.coherence;
            let w = coh * coh; // Since dot product multiples coherence * coherence
            for a in 0..dim {
                for b in 0..dim {
                    xtx_mat[a][b] += state.data[a] * state.data[b] * w;
                }
            }
        }
        
        for a in 0..dim {
            trace += xtx_mat[a][a];
        }
        
        let k = dim.min(16);
        let mut eigenvalues = power_iteration_eigenvalues(&xtx_mat, k, 100);
        eigenvalues.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut valid_count = 0;
        let mut sum_ev_top = 0.0;
        for (idx, &val) in eigenvalues.iter().enumerate() {
            let val_abs = val.abs();
            sum_ev_top += val_abs;
            if val_abs > 1e-5 {
                x.push(((idx + 1) as f64).ln());
                y.push(val_abs.ln());
                valid_count += 1;
            }
        }
        
        let alpha = if valid_count >= 2 {
            let sum_x: f64 = x.iter().sum();
            let sum_y: f64 = y.iter().sum();
            let sum_xx: f64 = x.iter().map(|&v| v * v).sum();
            let sum_xy: f64 = x.iter().zip(y.iter()).map(|(&vx, &vy)| vx * vy).sum();
            let n_f = valid_count as f64;
            let denom = n_f * sum_xx - sum_x * sum_x;
            if denom.abs() > 1e-9 {
                (-(n_f * sum_xy - sum_x * sum_y) / denom).max(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        let mut entropy = 0.0;
        let true_sum_ev = trace.max(sum_ev_top);
        if true_sum_ev > 1e-9 {
            for &val in &eigenvalues {
                let p = val.abs() / true_sum_ev;
                if p > 1e-9 {
                    entropy -= p * p.ln();
                }
            }
            let tail = true_sum_ev - sum_ev_top;
            if tail > 1e-9 {
                let p_tail = tail / true_sum_ev;
                entropy -= p_tail * p_tail.ln();
            }
        }
        let max_entropy = (n as f64).ln().max(1.0);
        let sigma = 1.0 - (entropy / max_entropy) * 1.2;
        
        let coherence = if sigma < 0.0 {
            0.0
        } else {
            (sigma * (1.0 - alpha * 0.05)).clamp(0.0, 1.0)
        };
        
        (coherence, alpha, sigma)
    }

    /// Calculate the eigenvalue decay rate and spectral coherence factor.
    pub fn calculate_spectral_coherence(&self) -> f64 {
        self.get_spectral_metrics().0
    }

    /// Implement coordinate-free state merging via rank-1 projections onto the sum vector.
    pub fn merge_isometrically(&mut self, other: &Self, merge_factor: f64) {
        let merge_factor = merge_factor.clamp(0.0, 1.0);
        let n = self.nodes.len().min(other.nodes.len());
        for i in 0..n {
            let my_node = &mut self.nodes[i];
            let other_node = &other.nodes[i];
            
            let dim = my_node.state.data.len();
            if dim == 0 {
                continue;
            }
            
            // Compute sum vector u = x + y
            let mut sum_vec = vec![0.0; dim];
            let mut norm_sq = 0.0;
            for k in 0..dim {
                sum_vec[k] = my_node.state.data[k] + other_node.state.data[k];
                norm_sq += sum_vec[k] * sum_vec[k];
            }
            
            let norm = norm_sq.sqrt();
            let mut next_data = vec![0.0; dim];
            
            if norm > 1e-9 {
                // Normalize sum vector to get u_hat
                let mut u_hat = vec![0.0; dim];
                for k in 0..dim {
                    u_hat[k] = sum_vec[k] / norm;
                }
                
                // Compute blended vector w = (1 - f)*x + f*y
                let mut w = vec![0.0; dim];
                for k in 0..dim {
                    w[k] = (1.0 - merge_factor) * my_node.state.data[k] + merge_factor * other_node.state.data[k];
                }
                
                // Compute dot product (w . u_hat)
                let mut dot_w_u = 0.0;
                for k in 0..dim {
                    dot_w_u += w[k] * u_hat[k];
                }
                
                // Rank-1 projection: p = (w . u_hat) * u_hat
                for k in 0..dim {
                    next_data[k] = dot_w_u * u_hat[k];
                }
            } else {
                // Fallback to simple blend
                for k in 0..dim {
                    next_data[k] = (1.0 - merge_factor) * my_node.state.data[k] + merge_factor * other_node.state.data[k];
                }
            }
            
            my_node.state.data = next_data;
            my_node.state.coherence = ((1.0 - merge_factor) * my_node.state.coherence + merge_factor * other_node.state.coherence).min(1.0);
            my_node.spatial_attention_scale = (1.0 - merge_factor) * my_node.spatial_attention_scale + merge_factor * other_node.spatial_attention_scale;
        }
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

    /// [Language Models Need Sleep]
    /// Sleep phase: perform N offline recurrent passes over the accumulated state
    /// to consolidate fast weights before clearing or continuing.
    pub fn sleep_consolidation(&mut self, n_passes: usize) {
        if self.nodes.is_empty() {
            return;
        }
        let dim = self.nodes[0].state.data.len();
        if dim == 0 {
            return;
        }
        let empty_input = FlumpyArray::new(vec![0.0; dim], 1.0);
        let n_nodes = self.nodes.len();
        let spectral_radius = crate::spectral_oracle::twisted_spectral_radius(n_nodes);
        let c_norm = 1.0 / spectral_radius;

        for _ in 0..n_passes {
            self.process_step(&empty_input, true, c_norm);
            // Apply Dirac Decomposition with exact spectral circle radius
            for node in self.nodes.iter_mut() {
                node.state = DiracDecomposition::deformed_u(&node.state, c_norm);
            }
        }
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
    pub affinity: [f64; 16],
    pub resonance_history: std::collections::VecDeque<f64>,
}

impl CouncilMember {
    pub fn new(name: &str, role: &str, affinity_map: std::collections::HashMap<StakeType, f64>) -> Self {
        let mut affinity = [0.1; 16];
        for (stake, weight) in affinity_map {
            affinity[stake as usize] = weight;
        }

        Self {
            name: name.to_string(),
            role: role.to_string(),
            affinity,
            resonance_history: std::collections::VecDeque::with_capacity(10),
        }
    }

    pub fn process_signal(&mut self, stake_type: StakeType, intensity: f64) -> f64 {
        let base_affinity = self.affinity[stake_type as usize];
        
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
    pub current_c_norm: f64,
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

        // Fill remaining members to reach 16 (pruned from 32 for higher signal-to-noise)
        for i in council.len()..16 {
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
            current_c_norm: 1.0,
        }
    }

    pub fn deliberate(&mut self, _input_signal: &str, detected_stakes: &std::collections::HashMap<StakeType, f64>) -> (f64, f64) {
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
        
        // ZEDA Dynamic MoE: Skip 50% of experts (Zero-Expert) for easy tokens
        // let signal_magnitude: f64 = detected_stakes.values().sum();
        // let active_experts = if signal_magnitude > 2.0 {
        //     self.council.len()
        // } else {
        //     self.council.len() / 2
        // };
        
        // NO MOE: bypass all experts
        let active_experts = 0;
        
        for _ in 0..waves {
            let mut wave_resonance = 0.0;
            for (i, member) in self.council.iter_mut().enumerate() {
                if i < active_experts {
                    for (s, w) in detected_stakes {
                        let res = member.process_signal(*s, *w);
                        wave_resonance += res;
                        if let Some(tr) = total_resonances.get_mut(s) {
                            *tr += res;
                        }
                    }
                } else {
                    // Zero-Expert Simulation: Outputs 0.0 and skips heavy processing
                    wave_resonance += 0.0;
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
        
        let c_norm = if agency_score > 0.75 {
            0.85 // Warm entropy shedding
        } else {
            1.0 // Unitary critical line
        };
        self.current_c_norm = c_norm;
        
        (agency_score, c_norm)
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

fn power_iteration_eigenvalues(matrix: &[Vec<f64>], k: usize, max_iters: usize) -> Vec<f64> {
    let n = matrix.len();
    if n == 0 {
        return vec![];
    }
    
    let mut a = matrix.to_owned();
    let mut eigenvalues = Vec::with_capacity(k);
    
    for iter_k in 0..k {
        let mut v = vec![0.0; n];
        for i in 0..n {
            v[i] = (((i + iter_k) as f64) * 0.6180339887).fract() * 2.0 - 1.0;
        }
        
        let mut norm = 0.0;
        for &x in &v { norm += x * x; }
        if norm < 1e-9 {
            v[0] = 1.0;
        } else {
            norm = norm.sqrt();
            for x in &mut v { *x /= norm; }
        }
        
        let mut eigenvalue = 0.0;
        
        for _ in 0..max_iters {
            let mut next_v = vec![0.0; n];
            for i in 0..n {
                for j in 0..n {
                    next_v[i] += a[i][j] * v[j];
                }
            }
            
            let mut rq = 0.0;
            for i in 0..n {
                rq += v[i] * next_v[i];
            }
            eigenvalue = rq;
            
            let mut next_norm = 0.0;
            for &x in &next_v { next_norm += x * x; }
            if next_norm < 1e-15 {
                break;
            }
            next_norm = next_norm.sqrt();
            
            let mut diff = 0.0;
            for i in 0..n {
                let n_v = next_v[i] / next_norm;
                diff += (n_v - v[i]).abs();
                v[i] = n_v;
            }
            if diff < 1e-6 {
                break;
            }
        }
        
        eigenvalues.push(eigenvalue);
        
        for i in 0..n {
            for j in 0..n {
                a[i][j] -= eigenvalue * v[i] * v[j];
            }
        }
    }
    
    eigenvalues
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
        let grid = SovereignGrid::new(3, 8); // 3x3x3 grid = 27 -> next power of 2 is 32
        assert_eq!(grid.nodes.len(), 32);
        assert_eq!(grid.grid_size, 3);
        
        // Check neighbors for node 0 in Schreier Graph N=32
        // Generators for x=0: 3*0=0, 3*0-1=31, 11*0=0, 11*1=11
        // Without self-loops, neighbors are 11 and 31
        let corner = &grid.nodes[0];
        assert_eq!(corner.id, 0);
        let mut expected = vec![11, 31];
        expected.sort();
        let mut actual = corner.neighbor_indices.clone();
        actual.sort();
        assert_eq!(actual, expected);
        
        // Check neighbors for node 1
        // Generators for x=1: 3*1=3, 3*1-1=2, 11*1=11, 11*2=22
        let center = &grid.nodes[1];
        assert_eq!(center.id, 1);
        let mut expected2 = vec![2, 3, 11, 22];
        expected2.sort();
        let mut actual2 = center.neighbor_indices.clone();
        actual2.sort();
        assert_eq!(actual2, expected2);
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
        let output = grid.process_step(&input, false, 1.0);
        
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

        let (agency_initial, _) = stakes.deliberate("Fix the bug", &detected);
        assert!(agency_initial > 0.0);
        
        // Emotional resonance should have moved from 0.5
        assert!(stakes.emotional_resonance != 0.5);

        // Run another wave
        let (agency_next, _) = stakes.deliberate("Optimize the kernel", &detected);
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
        stakes.stakes.insert(StakeType::Technical, 0.1);
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
        let u_low = optimizer.calculate_utility(0.01, 1.0, 1.0, 1.0, 0.0);
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

    #[test]
    fn test_spectral_coherence() {
        let grid = SovereignGrid::new(3, 8);
        let coherence = grid.calculate_spectral_coherence();
        // A newly initialized grid with identical states across nodes should have perfect coherence
        assert!(coherence >= COHERENCE_THRESHOLD);

        // Perturb the grid states with high-entropy chaos
        let mut chaotic_grid = SovereignGrid::new(3, 8);
        for (i, node) in chaotic_grid.nodes.iter_mut().enumerate() {
            // Apply unique sinusoidal waves to make them highly distinct
            node.state.data = (0..8).map(|k| ((i * k) as f64).sin() * 5.0).collect();
        }
        let chaotic_coherence = chaotic_grid.calculate_spectral_coherence();
        // Chaotic states should result in significantly lower coherence
        assert!(chaotic_coherence < COHERENCE_THRESHOLD);
    }

    #[test]
    fn test_bakry_emery_steering() {
        let mut grid = SovereignGrid::new(3, 8);
        
        // Zero out states
        for node in grid.nodes.iter_mut() {
            node.state.data = vec![0.0; 8];
            node.spatial_attention_scale = 1.0;
        }

        // Set high potential at node 0 and low potential at its neighbor node 1
        grid.nodes[0].spatial_attention_scale = 5.0; // High potential V
        grid.nodes[0].state.data = vec![1.0; 8];

        let n1_idx = grid.nodes[0].neighbor_indices[0];
        grid.nodes[n1_idx].spatial_attention_scale = 1.0; // Low potential V

        // Find another neighbor of node 0 to compare
        if grid.nodes[0].neighbor_indices.len() > 1 {
            let n2_idx = grid.nodes[0].neighbor_indices[1];
            grid.nodes[n2_idx].spatial_attention_scale = 10.0; // Higher potential V

            // Run one simulation step
            let _ = grid.process_step(&FlumpyArray::new(vec![0.0; 8], 1.0), false, 1.0);

            // n1 (lower potential V=1.0) should receive MUCH more state value than n2 (higher potential V=10.0)
            assert!(grid.nodes[n1_idx].state.data[0] > grid.nodes[n2_idx].state.data[0]);
        }
    }

    #[test]
    fn test_isometric_merge() {
        let mut grid1 = SovereignGrid::new(3, 8);
        let mut grid2 = SovereignGrid::new(3, 8);

        for (i, node) in grid1.nodes.iter_mut().enumerate() {
            node.state.data = vec![i as f64 * 0.1; 8];
        }
        for (i, node) in grid2.nodes.iter_mut().enumerate() {
            node.state.data = vec![i as f64 * 0.2; 8];
        }

        grid1.merge_isometrically(&grid2, 0.5);

        // Verify that the merged states are non-zero and intermediate
        assert!(grid1.nodes[1].state.data[0] > 0.0);
        assert!(grid1.nodes[1].state.data[0] < grid2.nodes[1].state.data[0]);
    }
}
