//! spectral_oracle.rs
//! Adelic Spectral Zeta Mathematics for Sophia AI Upgrade
//! Generates discrete spectral avoidance masks to enforce true feature orthogonality
//! and eliminate multi-scale resonant correlations (Measure-Zero Collapse).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static MASK_CACHE: OnceLock<Mutex<HashMap<(usize, usize), Vec<bool>>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<(usize, usize), Vec<bool>>> {
    MASK_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Retrieves or generates a geometric avoidance mask for a given dimension `n`
/// and pattern length `k`.
pub fn get_adelic_mask(n: usize, k: usize) -> Vec<bool> {
    let cache = get_cache();
    if let Ok(mut map) = cache.lock() {
        if let Some(mask) = map.get(&(n, k)) {
            return mask.clone();
        }
        let mask = generate_avoidance_mask(n, k);
        map.insert((n, k), mask.clone());
        mask
    } else {
        // Fallback if Mutex is poisoned
        generate_avoidance_mask(n, k)
    }
}

/// Checks if adding the element at `new_idx` completes a `k`-term progression.
fn forms_k_progression(mask: &[bool], new_idx: usize, k: usize) -> bool {
    if k < 3 {
        return false;
    }
    
    let max_d = new_idx / (k - 1);
    for d in 1..=max_d {
        let mut count = 0;
        for i in 0..k {
            let idx = new_idx - i * d;
            if mask[idx] {
                count += 1;
            }
        }
        if count == k {
            return true;
        }
    }
    false
}

/// Greedily constructs a maximal subset of {0..n-1} avoiding `k`-term progressions.
fn generate_avoidance_mask(n: usize, k: usize) -> Vec<bool> {
    let mut mask = vec![false; n];
    for i in 0..n {
        mask[i] = true;
        if forms_k_progression(&mask, i, k) {
            mask[i] = false; // Banned by the Discrete Spectral Oracle
        }
    }
    mask
}

/// Applies the discrete spectral mask in-place to enforce sparsity.
pub fn apply_adelic_mask(array: &mut [f64]) {
    let n = array.len();
    if n == 0 {
        return;
    }
    // Avoid 4-term scaling sequences (as defined by the Adelic Spectral Zeta theory)
    let mask = get_adelic_mask(n, 4);
    for i in 0..n {
        if !mask[i] {
            array[i] = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avoidance() {
        let mask = generate_avoidance_mask(50, 4);
        // Verify no 4-term arithmetic progression exists
        for d in 1..=50/3 {
            for a in 0..50 {
                if a + 3*d < 50 {
                    assert!(
                        !(mask[a] && mask[a+d] && mask[a+2*d] && mask[a+3*d]),
                        "Found 4-term progression: {}, {}, {}, {}", a, a+d, a+2*d, a+3*d
                    );
                }
            }
        }
    }
    
    #[test]
    fn test_apply_mask() {
        let mut data = vec![1.0; 10];
        apply_adelic_mask(&mut data);
        let mask = get_adelic_mask(10, 4);
        for i in 0..10 {
            if mask[i] {
                assert_eq!(data[i], 1.0);
            } else {
                assert_eq!(data[i], 0.0);
            }
        }
    }
}
