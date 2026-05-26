//! spectral_oracle.rs
//! Schreier Graph Connectivity & Spectral Oracles for Sophia AI
//! Generates Ramanujan-like expander topologies and exact sheet splitting over ZMod(2^(d-1)).

/// Computes the modular inverse of `a` modulo `m` using the Extended Euclidean Algorithm.
/// Panics if `a` and `m` are not coprime (which won't happen for 3 and 2^k).
pub fn mod_inverse(a: i64, m: i64) -> i64 {
    let mut t = 0;
    let mut newt = 1;
    let mut r = m;
    let mut newr = a;

    while newr != 0 {
        let quotient = r / newr;
        let temp_t = t - quotient * newt;
        t = newt;
        newt = temp_t;
        let temp_r = r - quotient * newr;
        r = newr;
        newr = temp_r;
    }

    if r > 1 {
        panic!("a is not invertible modulo m");
    }

    if t < 0 {
        t += m;
    }
    t
}

/// Generates the 4 geometric generator neighbors for a node `x` in the Schreier Graph G_d.
/// N must be 2^(d-1). The generators are: 3x, 3x-1, 3^(-1)x, 3^(-1)(x+1) mod N.
pub fn get_schreier_neighbors(x: usize, n: usize) -> Vec<usize> {
    if n < 2 {
        return vec![];
    }
    
    let x_i64 = x as i64;
    let n_i64 = n as i64;
    
    let inv3 = mod_inverse(3, n_i64);
    
    let n1 = (3 * x_i64).rem_euclid(n_i64) as usize;
    let n2 = (3 * x_i64 - 1).rem_euclid(n_i64) as usize;
    let n3 = (inv3 * x_i64).rem_euclid(n_i64) as usize;
    let n4 = (inv3 * (x_i64 + 1)).rem_euclid(n_i64) as usize;
    
    let mut neighbors = vec![n1, n2, n3, n4];
    
    // The graph must be loopless (x != y), and we deduplicate multiple edges
    neighbors.retain(|&y| y != x);
    neighbors.sort_unstable();
    neighbors.dedup();
    
    neighbors
}

/// Builds the explicit unweighted adjacency matrix for the underlying Schreier graph.
pub fn build_adjacency_matrix(n: usize) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        let neighbors = get_schreier_neighbors(i, n);
        for &j in &neighbors {
            matrix[i][j] = 1.0;
        }
    }
    matrix
}

/// Computes the Perron-Frobenius principal eigenvector via power iteration.
/// Returns the vector normalized so that its mean is 1.0.
pub fn compute_pf_eigenvector(matrix: &[Vec<f64>], iterations: usize) -> Vec<f64> {
    let n = matrix.len();
    if n == 0 { return vec![]; }
    
    // Start with uniform strictly positive vector
    let mut v = vec![1.0; n];
    
    for _ in 0..iterations {
        let mut next_v = vec![0.0; n];
        let mut norm = 0.0;
        for i in 0..n {
            for j in 0..n {
                next_v[i] += matrix[i][j] * v[j];
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
    fn test_mod_inverse() {
        assert_eq!(mod_inverse(3, 8), 3); // 3 * 3 = 9 = 1 mod 8
        assert_eq!(mod_inverse(3, 16), 11); // 3 * 11 = 33 = 1 mod 16
    }
    
    #[test]
    fn test_schreier_neighbors() {
        let neighbors = get_schreier_neighbors(1, 8);
        assert!(!neighbors.contains(&1));
    }
}
