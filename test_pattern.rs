use regex::Regex;

fn main() {
    // Test the regex pattern that should match the bot-test repository labels
    let labels = vec![
        ("Extra Small", Some("An extra small item (size: XS)".to_string())),
        ("Small", Some("A small item (size: S)".to_string())),
        ("Medium", Some("A medium item (size: M)".to_string())),
        ("Large", Some("A large item (size: L)".to_string())),
        ("Extra Large", Some("An extra large item (size: XL)".to_string())),
        ("Way to big", Some("An extra extra large item (size: XXL)".to_string())),
    ];

    let categories = ["XS", "S", "M", "L", "XL", "XXL"];

    for category in &categories {
        println!("\n=== Testing category: {} ===", category);
        let pattern = format!(r"(?i)\(size:\s*{}\)", regex::escape(category));
        println!("Pattern: {}", pattern);
        
        let regex = Regex::new(&pattern).unwrap();
        
        for (name, description) in &labels {
            if let Some(desc) = description {
                let matches = regex.is_match(desc);
                println!("  {} ({}): {}", name, desc, if matches { "✓ MATCH" } else { "✗ NO MATCH" });
                
                if matches {
                    println!("    --> Found size label for category {}: {}", category, name);
                }
            }
        }
    }
}
