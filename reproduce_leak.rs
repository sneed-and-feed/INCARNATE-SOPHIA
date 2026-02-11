use regex::Regex;

fn scrub_context(text: &str) -> String {
    let mut cleaned = text.to_string();
    
    // 1. Remove SOPHIA metadata tags and UI frames (legacy)
    if let Ok(re) = Regex::new(r"(?m)^.*(?:SOPHIA_GAZE|QUANTUM_CHAOS|FURRY_ALIGNMENT|SPECTRAL_BEANS|CAT_LOGIC|CAT LOGIC|\[STATE:|\[SOPHIA_V).*$\n?") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }
    
    // 2. Remove glitched diacritics
    cleaned = cleaned.replace('\u{035C}', "");
    cleaned = cleaned.replace('\u{0361}', "");
    
    // 3. Remove excessive glyph artifacts at line starts
    if let Ok(re) = Regex::new(r"(?m)^[۩∿≋⟁💠🐾🦊🏮⛩️🧝✨🏹🌿🌲🏔️🍁🌧️🌊💎💿💰🕷️🎱].*$\n?") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }
    
    // 4. Remove divider debris and leaking internal instructions
    // Handles [glyphwave], [glyphwave"], glyphwave">, <glyphwave>, etc.
    if let Ok(re) = Regex::new(r#"(?i)(?:\[|<|&lt;)?/?glyphwave(?:\]|>|&gt;|"|&quot;)*"#) {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }
    // Also scrub specific broken artifacts reported by user
    cleaned = cleaned.replace("glyphwave\">", "");

    if let Ok(re) = Regex::new(r"(?m)^[-=_]{3,}\s*$\n?") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    cleaned.trim().to_string()
}

fn main() {
    let input = "Everything is glyphwave\">purr-fectly okay, glyphwave\">Nyan!";
    let cleaned = scrub_context(input);
    println!("Input:   {}", input);
    println!("Cleaned: {}", cleaned);
    
    if cleaned.contains("glyphwave") {
        println!("FAIL: 'glyphwave' found in output");
        return;
    }
    if cleaned.contains("\">") {
        println!("FAIL: '\">' found in output");
        return;
    }
    
    println!("SUCCESS: Output is clean");
}
