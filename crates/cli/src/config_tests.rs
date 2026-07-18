use std::path::PathBuf;

use super::*;

// ---------------------------------------------------------------------------
// AppConfig::load — org_policy_source (ADR-003: Org-Level Policy Configuration)
//
// `org_policy_source` is a field on `merge_warden_core::config::ApplicationDefaults`,
// exactly like every other field under `[policies]` in this file's `AppConfig::policies`.
// It must be nested under `[policies.org_policy_source]`, not a top-level
// `[org_policy_source]` table, because `AppConfig` only maps the file's `[policies]`
// table onto `ApplicationDefaults`. A regression here silently disables org-level
// policy resolution with no load error and no runtime log line — see
// ADR-003 (docs/adr/ADR-003-org-level-policy.md) for background, and PR #335
// for the production incident this caused.
// ---------------------------------------------------------------------------

fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(name);
    path
}

#[test]
fn load_defaults_org_policy_source_to_none_when_absent_from_file() {
    let path = temp_path("merge_warden_cli_no_org_policy_source_test.toml");
    std::fs::write(&path, "[policies]\nenable_title_validation = true\n").unwrap();

    let r = AppConfig::load(&path);
    let _ = std::fs::remove_file(&path);

    let config = r.expect("AppConfig::load should succeed");
    assert!(
        config.policies.org_policy_source.is_none(),
        "org_policy_source must default to None when not present in the file"
    );
}

#[test]
fn load_reads_valid_org_policy_source_from_toml_file() {
    let path = temp_path("merge_warden_cli_valid_org_policy_source_test.toml");
    std::fs::write(
        &path,
        "[policies.org_policy_source]\nowner = \"my-org\"\nrepo = \"platform-configs\"\npath = \"merge-warden/org-policy.toml\"\nfail_if_unreachable = true\n",
    )
    .unwrap();

    let r = AppConfig::load(&path);
    let _ = std::fs::remove_file(&path);

    let config = r.expect("AppConfig::load should succeed");
    let source = config
        .policies
        .org_policy_source
        .expect("org_policy_source should be Some when [policies.org_policy_source] is present");
    assert_eq!(source.owner, "my-org");
    assert_eq!(source.repo, "platform-configs");
    assert_eq!(source.path, "merge-warden/org-policy.toml");
    assert!(source.fail_if_unreachable);
}

/// Regression test for the production incident described in ADR-003
/// (docs/adr/ADR-003-org-level-policy.md) and PR #335: a top-level
/// `[org_policy_source]` table (rather than `[policies.org_policy_source]`) is
/// silently ignored by the TOML parser and must not populate
/// `ApplicationDefaults.org_policy_source`, nor cause `AppConfig::load` to fail —
/// the file is otherwise valid TOML.
#[test]
fn load_ignores_top_level_org_policy_source_table() {
    let path = temp_path("merge_warden_cli_top_level_org_policy_source_test.toml");
    std::fs::write(
        &path,
        "[org_policy_source]\nowner = \"my-org\"\nrepo = \"platform-configs\"\npath = \"merge-warden/org-policy.toml\"\n\n[policies]\nenable_title_validation = true\n",
    )
    .unwrap();

    let r = AppConfig::load(&path);
    let _ = std::fs::remove_file(&path);

    let config = r.expect("AppConfig::load should succeed");
    assert!(
        config.policies.org_policy_source.is_none(),
        "a top-level [org_policy_source] table must not populate ApplicationDefaults.org_policy_source"
    );
    assert!(
        config.policies.enable_title_validation,
        "the [policies] table alongside the misplaced top-level table should still load normally"
    );
}
