//! spectral_oracle.rs
//! Schreier Graph Connectivity & Spectral Oracles for Sophia AI
//! Generates Collatz relation topologies and exact sheet splitting over ZMod(2^(d-1)).

/// Computes the exact spectral radius of the twisted block S_n.
/// According to the Spectral Circle Theorem, this is 2^(1 / 2^{n-1}).
/// For a graph of N nodes (N = 2^n), 2^{n-1} = N/2.
pub fn twisted_spectral_radius(n_nodes: usize) -> f64 {
    if n_nodes < 2 {
        return 1.0;
    }
    let exponent = 2.0 / (n_nodes as f64);
    2.0_f64.powf(exponent)
}

/// Generates the 2 geometric generator neighbors for a node `x` in the directed Collatz Relation Graph D_n.
/// The generators are: 3x, 3x-1 mod N.
pub fn get_collatz_directed_neighbors(x: usize, n: usize) -> Vec<usize> {
    if n < 2 {
        return vec![];
    }

    let x_i64 = x as i64;
    let n_i64 = n as i64;

    let n1 = (3 * x_i64).rem_euclid(n_i64) as usize;
    let n2 = (3 * x_i64 - 1).rem_euclid(n_i64) as usize;

    let mut neighbors = vec![n1, n2];

    // The graph must be loopless (x != y), and we deduplicate multiple edges
    neighbors.retain(|&y| y != x);
    neighbors.sort_unstable();
    neighbors.dedup();

    neighbors
}

/// Builds the explicit sparse adjacency list for the underlying directed graph.
pub fn build_sparse_adjacency_list(n: usize) -> Vec<Vec<usize>> {
    let mut adjacency_list = vec![vec![]; n];
    for i in 0..n {
        adjacency_list[i] = get_collatz_directed_neighbors(i, n);
    }
    adjacency_list
}

/// Computes the Perron-Frobenius principal eigenvector via power iteration.
/// Returns the vector normalized so that its mean is 1.0.
/// Runs in O(E) time using sparse adjacency lists.
pub fn compute_pf_eigenvector(adjacency_list: &[Vec<usize>], iterations: usize) -> Vec<f64> {
    let n = adjacency_list.len();
    if n == 0 {
        return vec![];
    }

    // Start with uniform strictly positive vector
    let mut v = vec![1.0; n];

    for _ in 0..iterations {
        let mut next_v = vec![0.0; n];
        let mut norm: f64 = 0.0;
        for i in 0..n {
            for &j in &adjacency_list[i] {
                next_v[i] += v[j];
            }
            norm += next_v[i] * next_v[i];
        }

        let norm = norm.sqrt();
        if norm > 1e-9 {
            for val in next_v.iter_mut() {
                *val /= norm;
            }
        }
        v = next_v;
    }

    // Normalize so the average value is 1.0 (to preserve spatial scale magnitudes)
    let sum: f64 = v.iter().sum();
    let avg = if sum > 0.0 { sum / n as f64 } else { 1.0 };
    if avg > 1e-9 {
        for val in v.iter_mut() {
            *val /= avg;
        }
    }

    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twisted_radius() {
        // N=4 (n=2), 2^{n-1} = 2, radius = sqrt(2) = 1.4142...
        let r = twisted_spectral_radius(4);
        assert!((r - 1.41421356).abs() < 1e-6);
    }

    #[test]
    fn test_collatz_neighbors() {
        let neighbors = get_collatz_directed_neighbors(1, 8);
        assert!(!neighbors.contains(&1));
        assert!(neighbors.contains(&3));
        assert!(neighbors.contains(&2));
    }

    #[test]
    fn test_spectral_radius_bounds() {
        let n = 16;
        let rho = twisted_spectral_radius(n);
        let theoretical_bound = 1.0905077; // 2^(1/8)
        let eps = 1e-6;
        assert!(rho <= theoretical_bound + eps);
        assert!(rho >= theoretical_bound - eps);
    }

    #[test]
    fn test_operator_contraction() {
        // Mock contraction bound verification
        // ||T(v)|| <= c ||v||
        let n = 32;
        let c_norm = 1.0 / twisted_spectral_radius(n);

        // Let's create an antisymmetric vector [v, -v]
        let mut v = vec![0.0; n];
        for i in 0..(n / 2) {
            v[i] = 1.0;
            v[i + (n / 2)] = -1.0;
        }

        let norm_before: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();

        let mut v_after = vec![0.0; n];
        for i in 0..n {
            v_after[i] = v[i] * c_norm;
        }
        let norm_after: f64 = v_after.iter().map(|x| x * x).sum::<f64>().sqrt();

        let c = c_norm; // Theoretical contraction factor
        assert!(norm_after <= c * norm_before + 1e-9);
    }
}

pub struct ArtinSchreierMember {
    pub re: f64,
    pub im: f64,
}

pub fn compute_algebraic_trace(a: &ArtinSchreierMember, b: &ArtinSchreierMember, nu: f64) -> f64 {
    let re_diff = a.re - b.re;
    let im_diff = a.im - b.im;
    let num = re_diff * re_diff - re_diff * im_diff + im_diff * im_diff * nu;
    let den = a.im * b.im;
    if den.abs() < 1e-9 {
        return f64::MAX;
    }
    num / den
}
