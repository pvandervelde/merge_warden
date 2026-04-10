//! Unit tests for the validation checks module
//!
//! These tests verify the behavior of PR title and work item reference validation functions,
//! including bypass functionality and edge cases.

use merge_warden_developer_platforms::models::{PullRequest, User};

use crate::{
    checks::{
        check_pr_title, check_work_item_reference, diagnose_pr_title,
        extract_closing_issue_reference, IssueReference, TitleDiagnosis, TitleIssue,
        TitleValidationResult,
    },
    config::{
        BypassRule, CurrentPullRequestValidationConfiguration, CONVENTIONAL_COMMIT_REGEX,
        WORK_ITEM_REGEX,
    },
    validation_result::{BypassInfo, BypassRuleType, ValidationResult},
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
        milestone_number: None,
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
        change_type_labels: None, // Use default behavior for tests
        bypass_rules: Default::default(),
        ..Default::default()
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

// Tests for extract_closing_issue_reference

/// Assertion 1: simple hash reference with closing keyword.
#[test]
fn should_parse_same_repo_hash_reference() {
    let result = extract_closing_issue_reference("fixes #42");
    assert_eq!(result, Some(IssueReference::SameRepo { issue_number: 42 }));
}

/// Assertion 2: GH-prefixed reference with closing keyword.
#[test]
fn should_parse_same_repo_gh_prefix_reference() {
    let result = extract_closing_issue_reference("closes GH-100");
    assert_eq!(result, Some(IssueReference::SameRepo { issue_number: 100 }));
}

/// Assertion 3: keyword matching is case-insensitive.
#[test]
fn should_be_case_insensitive_for_closing_keyword() {
    let result = extract_closing_issue_reference("RESOLVES #7");
    assert_eq!(result, Some(IssueReference::SameRepo { issue_number: 7 }));
}

/// Assertion 3 variant: mixed-case keyword.
#[test]
fn should_be_case_insensitive_mixed_case_keyword() {
    let result = extract_closing_issue_reference("Fixes #3");
    assert_eq!(result, Some(IssueReference::SameRepo { issue_number: 3 }));
}

/// Assertion 4: cross-repo owner/repo#NNN format.
#[test]
fn should_parse_cross_repo_owner_repo_reference() {
    let result = extract_closing_issue_reference("closes owner/repo#55");
    assert_eq!(
        result,
        Some(IssueReference::CrossRepo {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            issue_number: 55,
        })
    );
}

/// Assertion 5: full GitHub URL cross-repo reference.
#[test]
fn should_parse_cross_repo_full_github_url_reference() {
    let result = extract_closing_issue_reference("closes https://github.com/owner/repo/issues/88");
    assert_eq!(
        result,
        Some(IssueReference::CrossRepo {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            issue_number: 88,
        })
    );
}

/// Assertion 6: informational 'references' keyword is excluded.
#[test]
fn should_return_none_for_references_keyword() {
    assert_eq!(extract_closing_issue_reference("references #42"), None);
}

/// Assertion 7: informational 'relates to' keyword is excluded.
#[test]
fn should_return_none_for_relates_to_keyword() {
    assert_eq!(extract_closing_issue_reference("relates to #42"), None);
}

/// Assertion 8: empty string returns None.
#[test]
fn should_return_none_for_empty_body() {
    assert_eq!(extract_closing_issue_reference(""), None);
}

/// Assertion 9: body with no issue references at all returns None.
#[test]
fn should_return_none_when_no_references_present() {
    assert_eq!(
        extract_closing_issue_reference("This PR adds a new feature"),
        None
    );
}

/// Assertion 10: when informational reference precedes a closing reference,
/// the closing reference is returned (first *closing* keyword wins).
#[test]
fn should_skip_informational_and_return_first_closing_reference() {
    let result = extract_closing_issue_reference("references #10\nfixes #20");
    assert_eq!(result, Some(IssueReference::SameRepo { issue_number: 20 }));
}

/// Assertion 11: when multiple closing references exist, first one wins.
#[test]
fn should_return_first_of_multiple_closing_references() {
    let result = extract_closing_issue_reference("fixes #10\nfixes #20");
    assert_eq!(result, Some(IssueReference::SameRepo { issue_number: 10 }));
}

/// Extra: keyword mid-sentence is still matched.
#[test]
fn should_match_closing_keyword_mid_sentence() {
    let result = extract_closing_issue_reference("This PR fixes #99 for the user.");
    assert_eq!(result, Some(IssueReference::SameRepo { issue_number: 99 }));
}

/// Extra: multiple spaces between keyword and reference are allowed.
#[test]
fn should_handle_multiple_spaces_between_keyword_and_reference() {
    let result = extract_closing_issue_reference("fixes   #77");
    assert_eq!(result, Some(IssueReference::SameRepo { issue_number: 77 }));
}

/// Extra: issue_number() accessor returns the correct number for SameRepo.
#[test]
fn issue_number_accessor_returns_correct_number_for_same_repo() {
    let reference = IssueReference::SameRepo { issue_number: 123 };
    assert_eq!(reference.issue_number(), 123);
}

/// Extra: issue_number() accessor returns the correct number for CrossRepo.
#[test]
fn issue_number_accessor_returns_correct_number_for_cross_repo() {
    let reference = IssueReference::CrossRepo {
        owner: "org".to_string(),
        repo: "project".to_string(),
        issue_number: 456,
    };
    assert_eq!(reference.issue_number(), 456);
}

// ── TitleIssue construction tests ──────────────────────────────────────────

#[test]
fn should_construct_title_issue_empty_description() {
    let issue = TitleIssue::EmptyDescription;
    assert_eq!(issue, TitleIssue::EmptyDescription);
}

#[test]
fn should_construct_title_issue_invalid_scope() {
    let issue = TitleIssue::InvalidScope {
        scope: "Auth".to_string(),
    };
    assert!(matches!(issue, TitleIssue::InvalidScope { scope } if scope == "Auth"));
}

#[test]
fn should_construct_title_issue_leading_whitespace() {
    let issue = TitleIssue::LeadingWhitespace;
    assert_eq!(issue, TitleIssue::LeadingWhitespace);
}

#[test]
fn should_construct_title_issue_missing_colon() {
    let issue = TitleIssue::MissingColon;
    assert_eq!(issue, TitleIssue::MissingColon);
}

#[test]
fn should_construct_title_issue_missing_space_after_colon() {
    let issue = TitleIssue::MissingSpaceAfterColon;
    assert_eq!(issue, TitleIssue::MissingSpaceAfterColon);
}

#[test]
fn should_construct_title_issue_no_type_prefix() {
    let issue = TitleIssue::NoTypePrefix;
    assert_eq!(issue, TitleIssue::NoTypePrefix);
}

#[test]
fn should_construct_title_issue_unrecognized_type_with_nearest_valid() {
    let issue = TitleIssue::UnrecognizedType {
        found: "feature".to_string(),
        nearest_valid: Some("feat".to_string()),
    };
    assert!(
        matches!(issue, TitleIssue::UnrecognizedType { ref found, ref nearest_valid }
            if found == "feature" && nearest_valid.as_deref() == Some("feat"))
    );
}

#[test]
fn should_construct_title_issue_unrecognized_type_without_nearest_valid() {
    let issue = TitleIssue::UnrecognizedType {
        found: "xyz".to_string(),
        nearest_valid: None,
    };
    assert!(
        matches!(issue, TitleIssue::UnrecognizedType { ref found, ref nearest_valid }
            if found == "xyz" && nearest_valid.is_none())
    );
}

#[test]
fn should_construct_title_issue_uppercase_type() {
    let issue = TitleIssue::UppercaseType {
        found: "FEAT".to_string(),
    };
    assert!(matches!(issue, TitleIssue::UppercaseType { found } if found == "FEAT"));
}

#[test]
fn should_construct_title_issue_whitespace_before_colon() {
    let issue = TitleIssue::WhitespaceBeforeColon {
        found: "feat ".to_string(),
    };
    assert!(matches!(issue, TitleIssue::WhitespaceBeforeColon { found } if found == "feat "));
}

// ── TitleDiagnosis construction tests ──────────────────────────────────────

#[test]
fn should_construct_title_diagnosis_with_single_issue_and_fix() {
    let diagnosis = TitleDiagnosis {
        issues: vec![TitleIssue::UppercaseType {
            found: "FEAT".to_string(),
        }],
        suggested_fix: Some("feat: add login".to_string()),
    };
    assert_eq!(diagnosis.issues.len(), 1);
    assert_eq!(diagnosis.suggested_fix.as_deref(), Some("feat: add login"));
}

#[test]
fn should_construct_title_diagnosis_with_multiple_issues_and_no_fix() {
    let diagnosis = TitleDiagnosis {
        issues: vec![TitleIssue::NoTypePrefix, TitleIssue::EmptyDescription],
        suggested_fix: None,
    };
    assert_eq!(diagnosis.issues.len(), 2);
    assert!(diagnosis.suggested_fix.is_none());
}

#[test]
fn should_construct_title_diagnosis_with_no_issues() {
    let diagnosis = TitleDiagnosis {
        issues: vec![],
        suggested_fix: None,
    };
    assert!(diagnosis.issues.is_empty());
    assert!(diagnosis.suggested_fix.is_none());
}

// ── TitleValidationResult construction and delegation tests ────────────────

#[test]
fn should_construct_title_validation_result_valid_no_diagnosis() {
    let result = TitleValidationResult {
        validation: ValidationResult::valid(),
        diagnosis: None,
    };
    assert!(result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
    assert!(result.diagnosis.is_none());
}

#[test]
fn should_construct_title_validation_result_invalid_with_diagnosis() {
    let diagnosis = TitleDiagnosis {
        issues: vec![TitleIssue::NoTypePrefix],
        suggested_fix: None,
    };
    let result = TitleValidationResult {
        validation: ValidationResult::invalid(),
        diagnosis: Some(diagnosis),
    };
    assert!(!result.is_valid());
    assert!(!result.was_bypassed());
    assert!(result.bypass_info().is_none());
    assert!(result.diagnosis.is_some());
}

#[test]
fn should_construct_title_validation_result_bypassed_no_diagnosis() {
    let bypass_info = BypassInfo {
        rule_type: BypassRuleType::TitleConvention,
        user: "release-bot".to_string(),
    };
    let result = TitleValidationResult {
        validation: ValidationResult::bypassed(bypass_info.clone()),
        diagnosis: None,
    };
    assert!(result.is_valid());
    assert!(result.was_bypassed());
    assert_eq!(result.bypass_info(), Some(&bypass_info));
    assert!(result.diagnosis.is_none());
}

// ── diagnose_pr_title: 3.1 LeadingWhitespace ──────────────────────────────

#[test]
fn should_detect_leading_whitespace_and_suggest_trim() {
    let diagnosis = diagnose_pr_title(" feat: add login");
    assert!(
        diagnosis.issues.contains(&TitleIssue::LeadingWhitespace),
        "Expected LeadingWhitespace in issues: {:?}",
        diagnosis.issues
    );
    assert_eq!(
        diagnosis.suggested_fix.as_deref(),
        Some("feat: add login"),
        "Expected trimmed title as suggested_fix"
    );
}

#[test]
fn should_detect_leading_whitespace_only_once_even_with_multiple_spaces() {
    let diagnosis = diagnose_pr_title("   feat: add login");
    assert_eq!(
        diagnosis
            .issues
            .iter()
            .filter(|i| **i == TitleIssue::LeadingWhitespace)
            .count(),
        1,
        "LeadingWhitespace should appear exactly once"
    );
    assert_eq!(diagnosis.suggested_fix.as_deref(), Some("feat: add login"));
}

// ── diagnose_pr_title: 3.2 WhitespaceBeforeColon ──────────────────────────

#[test]
fn should_detect_whitespace_before_colon_with_no_scope() {
    let diagnosis = diagnose_pr_title("feat : add login");
    assert!(
        diagnosis
            .issues
            .iter()
            .any(|i| matches!(i, TitleIssue::WhitespaceBeforeColon { found } if found == "feat ")),
        "Expected WhitespaceBeforeColon {{ found: \"feat \" }}, got: {:?}",
        diagnosis.issues
    );
    assert_eq!(diagnosis.suggested_fix.as_deref(), Some("feat: add login"));
}

#[test]
fn should_detect_whitespace_before_colon_with_scope() {
    let diagnosis = diagnose_pr_title("feat(auth) : add login");
    assert!(
        diagnosis.issues.iter().any(
            |i| matches!(i, TitleIssue::WhitespaceBeforeColon { found } if found == "feat(auth) ")
        ),
        "Expected WhitespaceBeforeColon {{ found: \"feat(auth) \" }}, got: {:?}",
        diagnosis.issues
    );
    assert_eq!(
        diagnosis.suggested_fix.as_deref(),
        Some("feat(auth): add login")
    );
}

// ── diagnose_pr_title: 3.3 UppercaseType ──────────────────────────────────

#[test]
fn should_detect_fully_uppercase_type() {
    let diagnosis = diagnose_pr_title("FEAT: add login");
    assert!(
        diagnosis
            .issues
            .iter()
            .any(|i| matches!(i, TitleIssue::UppercaseType { found } if found == "FEAT")),
        "Expected UppercaseType {{ found: \"FEAT\" }}, got: {:?}",
        diagnosis.issues
    );
    assert_eq!(diagnosis.suggested_fix.as_deref(), Some("feat: add login"));
}

#[test]
fn should_detect_mixed_case_type() {
    let diagnosis = diagnose_pr_title("Fix: bug in auth");
    assert!(
        diagnosis
            .issues
            .iter()
            .any(|i| matches!(i, TitleIssue::UppercaseType { found } if found == "Fix")),
        "Expected UppercaseType {{ found: \"Fix\" }}, got: {:?}",
        diagnosis.issues
    );
    assert_eq!(diagnosis.suggested_fix.as_deref(), Some("fix: bug in auth"));
}

// ── diagnose_pr_title: 3.4 UnrecognizedType ───────────────────────────────

#[test]
fn should_detect_unrecognized_type_for_each_typo_map_entry() {
    let cases: &[(&str, &str, &str)] = &[
        ("bug: fix crash", "bug", "fix"),
        ("bugfix: fix crash", "bugfix", "fix"),
        ("dep: update packages", "dep", "chore"),
        ("dependencies: update packages", "dependencies", "chore"),
        ("enhancement: add feature", "enhancement", "feat"),
        ("feature: add login", "feature", "feat"),
        ("hotfix: urgent patch", "hotfix", "fix"),
    ];

    for (title, typo, correct) in cases {
        let diagnosis = diagnose_pr_title(title);
        assert!(
            diagnosis.issues.iter().any(|i| matches!(i,
                TitleIssue::UnrecognizedType { found, nearest_valid: Some(nv) }
                if found == typo && nv == correct
            )),
            "Title '{}': expected UnrecognizedType {{ found: {:?}, nearest_valid: Some({:?}) }}, got: {:?}",
            title, typo, correct, diagnosis.issues
        );
        // suggested_fix should replace the typo token with the correction
        let expected_fix = title.replacen(typo, correct, 1);
        assert_eq!(
            diagnosis.suggested_fix.as_deref(),
            Some(expected_fix.as_str()),
            "Title '{}': expected suggested_fix = {:?}",
            title,
            expected_fix
        );
    }
}

#[test]
fn should_detect_unrecognized_type_with_no_nearest_valid_and_no_suggested_fix() {
    let diagnosis = diagnose_pr_title("xyz: add login");
    assert!(
        diagnosis.issues.iter().any(|i| matches!(i,
            TitleIssue::UnrecognizedType { found, nearest_valid: None }
            if found == "xyz"
        )),
        "Expected UnrecognizedType {{ found: \"xyz\", nearest_valid: None }}, got: {:?}",
        diagnosis.issues
    );
    assert!(
        diagnosis.suggested_fix.is_none(),
        "Expected no suggested_fix for unknown type, got: {:?}",
        diagnosis.suggested_fix
    );
}

// ── diagnose_pr_title: 3.5 MissingColon ───────────────────────────────────

#[test]
fn should_detect_missing_colon_and_suggest_insertion() {
    let diagnosis = diagnose_pr_title("feat add login");
    assert!(
        diagnosis.issues.contains(&TitleIssue::MissingColon),
        "Expected MissingColon, got: {:?}",
        diagnosis.issues
    );
    assert_eq!(diagnosis.suggested_fix.as_deref(), Some("feat: add login"));
}

// ── diagnose_pr_title: 3.6 MissingSpaceAfterColon ─────────────────────────

#[test]
fn should_detect_missing_space_after_colon_and_suggest_insertion() {
    let diagnosis = diagnose_pr_title("feat:add login");
    assert!(
        diagnosis
            .issues
            .contains(&TitleIssue::MissingSpaceAfterColon),
        "Expected MissingSpaceAfterColon, got: {:?}",
        diagnosis.issues
    );
    assert_eq!(diagnosis.suggested_fix.as_deref(), Some("feat: add login"));
}

// ── diagnose_pr_title: 3.7 EmptyDescription ───────────────────────────────

#[test]
fn should_detect_empty_description_with_trailing_space() {
    let diagnosis = diagnose_pr_title("feat: ");
    assert!(
        diagnosis.issues.contains(&TitleIssue::EmptyDescription),
        "Expected EmptyDescription, got: {:?}",
        diagnosis.issues
    );
    assert!(
        diagnosis.suggested_fix.is_none(),
        "Expected no suggested_fix for EmptyDescription"
    );
}

#[test]
fn should_detect_empty_description_with_multiple_trailing_spaces() {
    let diagnosis = diagnose_pr_title("feat:  ");
    assert!(
        diagnosis.issues.contains(&TitleIssue::EmptyDescription),
        "Expected EmptyDescription, got: {:?}",
        diagnosis.issues
    );
    assert!(diagnosis.suggested_fix.is_none());
}

// ── diagnose_pr_title: 3.8 InvalidScope ───────────────────────────────────

#[test]
fn should_detect_uppercase_chars_in_scope_and_suggest_lowercase() {
    let diagnosis = diagnose_pr_title("feat(Auth): add login");
    assert!(
        diagnosis
            .issues
            .iter()
            .any(|i| matches!(i, TitleIssue::InvalidScope { scope } if scope == "Auth")),
        "Expected InvalidScope {{ scope: \"Auth\" }}, got: {:?}",
        diagnosis.issues
    );
    assert_eq!(
        diagnosis.suggested_fix.as_deref(),
        Some("feat(auth): add login")
    );
}

#[test]
fn should_detect_space_in_scope_and_suggest_hyphen_replacement() {
    let diagnosis = diagnose_pr_title("feat(user service): add user");
    assert!(
        diagnosis
            .issues
            .iter()
            .any(|i| matches!(i, TitleIssue::InvalidScope { scope } if scope == "user service")),
        "Expected InvalidScope {{ scope: \"user service\" }}, got: {:?}",
        diagnosis.issues
    );
    assert_eq!(
        diagnosis.suggested_fix.as_deref(),
        Some("feat(user-service): add user")
    );
}

// ── diagnose_pr_title: 3.9 NoTypePrefix ───────────────────────────────────

#[test]
fn should_detect_no_type_prefix_for_plain_sentence() {
    let diagnosis = diagnose_pr_title("Add login functionality");
    assert!(
        diagnosis.issues.contains(&TitleIssue::NoTypePrefix),
        "Expected NoTypePrefix for plain sentence, got: {:?}",
        diagnosis.issues
    );
    assert!(
        diagnosis.suggested_fix.is_none(),
        "Expected no suggested_fix for NoTypePrefix"
    );
}

#[test]
fn should_detect_no_type_prefix_for_empty_string() {
    let diagnosis = diagnose_pr_title("");
    assert!(
        diagnosis.issues.contains(&TitleIssue::NoTypePrefix),
        "Expected NoTypePrefix for empty string, got: {:?}",
        diagnosis.issues
    );
    assert!(diagnosis.suggested_fix.is_none());
}

#[test]
fn should_detect_no_type_prefix_for_whitespace_only_string() {
    let diagnosis = diagnose_pr_title("   ");
    assert!(
        diagnosis.issues.contains(&TitleIssue::LeadingWhitespace),
        "Expected LeadingWhitespace for whitespace-only string, got: {:?}",
        diagnosis.issues
    );
    assert!(
        diagnosis.issues.contains(&TitleIssue::NoTypePrefix),
        "Expected NoTypePrefix for whitespace-only string, got: {:?}",
        diagnosis.issues
    );
    assert!(diagnosis.suggested_fix.is_none());
}

// ── diagnose_pr_title: 3.10 Compound case ─────────────────────────────────

#[test]
fn should_detect_leading_whitespace_and_uppercase_type_together() {
    let diagnosis = diagnose_pr_title(" FEAT: add login");
    assert_eq!(
        diagnosis.issues,
        vec![
            TitleIssue::LeadingWhitespace,
            TitleIssue::UppercaseType {
                found: "FEAT".to_string()
            }
        ],
        "Expected [LeadingWhitespace, UppercaseType], got: {:?}",
        diagnosis.issues
    );
    assert_eq!(diagnosis.suggested_fix.as_deref(), Some("feat: add login"));
}

// ── diagnose_pr_title: 3.11 Valid title ───────────────────────────────────

#[test]
fn should_return_empty_issues_and_no_fix_for_valid_conventional_title() {
    let diagnosis = diagnose_pr_title("feat: add login");
    // diagnose_pr_title is called only for invalid titles, but a conforming title
    // should produce an empty issues list (no problems found).
    assert!(
        diagnosis.issues.is_empty(),
        "Expected no issues for valid title, got: {:?}",
        diagnosis.issues
    );
    assert!(
        diagnosis.suggested_fix.is_none(),
        "Expected no suggested_fix for valid title"
    );
}

// ── check_pr_title: 3.12 – 3.14 ───────────────────────────────────────────

#[test]
fn should_return_diagnosis_some_when_check_pr_title_with_invalid_title() {
    let pr = create_pull_request(1, "invalid title format", None, None);
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(!result.is_valid(), "Expected invalid result for bad title");
    assert!(
        result.diagnosis.is_some(),
        "Expected Some(diagnosis) for invalid title"
    );
}

#[test]
fn should_return_diagnosis_none_when_check_pr_title_with_valid_title() {
    let pr = create_pull_request(1, "feat: add new login feature", None, None);
    let bypass_rule = create_bypass_rule_disabled();
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid(), "Expected valid result for good title");
    assert!(
        result.diagnosis.is_none(),
        "Expected None diagnosis for valid title"
    );
}

#[test]
fn should_return_diagnosis_none_when_check_pr_title_with_bypassed_user() {
    let user = create_user(123, "release-bot");
    let pr = create_pull_request(1, "invalid title format", None, Some(user));
    let bypass_rule = create_bypass_rule_enabled_for_users(vec!["release-bot"]);
    let config = create_default_config();

    let result = check_pr_title(&pr, &bypass_rule, &config);

    assert!(result.is_valid(), "Expected bypassed result to be valid");
    assert!(result.was_bypassed(), "Expected bypass to have been used");
    assert!(
        result.diagnosis.is_none(),
        "Expected None diagnosis when bypassed"
    );
}
