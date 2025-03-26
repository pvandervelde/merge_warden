//! # Models
//!
//! This module contains the data models used throughout the Merge Warden core.
//!
//! These models represent the core entities that Merge Warden works with, such as
//! pull requests, comments, and labels. They are designed to be serializable and
//! deserializable to facilitate integration with Git provider APIs.

use serde::{Deserialize, Serialize};

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
/// use merge_warden_developer_platforms::models::Comment;
///
/// let comment = Comment {
///     id: 456,
///     body: "Please update your PR title to follow the Conventional Commits format.".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// The unique identifier of the comment
    pub id: u64,

    /// The text content of the comment
    pub body: String,
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
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// The name of the label
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
/// * `body` - The description/body of the pull request, if any
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::PullRequest;
///
/// let pr = PullRequest {
///     number: 123,
///     title: "feat(auth): add GitHub login".to_string(),
///     body: Some("This PR adds GitHub login functionality.\n\nFixes #42".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// The pull request number
    pub number: u64,

    /// The title of the pull request
    pub title: String,

    /// The description/body of the pull request, if any
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Review {
    id: u64,
    state: String,
    user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    login: String,
}
