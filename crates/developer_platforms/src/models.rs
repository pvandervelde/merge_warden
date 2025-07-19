//! # Models
//!
//! This module contains the data models used throughout the Merge Warden core.
//!
//! These models represent the core entities that Merge Warden works with, such as
//! pull requests, comments, and labels. They are designed to be serializable and
//! deserializable to facilitate integration with Git provider APIs.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;

/// Represents a comment on a pull request.
///
/// This struct contains the essential information about a comment
/// that is needed for tracking and management.
///
/// # Fields
///
/// * `id` - The unique identifier of the comment
/// * `body` - The text content of the comment
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{ Comment, User };
///
/// let comment = Comment {
///     id: 456,
///     body: "Please update your PR title to follow the Conventional Commits format.".to_string(),
///     user: User {
///         id: 10,
///         login: "a".to_string(),
///     }
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// The unique identifier of the comment
    pub id: u64,

    /// The text content of the comment
    pub body: String,

    /// The user who made the comment
    pub user: User,
}

/// Represents a GitHub App installation.
///
/// This struct contains information about a GitHub App installation
/// on a repository or organization. Used for authentication and
/// identifying the scope of the app's permissions.
///
/// # Fields
///
/// * `id` - The unique identifier of the installation
/// * `slug` - Optional slug identifier for the installation
/// * `client_id` - Optional client ID associated with the installation
/// * `node_id` - GitHub's global node identifier for GraphQL API
/// * `name` - Optional name of the installation
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::Installation;
/// use serde_json::from_str;
///
/// let json = r#"{
///     "id": 12345,
///     "slug": "my-app-installation",
///     "client_id": "Iv1.1234567890abcdef",
///     "node_id": "MDIzOkluc3RhbGxhdGlvbjEyMzQ1",
///     "name": "My App Installation"
/// }"#;
///
/// let installation: Installation = from_str(json).expect("Failed to parse installation");
/// assert_eq!(installation.id, 12345);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    /// The unique identifier of the installation
    pub id: u64,

    /// Optional slug identifier for the installation
    pub slug: Option<String>,

    /// Optional client ID associated with the installation
    pub client_id: Option<String>,

    /// GitHub's global node identifier for GraphQL API
    pub node_id: String,

    /// Optional name of the installation
    pub name: Option<String>,
}

/// Represents a label on a pull request.
///
/// This struct contains the essential information about a label
/// that is needed for categorization and filtering.
///
/// # Fields
///
/// * `name` - The name of the label
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::Label;
///
/// let label = Label {
///     name: "bug".to_string(),
///     description: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// The name of the label
    pub name: String,

    /// The description of the label (optional)
    pub description: Option<String>,
}

/// Represents an organization on a Git provider platform.
///
/// This struct contains essential information about an organization
/// that owns repositories and manages team access.
///
/// # Fields
///
/// * `name` - The name/login of the organization
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::Organization;
///
/// let org = Organization {
///     name: "my-company".to_string(),
/// };
/// assert_eq!(org.name, "my-company");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// The name/login of the organization
    pub name: String,
}

/// Represents a pull request from a Git provider.
///
/// This struct contains the essential information about a pull request
/// that is needed for validation and processing.
///
/// # Fields
///
/// * `number` - The pull request number
/// * `title` - The title of the pull request
/// * `draft` - Whether the pull request is a draft
/// * `body` - The description/body of the pull request, if any
/// * `author` - The user who created the pull request, if available
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{PullRequest, User};
///
/// let pr = PullRequest {
///     number: 123,
///     title: "feat(auth): add GitHub login".to_string(),
///     draft: false,
///     body: Some("This PR adds GitHub login functionality.\n\nFixes #42".to_string()),
///     author: Some(User {
///         id: 456,
///         login: "developer123".to_string(),
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// The pull request number
    pub number: u64,

    /// The title of the pull request
    pub title: String,

    /// Indicates if the pull request is a draft or not
    pub draft: bool,

    /// The description/body of the pull request, if any
    pub body: Option<String>,

    /// The user who created the pull request, if available
    pub author: Option<User>,
}

/// Represents a file that has been changed in a pull request.
///
/// This struct contains information about a file that was modified, added, or
/// deleted as part of a pull request. Used for calculating PR size and analyzing
/// the scope of changes.
///
/// # Fields
///
/// * `filename` - The path of the file relative to the repository root
/// * `additions` - Number of lines added to the file
/// * `deletions` - Number of lines deleted from the file
/// * `changes` - Total number of line changes (additions + deletions)
/// * `status` - The change status of the file (added, modified, deleted, renamed)
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::PullRequestFile;
///
/// let file = PullRequestFile {
///     filename: "src/main.rs".to_string(),
///     additions: 15,
///     deletions: 5,
///     changes: 20,
///     status: "modified".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestFile {
    /// The file path relative to the repository root
    pub filename: String,

    /// Number of lines added to the file
    pub additions: u32,

    /// Number of lines deleted from the file
    pub deletions: u32,

    /// Total changes (additions + deletions)
    pub changes: u32,

    /// File status (added, modified, deleted, renamed)
    pub status: String,
}

/// Represents a repository on a Git provider platform.
///
/// This struct contains essential information about a repository
/// that is used for identifying and accessing the repository.
///
/// # Fields
///
/// * `full_name` - The full name including owner (e.g., "owner/repo")
/// * `name` - The repository name only
/// * `node_id` - GitHub's global node identifier for GraphQL API
/// * `private` - Whether the repository is private or public
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::Repository;
/// use serde_json::from_str;
///
/// let json = r#"{
///     "full_name": "octocat/Hello-World",
///     "name": "Hello-World",
///     "node_id": "MDEwOlJlcG9zaXRvcnkxMjk2MjY5",
///     "private": false
/// }"#;
///
/// let repo: Repository = from_str(json).expect("Failed to parse repository");
/// assert_eq!(repo.full_name, "octocat/Hello-World");
/// assert_eq!(repo.name, "Hello-World");
/// assert!(!repo.private);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// The full name including owner (e.g., "owner/repo")
    pub full_name: String,

    /// The repository name only
    pub name: String,

    /// GitHub's global node identifier for GraphQL API
    pub node_id: String,

    /// Whether the repository is private or public
    pub private: bool,
}

/// Represents a review on a pull request.
///
/// This struct contains information about a code review submitted
/// by a user on a pull request, including the review state and reviewer.
///
/// # Fields
///
/// * `id` - The unique identifier of the review
/// * `state` - The state of the review (e.g., "approved", "changes_requested", "commented")
/// * `user` - The user who submitted the review
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{Review, User};
///
/// let review = Review {
///     id: 789,
///     state: "approved".to_string(),
///     user: User {
///         id: 123,
///         login: "reviewer123".to_string(),
///     },
/// };
/// assert_eq!(review.state, "approved");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// The unique identifier of the review
    pub id: u64,

    /// The state of the review (e.g., "approved", "changes_requested", "commented")
    pub state: String,

    /// The user who submitted the review
    pub user: User,
}

/// Represents a user on a Git provider platform.
///
/// This struct contains the essential user information needed
/// for identification and attribution of actions.
///
/// # Fields
///
/// * `id` - The unique identifier of the user
/// * `login` - The username/login of the user
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::User;
///
/// let user = User {
///     id: 456,
///     login: "octocat".to_string(),
/// };
/// assert_eq!(user.login, "octocat");
/// ```
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct User {
    /// The unique identifier of the user
    pub id: u64,

    /// The username/login of the user
    pub login: String,
}
