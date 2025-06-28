//! # PR Size Analysis
//!
//! This module contains data structures and logic for analyzing pull request sizes
//! and categorizing them based on the number of lines changed.
//!
//! PR size analysis helps development teams maintain better code quality through
//! more manageable pull request sizes, with research showing that review effectiveness
//! decreases significantly for larger PRs.

use merge_warden_developer_platforms::models::PullRequestFile;
use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "size_tests.rs"]
mod tests;

/// Represents the size category of a pull request based on lines changed.
///
/// These categories are based on industry research showing that smaller PRs
/// are reviewed more effectively and have lower defect rates.
///
/// # Categories
///
/// * `XS` - 1-10 lines: Trivial changes, very easy to review
/// * `S` - 11-50 lines: Small changes, easy to review thoroughly
/// * `M` - 51-100 lines: Medium changes, manageable review scope
/// * `L` - 101-250 lines: Large changes, approaching review complexity limits
/// * `XL` - 251-500 lines: Extra large changes, difficult to review effectively
/// * `XXL` - 500+ lines: Should be split for better reviewability
///
/// # Examples
///
/// ```
/// use merge_warden_core::size::PrSizeCategory;
///
/// // Categorize based on line count
/// let small_pr = PrSizeCategory::from_line_count(45);
/// assert_eq!(small_pr, PrSizeCategory::S);
///
/// let large_pr = PrSizeCategory::from_line_count(300);
/// assert_eq!(large_pr, PrSizeCategory::XL);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PrSizeCategory {
    /// 1-10 lines: Trivial changes
    XS,
    /// 11-50 lines: Small changes
    S,
    /// 51-100 lines: Medium changes
    M,
    /// 101-250 lines: Large changes
    L,
    /// 251-500 lines: Extra large changes
    XL,
    /// 500+ lines: Should be split
    XXL,
}

impl PrSizeCategory {
    /// Determine the size category from the total number of lines changed.
    ///
    /// Uses the standard thresholds defined in the industry research on
    /// effective PR review sizes.
    ///
    /// # Arguments
    ///
    /// * `line_count` - The total number of lines changed (additions + deletions)
    ///
    /// # Returns
    ///
    /// The appropriate `PrSizeCategory` for the given line count
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::size::PrSizeCategory;
    ///
    /// assert_eq!(PrSizeCategory::from_line_count(5), PrSizeCategory::XS);
    /// assert_eq!(PrSizeCategory::from_line_count(25), PrSizeCategory::S);
    /// assert_eq!(PrSizeCategory::from_line_count(75), PrSizeCategory::M);
    /// assert_eq!(PrSizeCategory::from_line_count(150), PrSizeCategory::L);
    /// assert_eq!(PrSizeCategory::from_line_count(300), PrSizeCategory::XL);
    /// assert_eq!(PrSizeCategory::from_line_count(600), PrSizeCategory::XXL);
    /// ```
    pub fn from_line_count(line_count: u32) -> Self {
        match line_count {
            0..=10 => PrSizeCategory::XS,
            11..=50 => PrSizeCategory::S,
            51..=100 => PrSizeCategory::M,
            101..=250 => PrSizeCategory::L,
            251..=500 => PrSizeCategory::XL,
            _ => PrSizeCategory::XXL,
        }
    }

    /// Determine the size category using configurable thresholds.
    ///
    /// This allows repositories to customize their size categories based on
    /// their specific needs and team preferences.
    ///
    /// # Arguments
    ///
    /// * `line_count` - The total number of lines changed
    /// * `thresholds` - Custom thresholds for each category
    ///
    /// # Returns
    ///
    /// The appropriate `PrSizeCategory` for the given line count and thresholds
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::size::{PrSizeCategory, SizeThresholds};
    ///
    /// let custom_thresholds = SizeThresholds {
    ///     xs: 5,
    ///     s: 25,
    ///     m: 75,
    ///     l: 200,
    ///     xl: 400,
    /// };
    ///
    /// assert_eq!(
    ///     PrSizeCategory::from_line_count_with_thresholds(30, &custom_thresholds),
    ///     PrSizeCategory::M
    /// );
    /// ```
    pub fn from_line_count_with_thresholds(line_count: u32, thresholds: &SizeThresholds) -> Self {
        match line_count {
            count if count <= thresholds.xs => PrSizeCategory::XS,
            count if count <= thresholds.s => PrSizeCategory::S,
            count if count <= thresholds.m => PrSizeCategory::M,
            count if count <= thresholds.l => PrSizeCategory::L,
            count if count <= thresholds.xl => PrSizeCategory::XL,
            _ => PrSizeCategory::XXL,
        }
    }

    /// Get the display name for the size category.
    ///
    /// Returns a human-readable string representation of the category.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::size::PrSizeCategory;
    ///
    /// assert_eq!(PrSizeCategory::XS.as_str(), "XS");
    /// assert_eq!(PrSizeCategory::XXL.as_str(), "XXL");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            PrSizeCategory::XS => "XS",
            PrSizeCategory::S => "S",
            PrSizeCategory::M => "M",
            PrSizeCategory::L => "L",
            PrSizeCategory::XL => "XL",
            PrSizeCategory::XXL => "XXL",
        }
    }

    /// Check if this size category indicates an oversized PR.
    ///
    /// Returns true for XXL category, which indicates a PR that should
    /// be split into smaller changes for better reviewability.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::size::PrSizeCategory;
    ///
    /// assert!(!PrSizeCategory::XL.is_oversized());
    /// assert!(PrSizeCategory::XXL.is_oversized());
    /// ```
    pub fn is_oversized(&self) -> bool {
        matches!(self, PrSizeCategory::XXL)
    }
}

impl std::fmt::Display for PrSizeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Configurable thresholds for PR size categorization.
///
/// Allows teams to customize the line count thresholds that determine
/// which size category a PR falls into based on their workflow and
/// review practices.
///
/// # Examples
///
/// ```
/// use merge_warden_core::size::SizeThresholds;
///
/// // Default thresholds based on industry research
/// let standard = SizeThresholds::default();
/// assert_eq!(standard.s, 50);
///
/// // Custom thresholds for a more conservative team
/// let conservative = SizeThresholds {
///     xs: 5,
///     s: 20,
///     m: 50,
///     l: 100,
///     xl: 200,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SizeThresholds {
    /// Maximum lines for XS category (default: 10)
    pub xs: u32,
    /// Maximum lines for S category (default: 50)
    pub s: u32,
    /// Maximum lines for M category (default: 100)
    pub m: u32,
    /// Maximum lines for L category (default: 250)
    pub l: u32,
    /// Maximum lines for XL category (default: 500)
    pub xl: u32,
    // Note: XXL is anything above xl threshold
}

impl Default for SizeThresholds {
    /// Create default thresholds based on industry research.
    ///
    /// These values are based on studies showing optimal PR sizes
    /// for effective code review and defect detection.
    fn default() -> Self {
        Self {
            xs: 10,
            s: 50,
            m: 100,
            l: 250,
            xl: 500,
        }
    }
}

/// Comprehensive information about a pull request's size and file changes.
///
/// Contains the calculated size metrics, categorization, and detailed
/// information about which files were included or excluded from the
/// size calculation.
///
/// # Examples
///
/// ```
/// use merge_warden_core::size::{PrSizeInfo, PrSizeCategory};
/// use merge_warden_developer_platforms::models::PullRequestFile;
///
/// let files = vec![
///     PullRequestFile {
///         filename: "src/main.rs".to_string(),
///         additions: 15,
///         deletions: 5,
///         changes: 20,
///         status: "modified".to_string(),
///     },
/// ];
///
/// let size_info = PrSizeInfo {
///     total_lines_changed: 20,
///     included_files: files,
///     excluded_files: vec![],
///     size_category: PrSizeCategory::XS,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PrSizeInfo {
    /// Total lines changed (additions + deletions) excluding filtered files
    pub total_lines_changed: u32,

    /// List of files included in the size calculation
    pub included_files: Vec<PullRequestFile>,

    /// List of files excluded from the size calculation (e.g., generated files)
    pub excluded_files: Vec<PullRequestFile>,

    /// The determined size category based on total lines changed
    pub size_category: PrSizeCategory,
}

impl PrSizeInfo {
    /// Create a new `PrSizeInfo` with the given files and thresholds.
    ///
    /// Automatically calculates the total lines changed and determines
    /// the appropriate size category.
    ///
    /// # Arguments
    ///
    /// * `included_files` - Files to include in size calculation
    /// * `excluded_files` - Files excluded from size calculation
    /// * `thresholds` - Size category thresholds to use
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::size::{PrSizeInfo, SizeThresholds};
    /// use merge_warden_developer_platforms::models::PullRequestFile;
    ///
    /// let files = vec![
    ///     PullRequestFile {
    ///         filename: "src/lib.rs".to_string(),
    ///         additions: 10,
    ///         deletions: 5,
    ///         changes: 15,
    ///         status: "modified".to_string(),
    ///     },
    /// ];
    ///
    /// let size_info = PrSizeInfo::new(
    ///     files,
    ///     vec![],
    ///     &SizeThresholds::default()
    /// );
    ///
    /// assert_eq!(size_info.total_lines_changed, 15);
    /// ```
    pub fn new(
        included_files: Vec<PullRequestFile>,
        excluded_files: Vec<PullRequestFile>,
        thresholds: &SizeThresholds,
    ) -> Self {
        let total_lines_changed: u32 = included_files.iter().map(|f| f.changes).sum();
        let size_category =
            PrSizeCategory::from_line_count_with_thresholds(total_lines_changed, thresholds);

        Self {
            total_lines_changed,
            included_files,
            excluded_files,
            size_category,
        }
    }

    /// Check if this PR is considered oversized based on its category.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::size::{PrSizeInfo, PrSizeCategory, SizeThresholds};
    ///
    /// let large_size_info = PrSizeInfo {
    ///     total_lines_changed: 600,
    ///     included_files: vec![],
    ///     excluded_files: vec![],
    ///     size_category: PrSizeCategory::XXL,
    /// };
    ///
    /// assert!(large_size_info.is_oversized());
    /// ```
    pub fn is_oversized(&self) -> bool {
        self.size_category.is_oversized()
    }

    /// Get the number of files included in the size calculation.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::size::PrSizeInfo;
    /// use merge_warden_developer_platforms::models::PullRequestFile;
    ///
    /// let files = vec![
    ///     PullRequestFile {
    ///         filename: "file1.rs".to_string(),
    ///         additions: 10,
    ///         deletions: 0,
    ///         changes: 10,
    ///         status: "added".to_string(),
    ///     },
    ///     PullRequestFile {
    ///         filename: "file2.rs".to_string(),
    ///         additions: 5,
    ///         deletions: 2,
    ///         changes: 7,
    ///         status: "modified".to_string(),
    ///     },
    /// ];
    ///
    /// let size_info = PrSizeInfo::new(
    ///     files,
    ///     vec![],
    ///     &merge_warden_core::size::SizeThresholds::default()
    /// );
    ///
    /// assert_eq!(size_info.included_file_count(), 2);
    /// ```
    pub fn included_file_count(&self) -> usize {
        self.included_files.len()
    }

    /// Get the number of files excluded from the size calculation.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::size::PrSizeInfo;
    /// use merge_warden_developer_platforms::models::PullRequestFile;
    ///
    /// let excluded = vec![
    ///     PullRequestFile {
    ///         filename: "package-lock.json".to_string(),
    ///         additions: 1000,
    ///         deletions: 500,
    ///         changes: 1500,
    ///         status: "modified".to_string(),
    ///     },
    /// ];
    ///
    /// let size_info = PrSizeInfo::new(
    ///     vec![],
    ///     excluded,
    ///     &merge_warden_core::size::SizeThresholds::default()
    /// );
    ///
    /// assert_eq!(size_info.excluded_file_count(), 1);
    /// ```
    pub fn excluded_file_count(&self) -> usize {
        self.excluded_files.len()
    }
}
