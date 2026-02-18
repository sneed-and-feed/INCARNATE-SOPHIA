
/// Analyze text for "chaos potential" (entropy, brainrot, intensity).
/// Returns a value between 0.0 and 1.0.
pub fn measure_chaos_potential(text: &str) -> f32 {
    let mut score: f32 = 0.0;
    let lower = text.to_lowercase();
    
    // 1. Caps Density (Intensity)
    let caps_count = text.chars().filter(|c| c.is_uppercase()).count();
    let total_chars = text.chars().filter(|c| c.is_alphabetic()).count();
    if total_chars > 0 {
        let caps_ratio = caps_count as f32 / total_chars as f32;
        if caps_ratio > 0.6 { score += 0.3; }
        else if caps_ratio > 0.3 { score += 0.1; }
    }

    // 2. Punctuation Intensity (???, !!!)
    if text.contains("?!") || text.contains("!?") { score += 0.2; }
    if text.matches('!').count() > 2 { score += 0.1; }

    // 3. Brainrot / Shitpost Vocabulary
    // (User requested removing specific trigger for skibidi)

    let markers = [
        "lol", "lmao", "kek", "based", "cringe", "gyatt", "riz", "ohio", 
        "sigma", "goated", "mid", "cap", "fr", "ong", "deadass", "bruh", "blud", 
        "yapping", "cook", "cooked", "locking in", "crash out", "fanum", "tax",
        "javanese", "poop", "unhinged", "deranged", "chaos", "schizo", "glowie"
    ];

    for marker in markers {
        if lower.contains(marker) {
            score += 0.15;
        }
    }

    // 4. Explicit Encouragement
    if lower.contains("do it") || lower.contains("go wild") || lower.contains("be funny") {
        score += 0.25;
    }

    // 5. Length Penalty (Long reasoned arguments are usually not chaotic)
    if text.len() > 500 {
        score -= 0.2;
    }

    score.clamp(0.0, 1.0)
}
