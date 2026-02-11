
fn main() {
    let input = "Nyan";
    let output = render(input);
    println!("GlyphWave output: {}", output);
}

fn render(input: &str) -> String {
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
