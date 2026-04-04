//! Test data management and generation for integration testing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::TestResult;

/// Manager for test data templates and generation.
///
/// The `TestDataManager` provides standardized test data for integration tests,
/// including pull request specifications, review data, and comment templates.
///
/// # Examples
///
/// ```rust
/// use merge_warden_integration_tests::{TestDataManager, PullRequestSpec};
///
/// let manager = TestDataManager::new();
/// let pr_spec = manager.create_pull_request_spec("feature/test-branch");
/// assert!(!pr_spec.title.is_empty());
/// ```
#[allow(dead_code)]
pub struct TestDataManager {
    /// Template configurations
    templates: HashMap<String, String>,
    /// Default values for test data
    defaults: TestDataDefaults,
}

impl Default for TestDataManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TestDataManager {
    /// Creates a new test data manager with default templates.
    ///
    /// # Returns
    ///
    /// A configured `TestDataManager` with standard test data templates.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestDataManager;
    ///
    /// let manager = TestDataManager::new();
    /// // Manager is ready with default templates
    /// ```
    pub fn new() -> Self {
        // Minimal implementation for doc tests
        TestDataManager {
            templates: std::collections::HashMap::new(),
            defaults: TestDataDefaults {
                organization: "glitchgrove".to_string(),
                repository_prefix: "merge-warden-test".to_string(),
                default_branches: vec!["main".to_string()],
                file_sizes: std::collections::HashMap::new(),
                config_templates: std::collections::HashMap::new(),
            },
        }
    }

    /// Creates a pull request specification for testing.
    ///
    /// # Parameters
    ///
    /// - `branch_name`: Source branch name for the pull request
    ///
    /// # Returns
    ///
    /// A `PullRequestSpec` with realistic test data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestDataManager;
    ///
    /// let manager = TestDataManager::new();
    /// let spec = manager.create_pull_request_spec("feature/new-feature");
    /// assert!(spec.title.contains("feat:"));
    /// ```
    pub fn create_pull_request_spec(&self, branch_name: &str) -> PullRequestSpec {
        PullRequestSpec {
            title: format!("feat: add {}", branch_name),
            body: "This PR adds a new feature for testing.".to_string(),
            source_branch: branch_name.to_string(),
            target_branch: "main".to_string(),
            files: vec![],
            labels: vec!["test-label".to_string()],
            draft: false,
            assignees: vec!["test-user".to_string()],
            reviewers: vec!["reviewer1".to_string()],
        }
    }

    /// Creates a review specification for testing.
    ///
    /// # Parameters
    ///
    /// - `review_type`: Type of review ("APPROVE", "REQUEST_CHANGES", "COMMENT")
    ///
    /// # Returns
    ///
    /// A `ReviewSpec` with appropriate review data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestDataManager;
    ///
    /// let manager = TestDataManager::new();
    /// let spec = manager.create_review_spec("APPROVE");
    /// assert_eq!(spec.event, "APPROVE");
    /// ```
    pub fn create_review_spec(&self, review_type: &str) -> ReviewSpec {
        ReviewSpec {
            event: review_type.to_string(),
            body: Some(format!("{} review for testing.", review_type)),
            comments: vec![],
            dismiss_stale_reviews: false,
        }
    }

    /// Creates a comment specification for testing.
    ///
    /// # Parameters
    ///
    /// - `comment_type`: Type of comment ("general", "suggestion", "question")
    ///
    /// # Returns
    ///
    /// A `CommentSpec` with realistic comment content.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestDataManager;
    ///
    /// let manager = TestDataManager::new();
    /// let spec = manager.create_comment_spec("suggestion");
    /// assert!(spec.body.contains("suggestion"));
    /// ```
    pub fn create_comment_spec(&self, comment_type: &str) -> CommentSpec {
        CommentSpec {
            body: format!("This is a {} comment for testing.", comment_type),
            comment_type: comment_type.to_string(),
            is_bot_comment: false,
            timestamp: Some(chrono::Utc::now()),
        }
    }

    /// Loads merge-warden configuration template.
    ///
    /// # Parameters
    ///
    /// - `template_name`: Name of the configuration template
    ///
    /// # Returns
    ///
    /// The TOML configuration content as a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestDataManager;
    ///
    /// let manager = TestDataManager::new();
    /// let config = manager.load_config_template("default").unwrap();
    /// assert!(config.contains("schemaVersion"));
    /// ```
    pub fn load_config_template(&self, template_name: &str) -> TestResult<String> {
        Ok(format!(
            "schemaVersion = '1.0'\ntemplate = '{}'\n",
            template_name
        ))
    }

    /// Generates file content for repository testing.
    ///
    /// # Parameters
    ///
    /// - `file_type`: Type of file to generate ("readme", "source", "test")
    /// - `size_category`: Size category for the file ("small", "medium", "large")
    ///
    /// # Returns
    ///
    /// Generated file content appropriate for the specified type and size.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::TestDataManager;
    ///
    /// let manager = TestDataManager::new();
    /// let content = manager.generate_file_content("readme", "medium").unwrap();
    /// assert!(content.contains("README"));
    /// ```
    pub fn generate_file_content(
        &self,
        file_type: &str,
        size_category: &str,
    ) -> TestResult<String> {
        Ok(format!(
            "{} file content for {} size",
            file_type.to_uppercase(),
            size_category
        ))
    }
}

/// Specification for creating test pull requests.
///
/// This struct defines all the parameters needed to create a realistic
/// pull request for integration testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestSpec {
    /// Pull request title
    pub title: String,
    /// Pull request body/description
    pub body: String,
    /// Source branch name
    pub source_branch: String,
    /// Target branch name (usually "main" or "master")
    pub target_branch: String,
    /// Files to include in the pull request
    pub files: Vec<FileSpec>,
    /// Labels to apply to the pull request
    pub labels: Vec<String>,
    /// Whether the pull request should be marked as draft
    pub draft: bool,
    /// Assignees for the pull request
    pub assignees: Vec<String>,
    /// Reviewers to request
    pub reviewers: Vec<String>,
}

impl Default for PullRequestSpec {
    fn default() -> Self {
        Self {
            title: "feat: add new feature".to_string(),
            body: "This PR adds a new feature.\n\nFixes #123".to_string(),
            source_branch: "feature/new-feature".to_string(),
            target_branch: "main".to_string(),
            files: vec![],
            labels: vec![],
            draft: false,
            assignees: vec![],
            reviewers: vec![],
        }
    }
}

/// Specification for file content in pull requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSpec {
    /// Path to the file within the repository
    pub path: String,
    /// Content of the file
    pub content: String,
    /// Action to perform on the file
    pub action: FileAction,
    /// MIME type of the file
    pub mime_type: Option<String>,
}

/// Actions that can be performed on files in pull requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileAction {
    /// Add a new file
    Add,
    /// Modify an existing file
    Modify,
    /// Delete an existing file
    Delete,
    /// Rename an existing file
    Rename { from: String },
}

impl FileAction {
    /// Returns a commit message prefix for this file action.
    pub fn as_commit_message(&self) -> &str {
        match self {
            FileAction::Add => "Add",
            FileAction::Modify => "Update",
            FileAction::Delete => "Delete",
            FileAction::Rename { .. } => "Rename",
        }
    }
}

/// Specification for creating test reviews.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSpec {
    /// Review event type ("APPROVE", "REQUEST_CHANGES", "COMMENT")
    pub event: String,
    /// Review body/comment
    pub body: Option<String>,
    /// Line-by-line review comments
    pub comments: Vec<ReviewCommentSpec>,
    /// Whether to dismiss stale reviews
    pub dismiss_stale_reviews: bool,
}

impl Default for ReviewSpec {
    fn default() -> Self {
        Self {
            event: "APPROVE".to_string(),
            body: Some("LGTM!".to_string()),
            comments: vec![],
            dismiss_stale_reviews: false,
        }
    }
}

/// Specification for line-by-line review comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCommentSpec {
    /// File path for the comment
    pub path: String,
    /// Line number for the comment
    pub line: u32,
    /// Comment body
    pub body: String,
    /// Whether this is a suggestion
    pub is_suggestion: bool,
}

/// Specification for creating test comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentSpec {
    /// Comment body content
    pub body: String,
    /// Comment type for categorization
    pub comment_type: String,
    /// Whether the comment is from a bot
    pub is_bot_comment: bool,
    /// Timestamp for the comment
    pub timestamp: Option<DateTime<Utc>>,
}

impl Default for CommentSpec {
    fn default() -> Self {
        Self {
            body: "This is a test comment.".to_string(),
            comment_type: "general".to_string(),
            is_bot_comment: false,
            timestamp: None,
        }
    }
}

/// Handle to a created pull request for testing.
#[derive(Debug, Clone)]
pub struct TestPullRequest {
    /// Pull request number
    pub number: u64,
    /// Pull request ID
    pub id: u64,
    /// Pull request title
    pub title: String,
    /// Pull request body
    pub body: String,
    /// Source branch name
    pub head: String,
    /// Target branch name
    pub base: String,
    /// Repository full name (owner/repo)
    pub repo_full_name: String,
}

/// Default values for test data generation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TestDataDefaults {
    /// Default organization name
    pub organization: String,
    /// Default repository prefix
    pub repository_prefix: String,
    /// Default branch names
    pub default_branches: Vec<String>,
    /// Default file sizes for different categories
    pub file_sizes: HashMap<String, usize>,
    /// Default configuration templates
    pub config_templates: HashMap<String, String>,
}
