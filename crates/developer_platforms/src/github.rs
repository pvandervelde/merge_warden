use async_trait::async_trait;
use jsonwebtoken::EncodingKey;
use octocrab::{
    models::{
        pulls::{Review, ReviewState},
        ReviewId,
    },
    Octocrab,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    errors::Error,
    models::{Comment, Label, PullRequest, User},
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
#[instrument]
pub async fn authenticate_with_access_token(
    octocrab: &Octocrab,
    installation_id: u64,
    repository_owner: &str,
    source_repository: &str,
) -> Result<Octocrab, Error> {
    debug!(
        repository_owner = repository_owner,
        repository = source_repository,
        installation_id,
        "Finding installation"
    );

    let (api_with_token, _) = octocrab
        .installation_and_token(installation_id.into())
        .await
        .map_err(|_| {
            error!(
                repository_owner = repository_owner,
                repository = source_repository,
                installation_id,
                "Failed to create a token for the installation",
            );

            Error::InvalidResponse
        })?;

    info!(
        repository_owner = repository_owner,
        repository = source_repository,
        installation_id,
        "Created access token for installation",
    );

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
#[instrument(skip(private_key))]
pub async fn create_app_client(app_id: u64, private_key: &str) -> Result<(Octocrab, User), Error> {
    //let app_id_struct = AppId::from(app_id);
    let key = EncodingKey::from_rsa_pem(private_key.as_bytes()).map_err(|e| {
        Error::AuthError(
            format!("Failed to translate the private key. Error was: {}", e).to_string(),
        )
    })?;

    let octocrab = Octocrab::builder()
        .app(app_id.into(), key)
        .build()
        .map_err(|_| {
            Error::AuthError("Failed to get a personal token for the app install.".to_string())
        })?;

    info!("Created access token for the GitHub app",);

    let author = match octocrab.current().app().await {
        Ok(a) => a,
        Err(e) => {
            log_octocrab_error(
                "Failed to retreive App information for the currently authenticated app",
                e,
            );
            return Err(Error::InvalidResponse);
        }
    };

    let user = User {
        id: author.id.into_inner(),
        login: author.name,
    };

    Ok((octocrab, user))
}

#[instrument(skip(token))]
pub fn create_token_client(token: &str) -> Result<Octocrab, Error> {
    Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|_| Error::ApiError())
}

fn log_octocrab_error(message: &str, e: octocrab::Error) {
    match e {
        octocrab::Error::GitHub { source, backtrace } => {
            let err = *source;
            error!(
                error_message = err.message,
                backtrace = backtrace.to_string(),
                "{}. Received an error from GitHub",
                message
            )
        }
        octocrab::Error::UriParse { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}. Failed to parse URI.",
            message
        ),

        octocrab::Error::Uri { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}, Failed to parse URI.",
            message
        ),
        octocrab::Error::InvalidHeaderValue { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}. One of the header values was invalid.",
            message
        ),
        octocrab::Error::InvalidUtf8 { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}. The message wasn't valid UTF-8.",
            message,
        ),
        _ => error!(message,),
    };
}

#[derive(Debug)]
pub struct GitHubProvider {
    client: Octocrab,
    user: User,
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
    #[instrument]
    async fn create_review(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        body: &str,
        event: &str,
    ) -> Result<(), Error> {
        // The app should never approve a PR
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

        debug!(
            repository_owner = repo_owner,
            repository = repo_name,
            pull_request = pr_number,
            "Creating review for pr",
        );

        self.client
            ._post(route, Some(&request))
            .await
            .map_err(|e| {
                log_octocrab_error("Failed to create new review", e);
                Error::FailedToUpdatePullRequest("Failed to create review".to_string())
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
    #[instrument]
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

        info!(
            repository_owner = repo_owner,
            repository = repo_name,
            pull_request = pr_number,
            review_id = review_id.into_inner(),
            "Dismissing review ...",
        );

        self.client._put(route, Some(&request)).await.map_err(|e| {
            log_octocrab_error("Failed to dismiss existing review", e);
            Error::FailedToUpdatePullRequest("Failed to dismiss review".to_string())
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
    #[instrument]
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
            .send()
            .await
        {
            Ok(p) => p,
            Err(e) => {
                log_octocrab_error("Failed to list reviews", e);
                return Err(Error::InvalidResponse);
            }
        };

        let mut reviews = current_page.take_items();
        while let Ok(Some(mut new_page)) = self.client.get_page(&current_page.next).await {
            reviews.extend(new_page.take_items());

            current_page = new_page;
        }

        Ok(reviews)
    }

    pub fn new(client: Octocrab, user: User) -> Self {
        Self { client, user }
    }
}

#[async_trait]
impl PullRequestProvider for GitHubProvider {
    #[instrument]
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
                draft: pr.draft.unwrap_or_default(),
                body: pr.body,
            }),
            Err(e) => {
                log_octocrab_error("Failed to get pull request information", e);
                return Err(Error::InvalidResponse);
            }
        }
    }

    #[instrument]
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
            Err(e) => {
                log_octocrab_error("Failed to add pull request comment", e);
                return Err(Error::FailedToUpdatePullRequest(
                    "Failed to add comment".to_string(),
                ));
            }
        }
    }

    #[instrument]
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
                warn!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    comment = comment_id,
                    "Failed to delete pr comment",
                );
                Error::FailedToUpdatePullRequest(format!("Failed to delete comment: {}", e))
            })
    }

    #[instrument]
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
            .send()
            .await
        {
            Ok(p) => p,
            Err(e) => {
                log_octocrab_error("Failed to list comments for pull request", e);
                return Err(Error::InvalidResponse);
            }
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
                user: User {
                    id: c.user.id.into_inner(),
                    login: c.user.login,
                },
            })
            .collect();

        Ok(result)
    }

    #[instrument]
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
            Err(e) => {
                log_octocrab_error("Failed to add new labels", e);
                return Err(Error::FailedToUpdatePullRequest(
                    "Failed to add labels".to_string(),
                ));
            }
        }
    }

    #[instrument]
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
            Err(e) => {
                log_octocrab_error("Failed to remove label", e);
                return Err(Error::FailedToUpdatePullRequest(
                    "Failed to remove label".to_string(),
                ));
            }
        }
    }

    #[instrument]
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
            Err(e) => {
                log_octocrab_error("Failed to list all labels for pull request", e);
                return Err(Error::InvalidResponse);
            }
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
    #[instrument]
    async fn update_pr_blocking_review(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        message: &str,
        message_prefix: &str,
        is_approved: bool,
    ) -> Result<(), Error> {
        let user = self.user.clone();
        let expected_user_name_on_reviews = format!("{}[bot]", user.login);

        info!(
            repository_owner = repo_owner,
            repository = repo_name,
            pull_request = pr_number,
            user_id = user.id,
            user_name = user.login,
            "Updating pull request review status ...",
        );

        // First, list existing reviews to check if we already have one
        let existing_reviews = self.list_reviews(repo_owner, repo_name, pr_number).await?;
        let mut own_reviews: Vec<&Review> = existing_reviews
            .iter()
            .filter(|r| {
                let review_user = match &r.user {
                    Some(u) => u,
                    None => return false,
                };

                expected_user_name_on_reviews == review_user.login
            })
            .collect();
        debug!(
            repository_owner = repo_owner,
            repository = repo_name,
            pull_request = pr_number,
            count = existing_reviews.len(),
            "Searched for existing reviews",
        );

        own_reviews.sort_by(|a, b| {
            let a_time = a.submitted_at.unwrap_or_default();
            let b_time = b.submitted_at.unwrap_or_default();
            b_time.cmp(&a_time)
        });

        // Dismiss every review except the most recent one
        if !own_reviews.is_empty() {
            let size = own_reviews.len() - 1;
            for review in own_reviews.iter().take(size).skip(1) {
                if review.state != Some(ReviewState::Dismissed) {
                    match self
                        .dismiss_review(
                            repo_owner,
                            repo_name,
                            pr_number,
                            review.id,
                            "Issues resolved",
                        )
                        .await
                    {
                        Ok(()) => {
                            debug!(
                                repository_owner = repo_owner,
                                repository = repo_name,
                                pull_request = pr_number,
                                review_id = review.id.into_inner(),
                                "Review removed"
                            )
                        }
                        Err(_) => {
                            warn!(
                                repository_owner = repo_owner,
                                repository = repo_name,
                                pull_request = pr_number,
                                review_id = review.id.into_inner(),
                                "Failed to remove review"
                            )
                        }
                    };
                }
            }
        }

        // Get the most recent review
        let most_recent_review = own_reviews.first();

        if is_approved {
            info!(
                repository_owner = repo_owner,
                repository = repo_name,
                pull_request = pr_number,
                "Pull request approved, removing reviews ...",
            );

            if let Some(review) = most_recent_review {
                match self
                    .dismiss_review(
                        repo_owner,
                        repo_name,
                        pr_number,
                        review.id,
                        "Issues resolved",
                    )
                    .await
                {
                    Ok(_) => {
                        debug!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr_number,
                            review_id = review.id.into_inner(),
                            "Review removed"
                        )
                    }
                    Err(_) => {
                        warn!(
                            repository_owner = repo_owner,
                            repository = repo_name,
                            pull_request = pr_number,
                            review_id = review.id.into_inner(),
                            "Failed to remove review"
                        )
                    }
                };
            }
        } else {
            info!(
                repository_owner = repo_owner,
                repository = repo_name,
                pull_request = pr_number,
                "Pull request not approved, adding or updating review ...",
            );

            // If the last review is the same as the current one then we don't make changes
            let mut found_match = false;
            if let Some(review) = most_recent_review {
                debug!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pull_request = pr_number,
                    review_id = review.id.into_inner(),
                    review_date = review.submitted_at.unwrap_or_default().to_string(),
                    "Processing review",
                );

                let review_message_option = review.body.clone();
                let review_message = review_message_option.unwrap_or_default();
                if review_message.starts_with(message_prefix) {
                    found_match = true;
                }
            }

            if found_match {
                info!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pull_request = pr_number,
                    "Current review is the same as the new one. Will not be creating a new one.",
                );
            } else {
                debug!(
                    repository_owner = repo_owner,
                    repository = repo_name,
                    pull_request = pr_number,
                    "Didn't find existing matching review, creating new one",
                );

                if let Some(review) = most_recent_review {
                    match self
                        .dismiss_review(
                            repo_owner,
                            repo_name,
                            pr_number,
                            review.id,
                            "Review no longer valid. Will be creating a new one.",
                        )
                        .await
                    {
                        Ok(_) => {
                            debug!(
                                repository_owner = repo_owner,
                                repository = repo_name,
                                pull_request = pr_number,
                                review_id = review.id.into_inner(),
                                "Review removed"
                            )
                        }
                        Err(_) => {
                            warn!(
                                repository_owner = repo_owner,
                                repository = repo_name,
                                pull_request = pr_number,
                                review_id = review.id.into_inner(),
                                "Failed to remove review"
                            )
                        }
                    };
                }

                // Create new review requesting changes
                self.create_review(repo_owner, repo_name, pr_number, message, "REQUEST_CHANGES")
                    .await?
            }
        }

        Ok(())
    }
}
