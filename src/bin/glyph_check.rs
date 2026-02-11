
use ironclaw::sneed_engine::GlyphWave;

fn main() {
    let input = "Nyan";
    let output = GlyphWave::render(input);
    println!("GlyphWave output: {}", output);
}
