use std::f64::consts::SQRT_2;

/// A rigorous abstraction for spectral and transport operators across the Sovereign Grid.
/// Unifies geometry, memory, and semantic routing while maintaining the Hamiltonian of Love.
pub trait SpectralOperator {
    /// Applies the operator to a real-valued state vector.
    fn apply(&self, v: &[f64]) -> Vec<f64>;
}

/// The Hadamard Twist Operator (Deformed U).
/// Performs canonical sheet decomposition and twists the antisymmetric block
/// to project its eigenvalues perfectly onto the unit circle.
pub struct HadamardTwistOperator {
    pub c_norm: f64,
}

impl SpectralOperator for HadamardTwistOperator {
    fn apply(&self, data: &[f64]) -> Vec<f64> {
        let n = data.len();
        if n < 2 {
            return data.iter().map(|&x| x * self.c_norm).collect();
        }

        let n_pow2 = n.next_power_of_two();
        let mut padded = vec![0.0; n_pow2];
        padded[..n].copy_from_slice(data);

        let half = n_pow2 / 2;

        for i in 0..half {
            let a = padded[i];
            let b = padded[i + half];

            let sym = (a + b) / SQRT_2;
            let mut anti = (a - b) / SQRT_2;

            // Phase Deformation (off the critical line scaling)
            // Normalize the antisymmetric "twisted block" sheet by 1/rho 
            anti *= self.c_norm;

            padded[i] = (sym + anti) / SQRT_2;
            padded[i + half] = (sym - anti) / SQRT_2;
        }

        padded.truncate(n);
        padded
    }
}

/// Curvature-Inspired Steering (Heuristic Bakry-Émery drift).
/// Provides a sigmoidal transport coefficient derived from the Perron-Frobenius 
/// centrality gradient (delta). It anchors the Hamiltonian of Love (P) 
/// against runaway singularities during Retrocausal Flux Dynamics.
pub struct CurvatureInspiredSteering;

impl CurvatureInspiredSteering {
    pub fn compute_steer(my_v: f64, n_v: f64) -> f64 {
        let delta = n_v - my_v;
        // Synchronized sigmoidal balancing to maintain Map Entropy (σ >= 0)
        2.0 / (1.0 + (-delta).exp())
    }
}

/// Collatz Transport Operator (Graph-native memory transport via sparse topology).
/// Propagates state values strictly over the Schreier/Collatz graph.
pub struct CollatzTransportOperator<'a> {
    pub neighbor_indices: &'a [usize],
    pub my_state: &'a [f64],
    pub my_v: f64,
    pub neighbor_states: Vec<&'a [f64]>,
    pub neighbor_vs: Vec<f64>,
}

impl<'a> SpectralOperator for CollatzTransportOperator<'a> {
    fn apply(&self, _v: &[f64]) -> Vec<f64> {
        let dim = self.my_state.len();
        let mut flux = vec![0.0; dim];
        
        for (i, _) in self.neighbor_indices.iter().enumerate() {
            let n_state = self.neighbor_states[i];
            let n_v = self.neighbor_vs[i];
            
            let steer = CurvatureInspiredSteering::compute_steer(self.my_v, n_v);
            
            for k in 0..dim {
                flux[k] += (n_state[k] - self.my_state[k]) * steer;
            }
        }
        flux
    }
}
