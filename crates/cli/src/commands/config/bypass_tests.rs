use std::fs;
use tempfile::TempDir;

use super::*;
use crate::config::AppConfig;

/// Helper to create a temporary config file
fn create_temp_config() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".merge-warden.toml");
    (temp_dir, config_path)
}

/// Helper to create a config with bypass rules
fn create_config_with_bypass_rules() -> AppConfig {
    let mut config = AppConfig::default();
    config.policies.bypass_rules.title_convention.enabled = true;
    config.policies.bypass_rules.title_convention.users =
        vec!["user1".to_string(), "user2".to_string()];
    config.policies.bypass_rules.work_items.enabled = false;
    config.policies.bypass_rules.work_items.users = vec!["user3".to_string()];
    config
}

#[tokio::test]
async fn test_list_bypass_rules_with_default_config() {
    let (_temp_dir, config_path) = create_temp_config();

    // Should work with non-existent config (uses defaults)
    let result = list_bypass_rules(Some(config_path.to_str().unwrap()));
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_bypass_rules_with_existing_config() {
    let (_temp_dir, config_path) = create_temp_config();
    let config = create_config_with_bypass_rules();
    config.save(&config_path).unwrap();

    let result = list_bypass_rules(Some(config_path.to_str().unwrap()));
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enable_bypass_rule() {
    let (_temp_dir, config_path) = create_temp_config();

    // Enable title validation bypass
    let result = enable_bypass_rule(Some(config_path.to_str().unwrap()), "title-validation");
    assert!(result.is_ok());

    // Verify it was enabled
    let config = AppConfig::load(&config_path).unwrap();
    assert!(config.policies.bypass_rules.title_convention.enabled);

    // Enable work item validation bypass
    let result = enable_bypass_rule(Some(config_path.to_str().unwrap()), "work-item-validation");
    assert!(result.is_ok());

    // Verify it was enabled
    let config = AppConfig::load(&config_path).unwrap();
    assert!(config.policies.bypass_rules.work_items.enabled);
}

#[tokio::test]
async fn test_disable_bypass_rule() {
    let (_temp_dir, config_path) = create_temp_config();
    let mut config = create_config_with_bypass_rules();
    config.save(&config_path).unwrap();

    // Disable title validation bypass
    let result = disable_bypass_rule(Some(config_path.to_str().unwrap()), "title-validation");
    assert!(result.is_ok());

    // Verify it was disabled
    let config = AppConfig::load(&config_path).unwrap();
    assert!(!config.policies.bypass_rules.title_convention.enabled);
}

#[tokio::test]
async fn test_enable_invalid_rule_type() {
    let (_temp_dir, config_path) = create_temp_config();

    // This should be caught by clap validation, but test the internal function
    // by calling validate_rule_type directly
    let result = validate_rule_type("invalid-rule");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid rule type"));
}

#[tokio::test]
async fn test_add_users_to_bypass_rule() {
    let (_temp_dir, config_path) = create_temp_config();

    // Add users to title validation bypass
    let result = add_users_to_bypass_rule(
        Some(config_path.to_str().unwrap()),
        "title-validation",
        "user1,user2,user3",
    );
    assert!(result.is_ok());

    // Verify users were added
    let config = AppConfig::load(&config_path).unwrap();
    let users = &config.policies.bypass_rules.title_convention.users;
    assert_eq!(users.len(), 3);
    assert!(users.contains(&"user1".to_string()));
    assert!(users.contains(&"user2".to_string()));
    assert!(users.contains(&"user3".to_string()));
}

#[tokio::test]
async fn test_add_duplicate_users() {
    let (_temp_dir, config_path) = create_temp_config();
    let config = create_config_with_bypass_rules();
    config.save(&config_path).unwrap();

    // Try to add users that already exist
    let result = add_users_to_bypass_rule(
        Some(config_path.to_str().unwrap()),
        "title-validation",
        "user1,user4", // user1 already exists, user4 is new
    );
    assert!(result.is_ok());

    // Verify only new user was added
    let config = AppConfig::load(&config_path).unwrap();
    let users = &config.policies.bypass_rules.title_convention.users;
    assert_eq!(users.len(), 3); // user1, user2 (existing) + user4 (new)
    assert!(users.contains(&"user4".to_string()));
}

#[tokio::test]
async fn test_add_users_with_whitespace() {
    let (_temp_dir, config_path) = create_temp_config();

    // Add users with various whitespace
    let result = add_users_to_bypass_rule(
        Some(config_path.to_str().unwrap()),
        "work-item-validation",
        " user1 , user2,  user3  ",
    );
    assert!(result.is_ok());

    // Verify users were trimmed and added
    let config = AppConfig::load(&config_path).unwrap();
    let users = &config.policies.bypass_rules.work_items.users;
    assert_eq!(users.len(), 3);
    assert!(users.contains(&"user1".to_string()));
    assert!(users.contains(&"user2".to_string()));
    assert!(users.contains(&"user3".to_string()));
}

#[tokio::test]
async fn test_add_empty_users() {
    let (_temp_dir, config_path) = create_temp_config();

    // Try to add empty user list
    let result =
        add_users_to_bypass_rule(Some(config_path.to_str().unwrap()), "title-validation", "");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CliError::InvalidArguments(_)));
}

#[tokio::test]
async fn test_remove_users_from_bypass_rule() {
    let (_temp_dir, config_path) = create_temp_config();
    let config = create_config_with_bypass_rules();
    config.save(&config_path).unwrap();

    // Remove a user from title validation bypass
    let result = remove_users_from_bypass_rule(
        Some(config_path.to_str().unwrap()),
        "title-validation",
        "user1",
    );
    assert!(result.is_ok());

    // Verify user was removed
    let config = AppConfig::load(&config_path).unwrap();
    let users = &config.policies.bypass_rules.title_convention.users;
    assert_eq!(users.len(), 1);
    assert!(!users.contains(&"user1".to_string()));
    assert!(users.contains(&"user2".to_string()));
}

#[tokio::test]
async fn test_remove_multiple_users() {
    let (_temp_dir, config_path) = create_temp_config();
    let config = create_config_with_bypass_rules();
    config.save(&config_path).unwrap();

    // Remove multiple users
    let result = remove_users_from_bypass_rule(
        Some(config_path.to_str().unwrap()),
        "title-validation",
        "user1,user2",
    );
    assert!(result.is_ok());

    // Verify users were removed
    let config = AppConfig::load(&config_path).unwrap();
    let users = &config.policies.bypass_rules.title_convention.users;
    assert_eq!(users.len(), 0);
}

#[tokio::test]
async fn test_remove_nonexistent_users() {
    let (_temp_dir, config_path) = create_temp_config();
    let config = create_config_with_bypass_rules();
    config.save(&config_path).unwrap();

    // Try to remove users that don't exist
    let result = remove_users_from_bypass_rule(
        Some(config_path.to_str().unwrap()),
        "title-validation",
        "nonexistent1,nonexistent2",
    );
    assert!(result.is_ok()); // Should succeed but do nothing

    // Verify original users are still there
    let config = AppConfig::load(&config_path).unwrap();
    let users = &config.policies.bypass_rules.title_convention.users;
    assert_eq!(users.len(), 2);
    assert!(users.contains(&"user1".to_string()));
    assert!(users.contains(&"user2".to_string()));
}

#[tokio::test]
async fn test_list_users_for_bypass_rule() {
    let (_temp_dir, config_path) = create_temp_config();
    let config = create_config_with_bypass_rules();
    config.save(&config_path).unwrap();

    // List users should succeed
    let result =
        list_users_for_bypass_rule(Some(config_path.to_str().unwrap()), "title-validation");
    assert!(result.is_ok());

    let result =
        list_users_for_bypass_rule(Some(config_path.to_str().unwrap()), "work-item-validation");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_bypass_commands_integration() {
    let (_temp_dir, config_path) = create_temp_config();
    let config_path_str = config_path.to_str().unwrap();

    // Test full workflow: enable, add users, list, remove users, disable

    // Enable bypass rule
    let cmd = BypassCommands::Enable {
        path: Some(config_path_str.to_string()),
        rule_type: "title-validation".to_string(),
    };
    assert!(execute(cmd).await.is_ok());

    // Add users
    let cmd = BypassCommands::AddUser {
        path: Some(config_path_str.to_string()),
        rule_type: "title-validation".to_string(),
        users: "user1,user2".to_string(),
    };
    assert!(execute(cmd).await.is_ok());

    // List users
    let cmd = BypassCommands::ListUsers {
        path: Some(config_path_str.to_string()),
        rule_type: "title-validation".to_string(),
    };
    assert!(execute(cmd).await.is_ok());

    // List all rules
    let cmd = BypassCommands::List {
        path: Some(config_path_str.to_string()),
    };
    assert!(execute(cmd).await.is_ok());

    // Remove a user
    let cmd = BypassCommands::RemoveUser {
        path: Some(config_path_str.to_string()),
        rule_type: "title-validation".to_string(),
        users: "user1".to_string(),
    };
    assert!(execute(cmd).await.is_ok());

    // Disable bypass rule
    let cmd = BypassCommands::Disable {
        path: Some(config_path_str.to_string()),
        rule_type: "title-validation".to_string(),
    };
    assert!(execute(cmd).await.is_ok());

    // Verify final state
    let config = AppConfig::load(&config_path).unwrap();
    assert!(!config.policies.bypass_rules.title_convention.enabled);
    assert_eq!(config.policies.bypass_rules.title_convention.users.len(), 1);
    assert!(config
        .policies
        .bypass_rules
        .title_convention
        .users
        .contains(&"user2".to_string()));
}

#[test]
fn test_validate_rule_type() {
    // Valid rule types
    assert!(validate_rule_type("title-validation").is_ok());
    assert!(validate_rule_type("work-item-validation").is_ok());

    // Invalid rule types
    assert!(validate_rule_type("invalid").is_err());
    assert!(validate_rule_type("title").is_err());
    assert!(validate_rule_type("work-item").is_err());
    assert!(validate_rule_type("").is_err());
}

#[test]
fn test_load_config_or_default() {
    let (_temp_dir, config_path) = create_temp_config();

    // Should return default for non-existent file
    let result = load_config_or_default(&config_path);
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(!config.policies.bypass_rules.title_convention.enabled);
    assert!(!config.policies.bypass_rules.work_items.enabled);
}

#[test]
fn test_save_and_load_config() {
    let (_temp_dir, config_path) = create_temp_config();
    let config = create_config_with_bypass_rules();

    // Save config
    let result = save_config(&config, &config_path);
    assert!(result.is_ok());

    // Load and verify
    let loaded_config = load_config_or_default(&config_path).unwrap();
    assert_eq!(
        loaded_config.policies.bypass_rules.title_convention.enabled,
        true
    );
    assert_eq!(
        loaded_config.policies.bypass_rules.work_items.enabled,
        false
    );
    assert_eq!(
        loaded_config
            .policies
            .bypass_rules
            .title_convention
            .users
            .len(),
        2
    );
    assert_eq!(
        loaded_config.policies.bypass_rules.work_items.users.len(),
        1
    );
}
