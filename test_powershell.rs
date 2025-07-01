use std::process::Command;

fn main() {
    println!("Testing regex pattern with PowerShell...");

    // Test if the pattern works
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(
            r#"
            $text = "An extra small item (size: XS)"
            $pattern = "(?i)\(size:\s*XS\)"
            if ($text -match $pattern) {
                Write-Output "MATCH: Pattern '$pattern' matches text '$text'"
            } else {
                Write-Output "NO MATCH: Pattern '$pattern' does not match text '$text'"
            }
        "#,
        )
        .output()
        .expect("Failed to execute command");

    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    println!("Error: {}", String::from_utf8_lossy(&output.stderr));
}
