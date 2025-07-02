//! Unit tests for the validation checks module
//!
//! These tests verify the behavior of PR title and work item reference validation functions,
//! including bypass functionality and edge cases.

use merge_warden_developer_platforms::models::{PullRequest, User};

use crate::{
    checks::{check_pr_title, check_work_item_reference},
    config::{
        BypassRule, CurrentPullRequestValidationConfiguration, CONVENTIONAL_COMMIT_REGEX,
        WORK_ITEM_REGEX,
    },
    validation_result::BypassRuleType,
};

// Helper functions for creating test data

fn create_user(id: u64, login: &str) -> User {
    User {
        id,
        login: login.to_string(),
    }
}

fn create_pull_request(
    number: u64,
    title: &str,
    body: Option<&str>,
    author: Option<User>,
) -> PullRequest {
    PullRequest {
        number,
        title: title.to_string(),
        draft: false,
        body: body.map(|b| b.to_string()),
        author,
    }
}

fn create_bypass_rule_disabled() -> BypassRule {
    BypassRule::new(false, Vec::<String>::new())
}

fn create_bypass_rule_enabled_for_users(users: Vec<&str>) -> BypassRule {
    BypassRule::new(true, users.into_iter().map(|u| u.to_string()).collect())
}

fn create_default_config() -> CurrentPullRequestValidationConfiguration {
    CurrentPullRequestValidationConfiguration {
        enforce_title_convention: true,
        title_pattern: CONVENTIONAL_COMMIT_REGEX.to_string(),
        invalid_title_label: Some("invalid-title".to_string()),
        enforce_work_item_references: true,
        work_item_reference_pattern: WORK_ITEM_REGEX.to_string(),
        missing_work_item_label: Some("missing-work-item".to_string()),
        pr_size_check: crate::config::PrSizeCheckConfig::default(),
        bypass_rules: Default::default(),
    }
}

fn create_config_with_invalid_title_regex() -> CurrentPullRequestValidationConfiguration {
    let mut config = create_default_config();
    config.title_pattern = r"[invalid regex(".to_string(); // Invalid regex
    config
}

fn create_config_with_invalid_work_item_regex() -> CurrentPullRequestValidationConfiguration {
    let mut config = create_default_config();
    config.work_item_reference_pattern = r"[invalid regex(".to_string(); // Invalid regex
    config
}

// Tests for check_pr_title function

#[test]
fn should_return_bypassed_when_author_can_bypass_title_validation_with_invalid_title() {
    let user = create_user(123, "bypass-user");
    let pr = create_pull_request(1, "invalid title format", None, Some(user.clone()));
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["bypass-user"]);
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(result.was_bypassed());
    let bypass_info = result.bypass_info().unwrap();
    assert_eq!(bypass_info.rule_type, BypassRuleType::TitleConvention);
    assert_eq!(bypass_info.user, "bypass-user");
}

#[test]
fn should_return_bypassed_when_author_can_bypass_title_validation_with_valid_title() {
    let user = create_user(123, "bypass-user");
    let pr = create_pull_request(1, "feat: add new feature", None, Some(user.clone()));
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["bypass-user"]);
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(result.was_bypassed());
    let bypass_info = result.bypass_info().unwrap();
    assert_eq!(bypass_info.rule_type, BypassRuleType::TitleConvention);
    assert_eq!(bypass_info.user, "bypass-user");
}

#[test]
fn should_return_invalid_when_author_cannot_bypass_title_validation_with_invalid_title() {
    let user = create_user(123, "regular-user");
    let pr = create_pull_request(1, "invalid title format", None, Some(user));
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["different-user"]);
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_author_is_none_with_bypass_enabled() {
    let pr = create_pull_request(1, "invalid title format", None, None);
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["any-user"]);
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_bypass_is_disabled_with_invalid_title() {
    let user = create_user(123, "any-user");
    let pr = create_pull_request(1, "invalid title format", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_regex_compilation_fails() {
    let user = create_user(123, "any-user");
    let pr = create_pull_request(1, "feat: add new feature", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_config_with_invalid_title_regex();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_build_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "build: update dependency versions", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_chore_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "chore: update build scripts", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_ci_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "ci: add new pipeline step", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_docs_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "docs: update API documentation", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_feat_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat: add user authentication", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_feat_type_with_scope() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat(auth): add GitHub OAuth integration",
        None,
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_feat_type_with_breaking_change() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat!: remove deprecated API endpoints",
        None,
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_feat_type_with_scope_and_breaking_change(
) {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat(api)!: change response format", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_fix_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "fix: resolve login bug", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_fix_type_with_scope() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "fix(auth): handle expired tokens correctly",
        None,
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_perf_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "perf: optimize database queries", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_refactor_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "refactor: reorganize code structure", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_revert_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "revert: undo previous changes", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_style_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "style: fix code formatting", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_test_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "test: add missing unit tests", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_with_complex_scope() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat(user-service): add profile management",
        None,
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_with_underscores_in_scope() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "fix(api_client): handle connection timeout",
        None,
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_title_matches_conventional_format_with_numbers_in_scope() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat(v2api): implement new endpoints", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_title_has_incorrect_case() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "FEAT: add new feature", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_title_has_invalid_type() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feature: add new functionality", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_title_missing_colon() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat add new feature", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_title_missing_space_after_colon() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat:add new feature", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_title_has_empty_description() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat: ", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_title_has_uppercase_scope() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat(AUTH): add authentication", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_title_has_spaces_in_scope() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat(user auth): add authentication", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_title_is_completely_invalid() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "This is just a random title", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

// Tests for check_work_item_reference function

#[test]
fn should_return_bypassed_when_author_can_bypass_work_item_validation_with_invalid_body() {
    let user = create_user(123, "bypass-user");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("No work item reference"),
        Some(user.clone()),
    );
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["bypass-user"]);
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(result.was_bypassed());
    let bypass_info = result.bypass_info().unwrap();
    assert_eq!(bypass_info.rule_type, BypassRuleType::WorkItemReference);
    assert_eq!(bypass_info.user, "bypass-user");
}

#[test]
fn should_return_bypassed_when_author_can_bypass_work_item_validation_with_no_body() {
    let user = create_user(123, "bypass-user");
    let pr = create_pull_request(1, "feat: add feature", None, Some(user.clone()));
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["bypass-user"]);
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(result.was_bypassed());
    let bypass_info = result.bypass_info().unwrap();
    assert_eq!(bypass_info.rule_type, BypassRuleType::WorkItemReference);
    assert_eq!(bypass_info.user, "bypass-user");
}

#[test]
fn should_return_bypassed_when_author_can_bypass_work_item_validation_with_valid_body() {
    let user = create_user(123, "bypass-user");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This fixes #123"),
        Some(user.clone()),
    );
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["bypass-user"]);
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(result.was_bypassed());
    let bypass_info = result.bypass_info().unwrap();
    assert_eq!(bypass_info.rule_type, BypassRuleType::WorkItemReference);
    assert_eq!(bypass_info.user, "bypass-user");
}

#[test]
fn should_return_invalid_when_author_cannot_bypass_work_item_validation_with_invalid_body() {
    let user = create_user(123, "regular-user");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("No work item reference"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["different-user"]);
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_author_is_none_with_work_item_bypass_enabled() {
    let pr = create_pull_request(1, "feat: add feature", Some("No work item reference"), None);
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["any-user"]);
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_body_is_none_with_bypass_disabled() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat: add feature", None, Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_work_item_bypass_is_disabled_with_invalid_body() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("No work item reference"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_work_item_regex_compilation_fails() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat: add feature", Some("This fixes #123"), Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_config_with_invalid_work_item_regex();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_fixes_with_hash_reference() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This change fixes #123"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_fixes_with_hash_reference_case_insensitive() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This change FIXES #123"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_closes_with_hash_reference() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat: add feature", Some("This closes #456"), Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_closes_with_hash_reference_case_insensitive() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat: add feature", Some("This Closes #456"), Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_resolves_with_hash_reference() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This resolves #789"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_references_with_hash_reference() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This references #101"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_relates_to_with_hash_reference() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This relates to #202"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_fixes_with_gh_reference() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This fixes GH-303"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_fixes_with_full_github_url() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This fixes https://github.com/owner/repo/issues/404"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_fixes_with_owner_repo_format() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This fixes owner/repo#505"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_multiple_work_item_references() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This fixes #123 and also relates to #456"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_work_item_reference_with_extra_whitespace() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This   fixes    #606"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_valid_when_body_contains_work_item_reference_with_newlines() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This is a great feature.\n\nFixes #707"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_body_has_no_work_item_reference() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This is just a description"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_body_has_invalid_keyword() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This handles #808"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_body_has_keyword_without_reference() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(
        1,
        "feat: add feature",
        Some("This fixes something"),
        Some(user),
    );
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_body_has_malformed_reference() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat: add feature", Some("This fixes #abc"), Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_body_has_reference_without_keyword() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat: add feature", Some("See issue #909"), Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}

#[test]
fn should_return_invalid_when_body_is_empty_string() {
    let user = create_user(123, "developer");
    let pr = create_pull_request(1, "feat: add feature", Some(""), Some(user));
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_work_item_reference(&pr, &bypass_rule, &config);

    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
}
