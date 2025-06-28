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

    // The user who made the comment
    pub user: User,
}

#[derive(Deserialize)]
pub struct Installation {
    pub id: u64,
    pub slug: Option<String>,
    pub client_id: Option<String>,
    pub node_id: String,
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
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// The name of the label
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// The name of the organization
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

#[derive(Deserialize)]
pub struct Repository {
    pub full_name: String,
    pub name: String,
    pub node_id: String,
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: u64,
    pub state: String,
    pub user: User,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct User {
    pub id: u64,
    pub login: String,
}
