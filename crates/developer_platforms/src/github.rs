use async_trait::async_trait;
use jsonwebtoken::EncodingKey;
use octocrab::{
    models::{
        pulls::{Review, ReviewState},
        InstallationId, InstallationToken, ReviewId,
    },
    params::apps::CreateInstallationAccessToken,
    Octocrab, Page,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

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

/// Authenticates with GitHub using an installation access token for a specific app installation.
///
/// This function retrieves an access token for a GitHub App installation and creates a new
/// `Octocrab` client authenticated with that token. It is useful for performing API operations
/// on behalf of a GitHub App installation.
///
/// # Arguments
///
/// * `octocrab` - An existing `Octocrab` client instance.
/// * `installation_id` - The ID of the GitHub App installation.
/// * `repository_owner` - The owner of the repository associated with the installation.
/// * `source_repository` - The name of the repository associated with the installation.
///
/// # Returns
///
/// A `Result` containing a new `Octocrab` client authenticated with the installation access token,
/// or an `Error` if the operation fails.
///
/// # Errors
///
/// This function returns an `Error` in the following cases:
/// - If the app installation cannot be found.
/// - If the access token cannot be created.
/// - If the new `Octocrab` client cannot be built.
///
/// # Example
///
/// ```rust,no_run
/// use anyhow::Result;
/// use octocrab::Octocrab;
/// use merge_warden_developer_platforms::github::authenticate_with_access_token;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let octocrab = Octocrab::builder().build().unwrap();
///     let installation_id = 12345678; // Replace with your installation ID
///     let repository_owner = "example-owner";
///     let source_repository = "example-repo";
///
///     let authenticated_client = authenticate_with_access_token(
///         &octocrab,
///         installation_id,
///         repository_owner,
///         source_repository,
///     )
///     .await?;
///
///     // Use `authenticated_client` to perform API operations
///     Ok(())
/// }
/// ```
pub async fn authenticate_with_access_token(
    octocrab: &Octocrab,
    installation_id: u64,
    repository_owner: &str,
    source_repository: &str,
) -> Result<Octocrab, Error> {
    // Get an access token for the specific app installation that sent the event
    // First find all the installations and use those to grab the specific one that
    // sent the event
    let installations = octocrab
        .apps()
        .installations()
        .send()
        .await
        .unwrap_or(Page::<octocrab::models::Installation>::default())
        .take_items();

    let id = InstallationId(installation_id);
    let Some(installation_index) = installations.iter().position(|l| l.id == id) else {
        return Err(Error::FailedToFindAppInstallation(
            repository_owner.to_string(),
            source_repository.to_string(),
            installation_id,
        ));
    };

    let installation = &installations[installation_index];
    debug!(
        "Creating access token for installation with id {}. Linked to repository at {}",
        installation.id,
        installation
            .repositories_url
            .clone()
            .unwrap_or("".to_string())
    );

    let create_access_token = CreateInstallationAccessToken::default();
    //create_access_token.repositories = vec![repository_name.clone()];

    // Create an access token for the installation
    let access: InstallationToken = octocrab
        .post(
            installations[installation_index]
                .access_tokens_url
                .as_ref()
                .unwrap(),
            Some(&create_access_token),
        )
        .await
        .map_err(|_| {
            Error::FailedToCreateAccessToken(
                repository_owner.to_string(),
                source_repository.to_string(),
                installation_id,
            )
        })?;

    // USe the API token
    let api_with_token = octocrab::OctocrabBuilder::new()
        .personal_token(access.token)
        .build()
        .unwrap();

    Ok(api_with_token)
}

/// Creates an `Octocrab` client authenticated as a GitHub App using a JWT token.
///
/// This function generates a JSON Web Token (JWT) for the specified GitHub App ID and private key,
/// and uses it to create an authenticated `Octocrab` client. The client can then be used to perform
/// API operations on behalf of the GitHub App.
///
/// # Arguments
///
/// * `app_id` - The ID of the GitHub App.
/// * `private_key` - The private key associated with the GitHub App, in PEM format.
///
/// # Returns
///
/// A `Result` containing an authenticated `Octocrab` client, or an `Error` if the operation fails.
///
/// # Errors
///
/// This function returns an `Error` in the following cases:
/// - If the private key cannot be parsed.
/// - If the JWT token cannot be created.
/// - If the `Octocrab` client cannot be built.
///
/// # Example
///
/// ```rust,no_run
/// use anyhow::Result;
/// use merge_warden_developer_platforms::github::create_app_client;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let app_id = 123456; // Replace with your GitHub App ID
///     let private_key = r#"
/// -----BEGIN RSA PRIVATE KEY-----
/// ...
/// -----END RSA PRIVATE KEY-----
/// "#; // Replace with your GitHub App private key
///
///     let client = create_app_client(app_id, private_key).await?;
///
///     // Use `client` to perform API operations
///     Ok(())
/// }
/// ```
pub async fn create_app_client(app_id: u64, private_key: &str) -> Result<Octocrab, Error> {
    //let app_id_struct = AppId::from(app_id);
    let key = EncodingKey::from_rsa_pem(private_key.as_bytes())
        .map_err(|_| Error::AuthError("Failed to translate the private key.".to_string()))?;

    //let octocrab = Octocrab::builder().app(app_id_struct, key).build()?;

    let token = octocrab::auth::create_jwt(app_id.into(), &key).unwrap();
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .map_err(|_| {
            Error::AuthError("Failed to get a personal token for the app install.".to_string())
        })?;

    Ok(octocrab)
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
