use merge_warden_developer_platforms::models::{PullRequest, User};

use crate::{
    bypass::{
        can_bypass_title_validation, can_bypass_validations, can_bypass_work_item_validation,
    },
    config::{BypassRule, BypassRules},
};

fn create_test_user(login: &str) -> User {
    User {
        id: 123,
        login: login.to_string(),
    }
}

fn create_enabled_bypass_rule(users: Vec<&str>) -> BypassRule {
    BypassRule {
        enabled: true,
        users: users.iter().map(|u| u.to_string()).collect(),
    }
}

fn create_disabled_bypass_rule(users: Vec<&str>) -> BypassRule {
    BypassRule {
        enabled: false,
        users: users.iter().map(|u| u.to_string()).collect(),
    }
}

#[test]
fn test_can_bypass_title_validation_enabled_user_in_list() {
    let user = create_test_user("release-bot");
    let bypass_rule = create_enabled_bypass_rule(vec!["release-bot", "admin"]);

    assert!(can_bypass_title_validation(Some(&user), &bypass_rule));
}

#[test]
fn test_can_bypass_title_validation_enabled_user_not_in_list() {
    let user = create_test_user("regular-user");
    let bypass_rule = create_enabled_bypass_rule(vec!["release-bot", "admin"]);

    assert!(!can_bypass_title_validation(Some(&user), &bypass_rule));
}

#[test]
fn test_can_bypass_title_validation_disabled_user_in_list() {
    let user = create_test_user("release-bot");
    let bypass_rule = create_disabled_bypass_rule(vec!["release-bot", "admin"]);

    assert!(!can_bypass_title_validation(Some(&user), &bypass_rule));
}

#[test]
fn test_can_bypass_title_validation_no_user() {
    let bypass_rule = create_enabled_bypass_rule(vec!["release-bot", "admin"]);

    assert!(!can_bypass_title_validation(None, &bypass_rule));
}

#[test]
fn test_can_bypass_title_validation_empty_users_list() {
    let user = create_test_user("release-bot");
    let bypass_rule = create_enabled_bypass_rule(vec![]);

    assert!(!can_bypass_title_validation(Some(&user), &bypass_rule));
}

#[test]
fn test_can_bypass_work_item_validation_enabled_user_in_list() {
    let user = create_test_user("hotfix-team");
    let bypass_rule = create_enabled_bypass_rule(vec!["hotfix-team", "security-team"]);

    assert!(can_bypass_work_item_validation(Some(&user), &bypass_rule));
}

#[test]
fn test_can_bypass_work_item_validation_enabled_user_not_in_list() {
    let user = create_test_user("regular-developer");
    let bypass_rule = create_enabled_bypass_rule(vec!["hotfix-team", "security-team"]);

    assert!(!can_bypass_work_item_validation(Some(&user), &bypass_rule));
}

#[test]
fn test_can_bypass_work_item_validation_disabled_user_in_list() {
    let user = create_test_user("hotfix-team");
    let bypass_rule = create_disabled_bypass_rule(vec!["hotfix-team", "security-team"]);

    assert!(!can_bypass_work_item_validation(Some(&user), &bypass_rule));
}

#[test]
fn test_can_bypass_work_item_validation_no_user() {
    let bypass_rule = create_enabled_bypass_rule(vec!["hotfix-team", "security-team"]);

    assert!(!can_bypass_work_item_validation(None, &bypass_rule));
}

#[test]
fn test_can_bypass_work_item_validation_case_sensitive() {
    let user = create_test_user("HotFix-Team");
    let bypass_rule = create_enabled_bypass_rule(vec!["hotfix-team"]);

    // Should be case-sensitive - different case means no bypass
    assert!(!can_bypass_work_item_validation(Some(&user), &bypass_rule));
}

#[test]
fn test_can_bypass_validations_both_enabled_user_in_both_lists() {
    let pr = PullRequest {
        number: 123,
        title: "hotfix: urgent fix".to_string(),
        draft: false,
        body: Some("Emergency fix".to_string()),
        author: Some(create_test_user("admin")),
    };

    let bypass_rules = BypassRules {
        title_convention: create_enabled_bypass_rule(vec!["admin", "release-bot"]),
        work_items: create_enabled_bypass_rule(vec!["admin", "hotfix-team"]),
    };

    let (can_bypass_title, can_bypass_work_item) = can_bypass_validations(&pr, &bypass_rules);

    assert!(can_bypass_title);
    assert!(can_bypass_work_item);
}

#[test]
fn test_can_bypass_validations_user_in_title_list_only() {
    let pr = PullRequest {
        number: 123,
        title: "feat: new feature".to_string(),
        draft: false,
        body: Some("New feature implementation".to_string()),
        author: Some(create_test_user("release-bot")),
    };

    let bypass_rules = BypassRules {
        title_convention: create_enabled_bypass_rule(vec!["release-bot"]),
        work_items: create_enabled_bypass_rule(vec!["hotfix-team"]),
    };

    let (can_bypass_title, can_bypass_work_item) = can_bypass_validations(&pr, &bypass_rules);

    assert!(can_bypass_title);
    assert!(!can_bypass_work_item);
}

#[test]
fn test_can_bypass_validations_user_in_work_item_list_only() {
    let pr = PullRequest {
        number: 123,
        title: "hotfix: security fix".to_string(),
        draft: false,
        body: Some("Security vulnerability fix".to_string()),
        author: Some(create_test_user("security-team")),
    };

    let bypass_rules = BypassRules {
        title_convention: create_enabled_bypass_rule(vec!["release-bot"]),
        work_items: create_enabled_bypass_rule(vec!["security-team", "hotfix-team"]),
    };

    let (can_bypass_title, can_bypass_work_item) = can_bypass_validations(&pr, &bypass_rules);

    assert!(!can_bypass_title);
    assert!(can_bypass_work_item);
}

#[test]
fn test_can_bypass_validations_user_in_neither_list() {
    let pr = PullRequest {
        number: 123,
        title: "feat: new feature".to_string(),
        draft: false,
        body: Some("Regular feature implementation".to_string()),
        author: Some(create_test_user("regular-developer")),
    };

    let bypass_rules = BypassRules {
        title_convention: create_enabled_bypass_rule(vec!["release-bot"]),
        work_items: create_enabled_bypass_rule(vec!["security-team", "hotfix-team"]),
    };

    let (can_bypass_title, can_bypass_work_item) = can_bypass_validations(&pr, &bypass_rules);

    assert!(!can_bypass_title);
    assert!(!can_bypass_work_item);
}

#[test]
fn test_can_bypass_validations_no_author() {
    let pr = PullRequest {
        number: 123,
        title: "feat: new feature".to_string(),
        draft: false,
        body: Some("Feature implementation".to_string()),
        author: None,
    };

    let bypass_rules = BypassRules {
        title_convention: create_enabled_bypass_rule(vec!["release-bot"]),
        work_items: create_enabled_bypass_rule(vec!["security-team"]),
    };

    let (can_bypass_title, can_bypass_work_item) = can_bypass_validations(&pr, &bypass_rules);

    assert!(!can_bypass_title);
    assert!(!can_bypass_work_item);
}

#[test]
fn test_can_bypass_validations_disabled_rules() {
    let pr = PullRequest {
        number: 123,
        title: "hotfix: urgent fix".to_string(),
        draft: false,
        body: Some("Emergency fix".to_string()),
        author: Some(create_test_user("admin")),
    };

    let bypass_rules = BypassRules {
        title_convention: create_disabled_bypass_rule(vec!["admin"]),
        work_items: create_disabled_bypass_rule(vec!["admin"]),
    };

    let (can_bypass_title, can_bypass_work_item) = can_bypass_validations(&pr, &bypass_rules);

    assert!(!can_bypass_title);
    assert!(!can_bypass_work_item);
}

#[test]
fn test_can_bypass_validations_mixed_enabled_disabled() {
    let pr = PullRequest {
        number: 123,
        title: "hotfix: urgent fix".to_string(),
        draft: false,
        body: Some("Emergency fix".to_string()),
        author: Some(create_test_user("admin")),
    };

    let bypass_rules = BypassRules {
        title_convention: create_enabled_bypass_rule(vec!["admin"]),
        work_items: create_disabled_bypass_rule(vec!["admin"]),
    };

    let (can_bypass_title, can_bypass_work_item) = can_bypass_validations(&pr, &bypass_rules);

    assert!(can_bypass_title);
    assert!(!can_bypass_work_item);
}

#[test]
fn test_can_bypass_validations_multiple_users_in_lists() {
    let pr = PullRequest {
        number: 123,
        title: "chore: maintenance".to_string(),
        draft: false,
        body: Some("Regular maintenance task".to_string()),
        author: Some(create_test_user("bot-user")),
    };

    let bypass_rules = BypassRules {
        title_convention: create_enabled_bypass_rule(vec![
            "admin",
            "release-bot",
            "bot-user",
            "automation",
        ]),
        work_items: create_enabled_bypass_rule(vec!["security-team", "bot-user", "hotfix-team"]),
    };

    let (can_bypass_title, can_bypass_work_item) = can_bypass_validations(&pr, &bypass_rules);

    assert!(can_bypass_title);
    assert!(can_bypass_work_item);
}
