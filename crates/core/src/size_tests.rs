use super::*;

#[test]
fn test_pr_size_category_from_line_count_xs() {
    assert_eq!(PrSizeCategory::from_line_count(0), PrSizeCategory::XS);
    assert_eq!(PrSizeCategory::from_line_count(1), PrSizeCategory::XS);
    assert_eq!(PrSizeCategory::from_line_count(5), PrSizeCategory::XS);
    assert_eq!(PrSizeCategory::from_line_count(10), PrSizeCategory::XS);
}

#[test]
fn test_pr_size_category_from_line_count_s() {
    assert_eq!(PrSizeCategory::from_line_count(11), PrSizeCategory::S);
    assert_eq!(PrSizeCategory::from_line_count(25), PrSizeCategory::S);
    assert_eq!(PrSizeCategory::from_line_count(50), PrSizeCategory::S);
}

#[test]
fn test_pr_size_category_from_line_count_m() {
    assert_eq!(PrSizeCategory::from_line_count(51), PrSizeCategory::M);
    assert_eq!(PrSizeCategory::from_line_count(75), PrSizeCategory::M);
    assert_eq!(PrSizeCategory::from_line_count(100), PrSizeCategory::M);
}

#[test]
fn test_pr_size_category_from_line_count_l() {
    assert_eq!(PrSizeCategory::from_line_count(101), PrSizeCategory::L);
    assert_eq!(PrSizeCategory::from_line_count(150), PrSizeCategory::L);
    assert_eq!(PrSizeCategory::from_line_count(250), PrSizeCategory::L);
}

#[test]
fn test_pr_size_category_from_line_count_xl() {
    assert_eq!(PrSizeCategory::from_line_count(251), PrSizeCategory::XL);
    assert_eq!(PrSizeCategory::from_line_count(350), PrSizeCategory::XL);
    assert_eq!(PrSizeCategory::from_line_count(500), PrSizeCategory::XL);
}

#[test]
fn test_pr_size_category_from_line_count_xxl() {
    assert_eq!(PrSizeCategory::from_line_count(501), PrSizeCategory::XXL);
    assert_eq!(PrSizeCategory::from_line_count(1000), PrSizeCategory::XXL);
    assert_eq!(PrSizeCategory::from_line_count(u32::MAX), PrSizeCategory::XXL);
}

#[test]
fn test_pr_size_category_from_line_count_with_custom_thresholds() {
    let custom_thresholds = SizeThresholds {
        xs: 5,
        s: 20,
        m: 50,
        l: 100,
        xl: 200,
    };

    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(3, &custom_thresholds), PrSizeCategory::XS);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(5, &custom_thresholds), PrSizeCategory::XS);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(6, &custom_thresholds), PrSizeCategory::S);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(20, &custom_thresholds), PrSizeCategory::S);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(25, &custom_thresholds), PrSizeCategory::M);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(75, &custom_thresholds), PrSizeCategory::L);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(150, &custom_thresholds), PrSizeCategory::XL);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(250, &custom_thresholds), PrSizeCategory::XXL);
}

#[test]
fn test_pr_size_category_as_str() {
    assert_eq!(PrSizeCategory::XS.as_str(), "XS");
    assert_eq!(PrSizeCategory::S.as_str(), "S");
    assert_eq!(PrSizeCategory::M.as_str(), "M");
    assert_eq!(PrSizeCategory::L.as_str(), "L");
    assert_eq!(PrSizeCategory::XL.as_str(), "XL");
    assert_eq!(PrSizeCategory::XXL.as_str(), "XXL");
}

#[test]
fn test_pr_size_category_display() {
    assert_eq!(format!("{}", PrSizeCategory::XS), "XS");
    assert_eq!(format!("{}", PrSizeCategory::S), "S");
    assert_eq!(format!("{}", PrSizeCategory::M), "M");
    assert_eq!(format!("{}", PrSizeCategory::L), "L");
    assert_eq!(format!("{}", PrSizeCategory::XL), "XL");
    assert_eq!(format!("{}", PrSizeCategory::XXL), "XXL");
}

#[test]
fn test_pr_size_category_is_oversized() {
    assert!(!PrSizeCategory::XS.is_oversized());
    assert!(!PrSizeCategory::S.is_oversized());
    assert!(!PrSizeCategory::M.is_oversized());
    assert!(!PrSizeCategory::L.is_oversized());
    assert!(!PrSizeCategory::XL.is_oversized());
    assert!(PrSizeCategory::XXL.is_oversized());
}

#[test]
fn test_pr_size_category_ordering() {
    // Test that categories are properly ordered from smallest to largest
    assert!(PrSizeCategory::XS < PrSizeCategory::S);
    assert!(PrSizeCategory::S < PrSizeCategory::M);
    assert!(PrSizeCategory::M < PrSizeCategory::L);
    assert!(PrSizeCategory::L < PrSizeCategory::XL);
    assert!(PrSizeCategory::XL < PrSizeCategory::XXL);
}

#[test]
fn test_size_thresholds_default() {
    let thresholds = SizeThresholds::default();
    assert_eq!(thresholds.xs, 10);
    assert_eq!(thresholds.s, 50);
    assert_eq!(thresholds.m, 100);
    assert_eq!(thresholds.l, 250);
    assert_eq!(thresholds.xl, 500);
}

#[test]
fn test_size_thresholds_custom() {
    let thresholds = SizeThresholds {
        xs: 5,
        s: 25,
        m: 75,
        l: 200,
        xl: 400,
    };

    // Test boundaries with custom thresholds
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(5, &thresholds), PrSizeCategory::XS);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(6, &thresholds), PrSizeCategory::S);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(25, &thresholds), PrSizeCategory::S);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(26, &thresholds), PrSizeCategory::M);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(401, &thresholds), PrSizeCategory::XXL);
}

#[test]
fn test_pr_size_info_new_empty() {
    let size_info = PrSizeInfo::new(vec![], vec![], &SizeThresholds::default());

    assert_eq!(size_info.total_lines_changed, 0);
    assert_eq!(size_info.size_category, PrSizeCategory::XS);
    assert_eq!(size_info.included_file_count(), 0);
    assert_eq!(size_info.excluded_file_count(), 0);
    assert!(!size_info.is_oversized());
}

#[test]
fn test_pr_size_info_new_single_file() {
    let file = PullRequestFile {
        filename: "src/main.rs".to_string(),
        additions: 15,
        deletions: 5,
        changes: 20,
        status: "modified".to_string(),
    };

    let size_info = PrSizeInfo::new(vec![file], vec![], &SizeThresholds::default());

    assert_eq!(size_info.total_lines_changed, 20);
    assert_eq!(size_info.size_category, PrSizeCategory::S);
    assert_eq!(size_info.included_file_count(), 1);
    assert_eq!(size_info.excluded_file_count(), 0);
    assert!(!size_info.is_oversized());
}

#[test]
fn test_pr_size_info_new_multiple_files() {
    let files = vec![
        PullRequestFile {
            filename: "src/main.rs".to_string(),
            additions: 15,
            deletions: 5,
            changes: 20,
            status: "modified".to_string(),
        },
        PullRequestFile {
            filename: "src/lib.rs".to_string(),
            additions: 30,
            deletions: 10,
            changes: 40,
            status: "modified".to_string(),
        },
        PullRequestFile {
            filename: "tests/test.rs".to_string(),
            additions: 25,
            deletions: 0,
            changes: 25,
            status: "added".to_string(),
        },
    ];

    let size_info = PrSizeInfo::new(files, vec![], &SizeThresholds::default());

    assert_eq!(size_info.total_lines_changed, 85); // 20 + 40 + 25
    assert_eq!(size_info.size_category, PrSizeCategory::M);
    assert_eq!(size_info.included_file_count(), 3);
    assert_eq!(size_info.excluded_file_count(), 0);
    assert!(!size_info.is_oversized());
}

#[test]
fn test_pr_size_info_new_with_excluded_files() {
    let included_files = vec![
        PullRequestFile {
            filename: "src/main.rs".to_string(),
            additions: 10,
            deletions: 5,
            changes: 15,
            status: "modified".to_string(),
        },
    ];

    let excluded_files = vec![
        PullRequestFile {
            filename: "package-lock.json".to_string(),
            additions: 1000,
            deletions: 500,
            changes: 1500,
            status: "modified".to_string(),
        },
        PullRequestFile {
            filename: "docs/generated.md".to_string(),
            additions: 200,
            deletions: 0,
            changes: 200,
            status: "added".to_string(),
        },
    ];

    let size_info = PrSizeInfo::new(included_files, excluded_files, &SizeThresholds::default());

    // Should only count included files
    assert_eq!(size_info.total_lines_changed, 15);
    assert_eq!(size_info.size_category, PrSizeCategory::S);
    assert_eq!(size_info.included_file_count(), 1);
    assert_eq!(size_info.excluded_file_count(), 2);
    assert!(!size_info.is_oversized());
}

#[test]
fn test_pr_size_info_oversized() {
    let files = vec![
        PullRequestFile {
            filename: "src/large_file.rs".to_string(),
            additions: 400,
            deletions: 200,
            changes: 600,
            status: "modified".to_string(),
        },
    ];

    let size_info = PrSizeInfo::new(files, vec![], &SizeThresholds::default());

    assert_eq!(size_info.total_lines_changed, 600);
    assert_eq!(size_info.size_category, PrSizeCategory::XXL);
    assert_eq!(size_info.included_file_count(), 1);
    assert!(size_info.is_oversized());
}

#[test]
fn test_pr_size_info_with_custom_thresholds() {
    let custom_thresholds = SizeThresholds {
        xs: 5,
        s: 15,
        m: 30,
        l: 60,
        xl: 120,
    };

    let files = vec![
        PullRequestFile {
            filename: "src/file.rs".to_string(),
            additions: 20,
            deletions: 10,
            changes: 30,
            status: "modified".to_string(),
        },
    ];

    let size_info = PrSizeInfo::new(files, vec![], &custom_thresholds);

    assert_eq!(size_info.total_lines_changed, 30);
    assert_eq!(size_info.size_category, PrSizeCategory::M); // Exactly at the M threshold
    assert!(!size_info.is_oversized());
}

#[test]
fn test_pr_size_category_serialization() {
    // Test that PrSizeCategory can be serialized and deserialized
    use serde_json::{from_str, to_string};

    let category = PrSizeCategory::M;
    let json_str = to_string(&category).expect("Failed to serialize PrSizeCategory");
    let deserialized: PrSizeCategory = from_str(&json_str).expect("Failed to deserialize PrSizeCategory");

    assert_eq!(category, deserialized);
}

#[test]
fn test_size_thresholds_serialization() {
    // Test that SizeThresholds can be serialized and deserialized
    use serde_json::{from_str, to_string};

    let thresholds = SizeThresholds {
        xs: 5,
        s: 25,
        m: 75,
        l: 200,
        xl: 400,
    };

    let json_str = to_string(&thresholds).expect("Failed to serialize SizeThresholds");
    let deserialized: SizeThresholds = from_str(&json_str).expect("Failed to deserialize SizeThresholds");

    assert_eq!(thresholds, deserialized);
}

#[test]
fn test_boundary_conditions() {
    // Test exact boundary conditions between categories
    let thresholds = SizeThresholds::default();

    // Test XS/S boundary
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(10, &thresholds), PrSizeCategory::XS);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(11, &thresholds), PrSizeCategory::S);

    // Test S/M boundary
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(50, &thresholds), PrSizeCategory::S);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(51, &thresholds), PrSizeCategory::M);

    // Test M/L boundary
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(100, &thresholds), PrSizeCategory::M);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(101, &thresholds), PrSizeCategory::L);

    // Test L/XL boundary
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(250, &thresholds), PrSizeCategory::L);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(251, &thresholds), PrSizeCategory::XL);

    // Test XL/XXL boundary
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(500, &thresholds), PrSizeCategory::XL);
    assert_eq!(PrSizeCategory::from_line_count_with_thresholds(501, &thresholds), PrSizeCategory::XXL);
}

#[test]
fn test_pr_size_info_zero_changes_file() {
    // Test files with zero changes (edge case)
    let files = vec![
        PullRequestFile {
            filename: "unchanged.txt".to_string(),
            additions: 0,
            deletions: 0,
            changes: 0,
            status: "unchanged".to_string(),
        },
        PullRequestFile {
            filename: "src/code.rs".to_string(),
            additions: 5,
            deletions: 2,
            changes: 7,
            status: "modified".to_string(),
        },
    ];

    let size_info = PrSizeInfo::new(files, vec![], &SizeThresholds::default());

    assert_eq!(size_info.total_lines_changed, 7); // Only the modified file counts
    assert_eq!(size_info.size_category, PrSizeCategory::XS);
    assert_eq!(size_info.included_file_count(), 2);
}
