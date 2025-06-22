//! Bypass rule management commands for the CLI
//!
//! This module provides commands to manage bypass rules that allow specific users
//! to skip validation checks. Bypass rules are stored in the local configuration file.

use anyhow::Result;
use clap::Subcommand;
use tracing::{debug, error, info, instrument};

use crate::config::{get_config_path, AppConfig};
use crate::errors::CliError;

/// Subcommands for bypass rule management
#[derive(Subcommand, Debug)]
pub enum BypassCommands {
    /// List all bypass rules and their current status
    List {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Enable a specific bypass rule
    Enable {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,

        /// Rule type to enable (title-validation or work-item-validation)
        #[arg(value_parser = validate_rule_type)]
        rule_type: String,
    },

    /// Disable a specific bypass rule
    Disable {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,

        /// Rule type to disable (title-validation or work-item-validation)
        #[arg(value_parser = validate_rule_type)]
        rule_type: String,
    },

    /// Add users to a bypass rule's user list
    AddUser {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,

        /// Rule type to modify (title-validation or work-item-validation)
        #[arg(value_parser = validate_rule_type)]
        rule_type: String,

        /// Comma-separated list of GitHub usernames to add
        users: String,
    },

    /// Remove users from a bypass rule's user list
    RemoveUser {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,

        /// Rule type to modify (title-validation or work-item-validation)
        #[arg(value_parser = validate_rule_type)]
        rule_type: String,

        /// Comma-separated list of GitHub usernames to remove
        users: String,
    },

    /// List users for a specific bypass rule
    ListUsers {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,

        /// Rule type to query (title-validation or work-item-validation)
        #[arg(value_parser = validate_rule_type)]
        rule_type: String,
    },
}

/// Execute bypass rule management commands
#[instrument]
pub async fn execute(cmd: BypassCommands) -> Result<(), CliError> {
    match cmd {
        BypassCommands::List { path } => list_bypass_rules(path.as_deref()),
        BypassCommands::Enable { path, rule_type } => {
            enable_bypass_rule(path.as_deref(), &rule_type)
        }
        BypassCommands::Disable { path, rule_type } => {
            disable_bypass_rule(path.as_deref(), &rule_type)
        }
        BypassCommands::AddUser {
            path,
            rule_type,
            users,
        } => add_users_to_bypass_rule(path.as_deref(), &rule_type, &users),
        BypassCommands::RemoveUser {
            path,
            rule_type,
            users,
        } => remove_users_from_bypass_rule(path.as_deref(), &rule_type, &users),
        BypassCommands::ListUsers { path, rule_type } => {
            list_users_for_bypass_rule(path.as_deref(), &rule_type)
        }
    }
}

/// Validate that the rule type is one of the supported types
fn validate_rule_type(rule_type: &str) -> Result<String, String> {
    match rule_type {
        "title-validation" | "work-item-validation" => Ok(rule_type.to_string()),
        _ => Err(format!(
            "Invalid rule type '{}'. Must be 'title-validation' or 'work-item-validation'",
            rule_type
        )),
    }
}

/// List all bypass rules and their current configuration
#[instrument]
fn list_bypass_rules(path: Option<&str>) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!(message = "Listing bypass rules", path = ?config_path);

    let config = load_config_or_default(&config_path)?;

    println!("Bypass Rules Configuration");
    println!("==========================");
    println!();

    // Title validation bypass rule
    println!("Title Validation Bypass:");
    println!(
        "  Enabled: {}",
        config.policies.bypass_rules.title_convention.enabled
    );
    println!(
        "  Users: {:?}",
        config.policies.bypass_rules.title_convention.users
    );
    println!();

    // Work item validation bypass rule
    println!("Work Item Validation Bypass:");
    println!(
        "  Enabled: {}",
        config.policies.bypass_rules.work_items.enabled
    );
    println!(
        "  Users: {:?}",
        config.policies.bypass_rules.work_items.users
    );

    Ok(())
}

/// Enable a bypass rule
#[instrument]
fn enable_bypass_rule(path: Option<&str>, rule_type: &str) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!(
        message = "Enabling bypass rule",
        path = ?config_path,
        rule_type = rule_type
    );

    let mut config = load_config_or_default(&config_path)?;

    // Update the enabled field
    match rule_type {
        "title-validation" => config.policies.bypass_rules.title_convention.enabled = true,
        "work-item-validation" => config.policies.bypass_rules.work_items.enabled = true,
        _ => unreachable!(), // Already validated
    }

    // Save the updated config
    save_config(&config, &config_path)?;

    info!(
        message = "Bypass rule enabled",
        rule_type = rule_type,
        path = ?config_path
    );
    println!("Bypass rule '{}' enabled", rule_type);
    Ok(())
}

/// Disable a bypass rule
#[instrument]
fn disable_bypass_rule(path: Option<&str>, rule_type: &str) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!(
        message = "Disabling bypass rule",
        path = ?config_path,
        rule_type = rule_type
    );

    let mut config = load_config_or_default(&config_path)?;

    // Update the enabled field
    match rule_type {
        "title-validation" => config.policies.bypass_rules.title_convention.enabled = false,
        "work-item-validation" => config.policies.bypass_rules.work_items.enabled = false,
        _ => unreachable!(), // Already validated
    }

    // Save the updated config
    save_config(&config, &config_path)?;

    info!(
        message = "Bypass rule disabled",
        rule_type = rule_type,
        path = ?config_path
    );
    println!("Bypass rule '{}' disabled", rule_type);
    Ok(())
}

/// Add users to a bypass rule's user list
#[instrument]
fn add_users_to_bypass_rule(
    path: Option<&str>,
    rule_type: &str,
    users: &str,
) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    let user_list: Vec<String> = users
        .split(',')
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
        .collect();

    if user_list.is_empty() {
        return Err(CliError::InvalidArguments(
            "No valid users provided".to_string(),
        ));
    }

    debug!(
        message = "Adding users to bypass rule",
        path = ?config_path,
        rule_type = rule_type,
        users = ?user_list
    );

    let mut config = load_config_or_default(&config_path)?;

    // Get mutable reference to the appropriate rule
    let rule = match rule_type {
        "title-validation" => &mut config.policies.bypass_rules.title_convention,
        "work-item-validation" => &mut config.policies.bypass_rules.work_items,
        _ => unreachable!(), // Already validated
    };

    // Add users that aren't already in the list
    let mut added_users = Vec::new();
    for user in user_list {
        if !rule.users.contains(&user) {
            rule.users.push(user.clone());
            added_users.push(user);
        }
    }

    if added_users.is_empty() {
        println!(
            "All specified users are already in the {} bypass list",
            rule_type
        );
        return Ok(());
    }

    // Save the updated config
    save_config(&config, &config_path)?;

    info!(
        message = "Users added to bypass rule",
        rule_type = rule_type,
        added_users = ?added_users,
        path = ?config_path
    );
    println!(
        "Added users to {} bypass: {}",
        rule_type,
        added_users.join(", ")
    );
    Ok(())
}

/// Remove users from a bypass rule's user list
#[instrument]
fn remove_users_from_bypass_rule(
    path: Option<&str>,
    rule_type: &str,
    users: &str,
) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    let user_list: Vec<String> = users
        .split(',')
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
        .collect();

    if user_list.is_empty() {
        return Err(CliError::InvalidArguments(
            "No valid users provided".to_string(),
        ));
    }

    debug!(
        message = "Removing users from bypass rule",
        path = ?config_path,
        rule_type = rule_type,
        users = ?user_list
    );

    let mut config = load_config_or_default(&config_path)?;

    // Get mutable reference to the appropriate rule
    let rule = match rule_type {
        "title-validation" => &mut config.policies.bypass_rules.title_convention,
        "work-item-validation" => &mut config.policies.bypass_rules.work_items,
        _ => unreachable!(), // Already validated
    };

    // Remove users from the list
    let mut removed_users = Vec::new();
    for user in user_list {
        if let Some(pos) = rule.users.iter().position(|u| u == &user) {
            rule.users.remove(pos);
            removed_users.push(user);
        }
    }

    if removed_users.is_empty() {
        println!(
            "None of the specified users were found in the {} bypass list",
            rule_type
        );
        return Ok(());
    }

    // Save the updated config
    save_config(&config, &config_path)?;

    info!(
        message = "Users removed from bypass rule",
        rule_type = rule_type,
        removed_users = ?removed_users,
        path = ?config_path
    );
    println!(
        "Removed users from {} bypass: {}",
        rule_type,
        removed_users.join(", ")
    );
    Ok(())
}

/// List users for a specific bypass rule
#[instrument]
fn list_users_for_bypass_rule(path: Option<&str>, rule_type: &str) -> Result<(), CliError> {
    let config_path = get_config_path(path);
    debug!(
        message = "Listing users for bypass rule",
        path = ?config_path,
        rule_type = rule_type
    );

    let config = load_config_or_default(&config_path)?;

    let rule = match rule_type {
        "title-validation" => &config.policies.bypass_rules.title_convention,
        "work-item-validation" => &config.policies.bypass_rules.work_items,
        _ => unreachable!(), // Already validated
    };

    println!("{} Bypass Users:", rule_type);
    println!("Enabled: {}", rule.enabled);
    if rule.users.is_empty() {
        println!("No users configured");
    } else {
        for user in &rule.users {
            println!("  - {}", user);
        }
    }

    Ok(())
}

/// Load configuration or create default if it doesn't exist
fn load_config_or_default(config_path: &std::path::Path) -> Result<AppConfig, CliError> {
    if config_path.exists() {
        AppConfig::load(config_path).map_err(|e| {
            error!(message = "Failed to load configuration", path = ?config_path, error = ?e);
            CliError::ConfigError("Failed to load the configuration".to_string())
        })
    } else {
        debug!(message = "Configuration file not found, using defaults", path = ?config_path);
        Ok(AppConfig::default())
    }
}

/// Save configuration to file
fn save_config(config: &AppConfig, config_path: &std::path::Path) -> Result<(), CliError> {
    config.save(config_path).map_err(|e| {
        error!(message = "Failed to save configuration", path = ?config_path, error = ?e);
        CliError::ConfigError("Failed to save configuration".to_string())
    })
}

#[cfg(test)]
#[path = "bypass_tests.rs"]
mod tests;
