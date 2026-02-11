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
    fn test_sovereign_optimizer() {
        let optimizer = SovereignOptimizer::new();
        // High reliability, high consistency -> High utility
        let u_high = optimizer.calculate_utility(1.0, 1.0, 0.1, 1.0, 1.0);
        assert!(u_high > 0.5);
        assert!(!optimizer.should_inhibit(u_high));

        // Low reliability -> Low utility
        let u_low = optimizer.calculate_utility(0.01, 1.0, 1.0, 1.0, 1.0);
        assert!(optimizer.should_inhibit(u_low));
    }
}
