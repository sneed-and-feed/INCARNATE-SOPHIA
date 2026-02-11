use std::fs;
use ironclaw::sneed_engine::LuoShuGate;

fn main() {
    let gate = [
        [8.0, 1.0, 6.0],
        [3.0, 5.0, 7.0],
        [4.0, 9.0, 2.0],
    ];
    let result = if LuoShuGate::check_invariants(&gate) {
        "Burenyu! Coherence is STABLE (15.0)."
    } else {
        "HISSS! Coherence FAILED!"
    };
    fs::write("chaos_report.txt", result).unwrap();
}
