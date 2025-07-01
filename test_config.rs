use merge_warden_core::config::*;
use std::fs;

fn main() {
    let toml_content =
        fs::read_to_string(".github/merge-warden.toml").expect("Failed to read config file");

    println!("TOML content:");
    println!("{}", toml_content);

    match toml::from_str::<RootConfig>(&toml_content) {
        Ok(config) => {
            println!("Configuration parsed successfully!");
            println!(
                "PR Size enabled: {}",
                config.policies.pull_requests.size_policies.enabled
            );
            println!(
                "PR Size config: {:?}",
                config.policies.pull_requests.size_policies
            );
        }
        Err(e) => {
            println!("Failed to parse configuration: {}", e);
        }
    }
}
