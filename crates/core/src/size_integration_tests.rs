//! Integration tests for PR size analysis functionality.
//!
//! These tests verify the end-to-end behavior of size analysis,
//! including file filtering and size categorization.

use super::*;
use merge_warden_developer_platforms::models::PullRequestFile;

#[test]
fn test_pr_size_info_from_files_with_exclusions() {
    let files = vec![
        PullRequestFile {
            filename: "src/main.rs".to_string(),
            additions: 10,
            deletions: 5,
            changes: 15,
            status: "modified".to_string(),
        },
        PullRequestFile {
            filename: "package-lock.json".to_string(),
            additions: 1000,
            deletions: 500,
            changes: 1500,
            status: "modified".to_string(),
        },
    ];

    let exclusion_patterns = vec!["package-lock.json".to_string()];
    let size_info = PrSizeInfo::from_files_with_exclusions(
        &files,
        &SizeThresholds::default(),
        &exclusion_patterns,
    );

    // Only the main.rs file should be included in size calculation
    assert_eq!(size_info.total_lines_changed, 15);
    assert_eq!(size_info.included_files.len(), 1);
    assert_eq!(size_info.excluded_files.len(), 1);
    assert_eq!(size_info.included_files[0].filename, "src/main.rs");
    assert_eq!(size_info.excluded_files[0].filename, "package-lock.json");
}

#[test]
fn test_pr_size_info_no_exclusions() {
    let files = vec![
        PullRequestFile {
            filename: "src/main.rs".to_string(),
            additions: 10,
            deletions: 5,
            changes: 15,
            status: "modified".to_string(),
        },
        PullRequestFile {
            filename: "src/lib.rs".to_string(),
            additions: 20,
            deletions: 10,
            changes: 30,
            status: "modified".to_string(),
        },
    ];

    let size_info = PrSizeInfo::from_files_with_exclusions(&files, &SizeThresholds::default(), &[]);

    // All files should be included
    assert_eq!(size_info.total_lines_changed, 45); // 15 + 30
    assert_eq!(size_info.included_files.len(), 2);
    assert_eq!(size_info.excluded_files.len(), 0);
}

#[test]
fn test_size_category_determination() {
    let files = vec![PullRequestFile {
        filename: "src/big_file.rs".to_string(),
        additions: 600,
        deletions: 200,
        changes: 800,
        status: "modified".to_string(),
    }];

    let size_info = PrSizeInfo::from_files_with_exclusions(&files, &SizeThresholds::default(), &[]);

    // 800 lines should be XXL with default thresholds
    assert!(size_info.is_oversized());
    assert_eq!(size_info.size_category, PrSizeCategory::XXL);
}
