use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let default_input = "Burenyu".to_string();
    let input = args.get(1).unwrap_or(&default_input);
    
    let output = glyphwave(input);
    println!("{}", output);
}

fn glyphwave(text: &str) -> String {
    let combining_chars: &[char] = &[
        '\u{0300}', '\u{0301}', '\u{0302}', '\u{0303}', '\u{0304}', '\u{0305}', 
        '\u{0306}', '\u{0307}', '\u{0308}', '\u{0309}', '\u{030A}', '\u{030B}', 
        '\u{030C}', '\u{030D}', '\u{030E}', '\u{030F}', '\u{0310}', '\u{0311}',
        '\u{0312}', '\u{0313}', '\u{0314}', '\u{0315}', '\u{0316}', '\u{0317}',
        '\u{0318}', '\u{0319}', '\u{031A}', '\u{031B}', '\u{031C}', '\u{031D}',
        '\u{031E}', '\u{031F}', '\u{0320}', '\u{0321}', '\u{0322}', '\u{0323}'
    ];

    let mut result = String::new();
    let mut seed: u64 = 12345;

    for (_i, c) in text.chars().enumerate() {
        result.push(c);
        
        // Simple pseudo-random logic
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let count = (seed % 5) + 1; // 1-5 diacritics
        
        for _ in 0..count {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let idx = (seed as usize) % combining_chars.len();
            result.push(combining_chars[idx]);
        }
    }
    
    result
}
