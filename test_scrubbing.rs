// Quick test to verify the scrubbing regex works as expected
use regex::Regex;

fn main() {
    let test_cases = vec![
        "glyphwave\">Nyan!",
        "glyphwave'>Meow!",
        "[glyphwave]Purr",
        "<glyphwave>Test",
        "Normal text glyphwave\"> more text",
    ];

    // The regex pattern we're using in context_monitor.rs
    let re = Regex::new(r"(?i)glyphwave[^a-z\s]*").unwrap();

    println!("Testing glyphwave scrubbing regex:\n");
    for input in test_cases {
        let output = re.replace_all(input, "");
        println!("Input:  {}", input);
        println!("Output: {}", output);
        println!("Match:  {}\n", if input != output { "✓" } else { "✗" });
    }
}
