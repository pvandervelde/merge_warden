use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use octocrab::{
    models::{
        pulls::{Review, ReviewState},
        ReviewId,
    },
    Octocrab,
};
use serde::{Deserialize, Serialize};

use crate::{
    errors::Error,
    models::{Comment, Label, PullRequest},
    PullRequestProvider,
};

#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    iat: u64,
    exp: u64,
    iss: u64,
}

/// Creates an authenticated GitHub client using a GitHub App's credentials.
///
/// This function generates a JSON Web Token (JWT) for authenticating as a GitHub App
/// and optionally retrieves an installation token if an installation ID is provided.
///
/// See: https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation
///
/// # Arguments
///
/// * `app_id` - The ID of the GitHub App.
/// * `private_key` - The private key associated with the GitHub App, in PEM format.
/// * `installation_id` - An optional installation ID for the GitHub App. If provided,
///   an installation token will be retrieved for the specified installation.
///
/// # Returns
///
/// Returns a `Result` containing an authenticated `Octocrab` client if successful,
/// or an `Error` if authentication or client creation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The private key is invalid.
/// - The JWT cannot be created.
/// - The Octocrab client cannot be built.
/// - The installation token cannot be retrieved (if `installation_id` is provided).
///
/// # Example
///
/// ```rust
/// use your_crate::create_app_client;
///
/// #[tokio::main]
/// async fn main() {
///     let app_id = 12345;
///     let private_key = "your-private-key";
///     let installation_id = Some(67890);
///
///     match create_app_client(app_id, private_key, installation_id).await {
///         Ok(client) => {
///             // Use the authenticated client
///         }
///         Err(e) => {
///             eprintln!("Failed to create app client: {}", e);
///         }
///     }
/// }
/// ```
pub async fn create_app_client(
    app_id: u64,
    private_key: &str,
    installation_id: Option<u64>,
) -> Result<Octocrab, Error> {
    // Create JWT for GitHub App authentication
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = JWTClaims {
        iat: now - 60,
        exp: now + (10 * 60), // 10 minutes expiration
        iss: app_id,
    };

    let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes())
        .map_err(|e| Error::AuthError(format!("Invalid private key: {}", e)))?;

    let jwt = jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
        .map_err(|e| Error::AuthError(format!("Failed to create JWT: {}", e)))?;

    // Create an authenticated octocrab instance
    let app_client = Octocrab::builder()
        .personal_token(jwt)
        .build()
        .map_err(|e| Error::AuthError(format!("Failed to build octocrab instance: {}", e)))?;

    // If installation ID is provided, get an installation token
    if let Some(installation_id) = installation_id {
        let installation_result = app_client
            .installation_and_token(installation_id.into())
            .await;
        let (client, _secret) = match installation_result {
            Ok(p) => p,
            Err(e) => return Err(Error::AuthError(e.to_string())),
        };

        Ok(client)
    } else {
        Ok(app_client)
    }
}

pub fn create_token_client(token: &str) -> Result<Octocrab, Error> {
    Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|_| Error::ApiError())
}

pub struct GitHubProvider {
    client: Octocrab,
}

impl GitHubProvider {
    /// Creates a new review for a pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `body` - The review comment
    /// * `event` - The review event type (APPROVE, REQUEST_CHANGES, COMMENT)
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn create_review(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        body: &str,
        event: &str,
    ) -> Result<(), Error> {
        // Prevent accidental approvals
        if event == "APPROVE" {
            return Err(Error::ApprovalProhibited);
        }

        #[derive(Debug, Serialize)]
        struct CreateReviewRequest<'a> {
            body: &'a str,
            event: &'a str,
        }

        let route = format!(
            "/repos/{}/{}/pulls/{}/reviews",
            repo_owner, repo_name, pr_number
        );
        let request = CreateReviewRequest { body, event };

        self.client
            ._post(route, Some(&request))
            .await
            .map_err(|e| {
                Error::FailedToUpdatePullRequest(format!("Failed to create review: {}", e))
            })?;

        Ok(())
    }

    /// Dismisses a review for a pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `review_id` - The ID of the review to dismiss
    /// * `message` - The dismissal message
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn dismiss_review(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        review_id: ReviewId,
        message: &str,
    ) -> Result<(), Error> {
        #[derive(Debug, Serialize)]
        struct DismissReviewRequest<'a> {
            message: &'a str,
        }

        let route = format!(
            "/repos/{}/{}/pulls/{}/reviews/{}/dismissals",
            repo_owner, repo_name, pr_number, review_id
        );
        let request = DismissReviewRequest { message };

        self.client._put(route, Some(&request)).await.map_err(|e| {
            Error::FailedToUpdatePullRequest(format!("Failed to dismiss review: {}", e))
        })?;

        Ok(())
    }

    pub async fn from_app(
        app_id: u64,
        private_key: &str,
        installation_id: Option<u64>,
    ) -> Result<Self, Error> {
        let client = create_app_client(app_id, private_key, installation_id).await?;
        Ok(Self::new(client))
    }

    pub fn from_token(token: &str) -> Result<Self, Error> {
        let client = create_token_client(token)?;
        Ok(Self::new(client))
    }

    /// Lists all reviews for a pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of reviews
    async fn list_reviews(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Review>, Error> {
        let mut current_page = match self
            .client
            .pulls(repo_owner, repo_name)
            .list_reviews(pr_number)
            .per_page(100)
            .page(2u32)
            .send()
            .await
        {
            Ok(p) => p,
            Err(_) => return Err(Error::InvalidResponse),
        };

        let mut reviews = current_page.take_items();
        while let Ok(Some(mut new_page)) = self.client.get_page(&current_page.next).await {
            reviews.extend(new_page.take_items());

            current_page = new_page;
        }

        Ok(reviews)
    }

    pub fn new(client: Octocrab) -> Self {
        Self { client }
    }

    /// Updates an existing review for a pull request.
    ///
    /// Note: GitHub doesn't have a direct API for updating reviews, so we dismiss the old one
    /// and create a new one.
    ///
    /// # Arguments
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `pr_number` - The pull request number
    /// * `review_id` - The ID of the review to update
    /// * `body` - The updated review comment
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    async fn update_review(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        review_id: ReviewId,
        body: &str,
    ) -> Result<(), Error> {
        // Dismiss the old review
        self.dismiss_review(
            repo_owner,
            repo_name,
            pr_number,
            review_id,
            "Updating review",
        )
        .await?;

        // Create a new review
        self.create_review(repo_owner, repo_name, pr_number, body, "REQUEST_CHANGES")
            .await?;

        Ok(())
    }
}

#[async_trait]
impl PullRequestProvider for GitHubProvider {
    async fn get_pull_request(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<PullRequest, Error> {
        match self
            .client
            .pulls(repo_owner, repo_name)
            .get(pr_number)
            .await
        {
            Ok(pr) => Ok(PullRequest {
                number: pr.number,
                title: pr.title.unwrap_or(String::new()),
                body: pr.body,
            }),
            Err(_) => Err(Error::InvalidResponse),
        }
    }

    async fn add_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        comment: &str,
    ) -> Result<(), Error> {
        match self
            .client
            .issues(repo_owner, repo_name)
            .create_comment(pr_number, comment)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::FailedToUpdatePullRequest(format!(
                "Failed to add comment: {}",
                e
            ))),
        }
    }

    async fn delete_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        comment_id: u64,
    ) -> Result<(), Error> {
        self.client
            .issues(repo_owner, repo_name)
            .delete_comment(comment_id.into())
            .await
            .map_err(|e| {
                Error::FailedToUpdatePullRequest(format!("Failed to delete comment: {}", e))
            })
    }

    async fn list_comments(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Comment>, Error> {
        let mut current_page = match self
            .client
            .issues(repo_owner, repo_name)
            .list_comments(pr_number)
            .since(chrono::Utc::now())
            .per_page(100)
            .send()
            .await
        {
            Ok(p) => p,
            Err(_) => return Err(Error::InvalidResponse),
        };

        let mut comments = current_page.take_items();
        while let Ok(Some(mut new_page)) = self.client.get_page(&current_page.next).await {
            comments.extend(new_page.take_items());

            current_page = new_page;
        }

        let result = comments
            .into_iter()
            .map(|c| Comment {
                id: c.id.0,
                body: c.body.unwrap_or_default(),
            })
            .collect();

        Ok(result)
    }

    async fn add_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        labels: &[String],
    ) -> Result<(), Error> {
        match self
            .client
            .issues(repo_owner, repo_name)
            .add_labels(pr_number, labels)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::FailedToUpdatePullRequest(format!(
                "Failed to add labels: {}",
                e
            ))),
        }
    }

    async fn remove_label(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        label: &str,
    ) -> Result<(), Error> {
        match self
            .client
            .issues(repo_owner, repo_name)
            .remove_label(pr_number, label)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::FailedToUpdatePullRequest(format!(
                "Failed to remove label: {}",
                e
            ))),
        }
    }

    async fn list_labels(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<Label>, Error> {
        let mut current_page = match self
            .client
            .issues(repo_owner, repo_name)
            .list_labels_for_issue(pr_number)
            .send()
            .await
        {
            Ok(p) => p,
            Err(_) => return Err(Error::InvalidResponse),
        };

        let mut labels = current_page.take_items();
        while let Ok(Some(mut new_page)) = self.client.get_page(&current_page.next).await {
            labels.extend(new_page.take_items());

            current_page = new_page;
        }

        let result = labels.into_iter().map(|l| Label { name: l.name }).collect();

        Ok(result)
    }

    /// Updates the blocking review status of a pull request adding a review that blocks the PR from
    /// being merged.
    ///
    /// # Parameters
    /// - `repo_owner`: The owner of the repository.
    /// - `repo_name`: The name of the repository.
    /// - `pr_number`: The number of the pull request to update.
    /// - `message`: A message describing the reason for the update.
    /// - `is_approved`: A boolean indicating whether the pull request should be approved for merging.
    ///
    /// # Behavior
    /// - If `is_approved` is `false`, a review requesting changes is added to the PR.
    /// - If `is_approved` is `true`, any reviews requesting changes made by the current application are removed.
    ///
    /// # Returns
    /// - `Ok(())` if the operation succeeds.
    /// - `Err(Error)` if the operation fails, with an error message indicating the failure reason.
    ///
    /// # Notes
    /// - https://docs.github.com/en/rest/pulls/reviews?apiVersion=2022-11-28#create-a-review-for-a-pull-request
    /// - This function never approves a PR. It only blocks the PR or provides no review
    ///
    /// # Errors
    /// - Returns an error if the API call to update the pull request fails.
    async fn update_pr_blocking_review(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        message: &str,
        is_approved: bool,
    ) -> Result<(), Error> {
        // First, list existing reviews to check if we already have one
        let existing_reviews = self.list_reviews(repo_owner, repo_name, pr_number).await?;

        if is_approved {
            // If PR should be approved, dismiss any existing blocking reviews
            for review in existing_reviews {
                if review.state == Some(ReviewState::ChangesRequested) {
                    self.dismiss_review(
                        repo_owner,
                        repo_name,
                        pr_number,
                        review.id,
                        "Issues resolved",
                    )
                    .await?;
                }
            }
        } else {
            // If PR should be blocked, create or update a review requesting changes
            let mut existing_review_id = None;
            for review in existing_reviews {
                if review.state == Some(ReviewState::ChangesRequested) {
                    existing_review_id = Some(review.id);
                    break;
                }
            }

            if let Some(review_id) = existing_review_id {
                // Update existing review
                self.update_review(repo_owner, repo_name, pr_number, review_id, message)
                    .await?;
            } else {
                // Create new review requesting changes
                self.create_review(repo_owner, repo_name, pr_number, message, "REQUEST_CHANGES")
                    .await?;
            }
        }

        Ok(())
    }
}
