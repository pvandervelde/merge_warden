//! Repository management for integration testing.
//!
//! This module provides automated GitHub repository creation, configuration, and cleanup
//! specifically for integration testing of the Merge Warden bot.

use crate::environment::TestRepository;
use crate::errors::{TestError, TestResult};

/// Automated GitHub repository management for integration testing.
///
/// The `TestRepositoryManager` handles the complete lifecycle of test repositories
/// including creation, configuration, and cleanup within the designated GitHub
/// organization for testing.
///
/// # Features
///
/// - **Automated Creation**: Creates uniquely named repositories in test organization
/// - **Configuration Setup**: Applies merge-warden.toml and branch protection rules
/// - **Content Management**: Adds initial content and file structures for testing
/// - **Cleanup Management**: Tracks and cleans up all created repositories
///
/// # Examples
///
/// ```rust
/// use merge_warden_integration_tests::{TestRepositoryManager, TestError};
///
/// #[tokio::test]
/// async fn test_repository_lifecycle() -> Result<(), TestError> {
///     let mut manager = TestRepositoryManager::new("github_token".to_string()).await?;
///
///     let repo = manager.create_repository("test-case").await?;
///     manager.setup_configuration(&repo).await?;
///
///     // Repository automatically cleaned up when manager is dropped
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct TestRepositoryManager {
    /// GitHub API client for repository operations
    github_client: octocrab::Octocrab,
    /// Target organization for test repositories
    organization: String,
    /// Prefix for repository names
    repository_prefix: String,
    /// List of created repositories for cleanup
    created_repositories: Vec<String>,
}

impl TestRepositoryManager {
    /// Creates a new repository manager for the specified organization.
    ///
    /// # Parameters
    ///
    /// - `github_token`: GitHub personal access token with repo permissions
    /// - `organization`: GitHub organization for test repositories (default: "glitchgrove")
    /// - `prefix`: Prefix for repository names (default: "merge-warden-test")
    ///
    /// # Returns
    ///
    /// A configured `TestRepositoryManager` ready for repository operations.
    ///
    /// # Errors
    ///
    /// Returns `TestError::AuthenticationError` if the GitHub token is invalid
    /// or lacks sufficient permissions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestRepositoryManager, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_manager_creation() -> Result<(), TestError> {
    ///     let manager = TestRepositoryManager::new("github_token".to_string()).await?;
    ///     assert_eq!(manager.organization(), "glitchgrove");
    ///     Ok(())
    /// }
    /// ```
    /// Creates a new repository manager authenticated as the test helper GitHub App.
    ///
    /// Generates a JWT from the app credentials, exchanges it for an installation
    /// access token scoped to `organization`, then builds an authenticated octocrab
    /// client. No personal access token is required.
    ///
    /// # Parameters
    ///
    /// - `app_id`: Numeric GitHub App ID for the test helper app
    /// - `private_key`: PEM-encoded RSA private key for the test helper app
    /// - `organization`: GitHub organization where test repositories will be created
    /// - `prefix`: Prefix for repository names (e.g. "merge-warden-test")
    ///
    /// # Returns
    ///
    /// A configured `TestRepositoryManager` ready for repository operations.
    ///
    /// # Errors
    ///
    /// Returns `TestError::AuthenticationError` if:
    /// - The private key cannot be parsed as RSA PEM
    /// - JWT signing fails
    ///
    /// Returns `TestError::GitHubApiError` if:
    /// - The app installation for the org cannot be found
    /// - The installation access token request fails
    pub async fn new(
        app_id: String,
        private_key: String,
        organization: String,
        prefix: String,
    ) -> TestResult<Self> {
        use github_bot_sdk::auth::{AuthenticationProvider, InstallationId};
        use merge_warden_developer_platforms::app_auth::AppAuthProvider;

        // In mock-services mode, skip real JWT auth and return a stub instance.
        if std::env::var("USE_MOCK_SERVICES").unwrap_or_default() == "true" {
            let github_client = octocrab::Octocrab::builder().build().map_err(|e| {
                TestError::environment_error("build_mock_repo_client", &e.to_string())
            })?;
            return Ok(Self {
                github_client,
                organization,
                repository_prefix: prefix,
                created_repositories: Vec::new(),
            });
        }

        let app_id_num: u64 = app_id.parse().map_err(|_| {
            TestError::InvalidConfiguration(
                "REPO_CREATION_APP_ID must be a valid integer".to_string(),
            )
        })?;

        let auth_provider =
            AppAuthProvider::new(app_id_num, &private_key, "https://api.github.com").map_err(
                |e| TestError::authentication_error("repo_creation_auth_provider", &e.to_string()),
            )?;

        let jwt = auth_provider
            .app_token()
            .await
            .map_err(|e| TestError::authentication_error("test_app_jwt", &e.to_string()))?;

        // Look up the app's installation for the target org using the JWT via reqwest
        let http_client = reqwest::Client::new();
        let installation: serde_json::Value = http_client
            .get(format!(
                "https://api.github.com/orgs/{}/installation",
                organization
            ))
            .header("Authorization", format!("Bearer {}", jwt.token()))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "merge-warden-integration-tests")
            .send()
            .await
            .map_err(|e| TestError::github_api_error("get_org_installation", &e.to_string()))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| TestError::github_api_error("parse_org_installation", &e.to_string()))?;

        let installation_id = installation["id"].as_u64().ok_or_else(|| {
            TestError::environment_error(
                "parse_installation_id",
                "No installation id found in response",
            )
        })?;

        // Exchange for an installation access token via AppAuthProvider
        let token = auth_provider
            .installation_token(InstallationId::new(installation_id))
            .await
            .map_err(|e| {
                TestError::authentication_error("repo_creation_installation_token", &e.to_string())
            })?;

        let github_client = octocrab::Octocrab::builder()
            .personal_token(token.token().to_string())
            .build()
            .map_err(|e| {
                TestError::environment_error("build_authenticated_client", &e.to_string())
            })?;

        Ok(TestRepositoryManager {
            github_client,
            organization,
            repository_prefix: prefix,
            created_repositories: Vec::new(),
        })
    }

    /// Creates a new test repository with unique naming.
    ///
    /// This method creates a repository with a unique name in the configured organization,
    /// ensuring no naming conflicts between test runs.
    ///
    /// # Parameters
    ///
    /// - `name_suffix`: Descriptive suffix for the repository name
    ///
    /// # Returns
    ///
    /// A `TestRepository` representing the created repository with all metadata.
    ///
    /// # Errors
    ///
    /// Returns `TestError::GitHubApiError` if repository creation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestRepositoryManager, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_repository_creation() -> Result<(), TestError> {
    ///     let mut manager = TestRepositoryManager::new("token".to_string()).await?;
    ///     let repo = manager.create_repository("basic-test").await?;
    ///
    ///     assert!(repo.name.contains("basic-test"));
    ///     assert_eq!(repo.organization, "glitchgrove");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_repository(&mut self, name_suffix: &str) -> TestResult<TestRepository> {
        use uuid::Uuid;

        // Generate unique repository name
        let repo_name = format!(
            "{}-{}-{}",
            self.repository_prefix,
            name_suffix,
            Uuid::new_v4()
        );

        // Create repository via GitHub API POST request
        let route = format!("/orgs/{}/repos", self.organization);
        let body = serde_json::json!({
            "name": repo_name,
            "private": true,
            "auto_init": true,
            "description": "Test repository for Merge Warden integration testing"
        });

        let create_result: octocrab::models::Repository = self
            .github_client
            .post(route, Some(&body))
            .await
            .map_err(|e| TestError::github_api_error("create_repository", &e.to_string()))?;

        // Track for cleanup
        self.created_repositories.push(repo_name.clone());

        // Create TestRepository handle
        let repo = TestRepository {
            name: repo_name.clone(),
            organization: self.organization.clone(),
            id: create_result.id.0,
            full_name: format!("{}/{}", self.organization, repo_name),
            clone_url: create_result
                .clone_url
                .map(|u| u.to_string())
                .unwrap_or_default(),
            default_branch: create_result
                .default_branch
                .unwrap_or_else(|| "main".to_string()),
            private: true,
            created_at: chrono::Utc::now(),
        };

        Ok(repo)
    }

    /// Sets up merge-warden configuration for a test repository.
    ///
    /// This method configures the repository with:
    /// - merge-warden.toml configuration file
    /// - Branch protection rules
    /// - Repository settings optimized for testing
    ///
    /// # Parameters
    ///
    /// - `repository`: The repository to configure
    /// - `config_spec`: Optional custom configuration specification
    ///
    /// # Errors
    ///
    /// Returns `TestError::GitHubApiError` if configuration setup fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestRepositoryManager, RepositorySpec, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_repository_configuration() -> Result<(), TestError> {
    ///     let mut manager = TestRepositoryManager::new("token".to_string()).await?;
    ///     let repo = manager.create_repository("config-test").await?;
    ///
    ///     manager.setup_configuration(&repo, None).await?;
    ///
    ///     // Verify configuration was applied
    ///     let config_content = manager.get_file_content(&repo, ".github/merge-warden.toml").await?;
    ///     assert!(config_content.contains("schemaVersion"));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn setup_configuration(
        &self,
        repository: &TestRepository,
        _config_spec: Option<&RepositorySpec>,
    ) -> TestResult<()> {
        // Default merge-warden configuration
        let config_content = r#"schemaVersion = 1

[policies.pullRequests.prTitle]
format = "conventional-commits"

[policies.pullRequests.prBody]
requireWorkItemReference = false

[policies.pullRequests.prSize]
enabled = true
maxLines = 1000
"#;

        // Create .github directory and add merge-warden.toml
        self.add_file(
            repository,
            &repository.default_branch,
            ".github/merge-warden.toml",
            config_content,
            "Add merge-warden configuration",
        )
        .await?;

        Ok(())
    }

    /// Adds initial content to a test repository.
    ///
    /// This method populates the repository with standard files and structure
    /// needed for comprehensive testing of bot functionality.
    ///
    /// # Parameters
    ///
    /// - `repository`: The repository to populate
    /// - `content_spec`: Specification of files and content to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestRepositoryManager, FileChange, FileAction, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_content_addition() -> Result<(), TestError> {
    ///     let mut manager = TestRepositoryManager::new("token".to_string()).await?;
    ///     let repo = manager.create_repository("content-test").await?;
    ///
    ///     let files = vec![
    ///         FileChange {
    ///             path: "README.md".to_string(),
    ///             content: "# Test Repository".to_string(),
    ///             action: FileAction::Add,
    ///         }
    ///     ];
    ///
    ///     manager.add_initial_content(&repo, &files).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn add_initial_content(
        &self,
        repository: &TestRepository,
        files: &[FileChange],
    ) -> TestResult<()> {
        for file in files {
            match &file.action {
                FileAction::Add => {
                    self.add_file(
                        repository,
                        &repository.default_branch,
                        &file.path,
                        &file.content,
                        &format!("Add {}", file.path),
                    )
                    .await?;
                }
                _ => {
                    // Handle other actions if needed
                }
            }
        }
        Ok(())
    }

    /// Retrieves the content of a file from the repository.
    ///
    /// # Parameters
    ///
    /// - `repository`: The repository to read from
    /// - `file_path`: Path to the file within the repository
    ///
    /// # Returns
    ///
    /// The file content as a string.
    ///
    /// # Errors
    ///
    /// Returns `TestError::GitHubApiError` if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestRepositoryManager, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_file_retrieval() -> Result<(), TestError> {
    ///     let manager = TestRepositoryManager::new("token".to_string()).await?;
    ///     let repo = create_test_repository().await?;
    ///
    ///     let content = manager.get_file_content(&repo, "README.md").await?;
    ///     assert!(!content.is_empty());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_file_content(
        &self,
        repository: &TestRepository,
        file_path: &str,
    ) -> TestResult<String> {
        use base64::{engine::general_purpose, Engine as _};

        let content = self
            .github_client
            .repos(&repository.organization, &repository.name)
            .get_content()
            .path(file_path)
            .send()
            .await
            .map_err(|e| TestError::github_api_error("get_file_content", &e.to_string()))?;

        if let Some(encoded_content) = content.items[0].content.as_ref() {
            let decoded = general_purpose::STANDARD
                .decode(encoded_content.replace("\n", ""))
                .map_err(|e| TestError::environment_error("decode_file_content", &e.to_string()))?;
            String::from_utf8(decoded)
                .map_err(|e| TestError::environment_error("utf8_decode", &e.to_string()))
        } else {
            Err(TestError::environment_error(
                "get_file_content",
                "No content found",
            ))
        }
    }

    /// Cleans up all created repositories.
    ///
    /// This method deletes all repositories created during testing to ensure
    /// no resources are left behind.
    ///
    /// # Errors
    ///
    /// Returns `TestError::CleanupFailed` if any repositories cannot be deleted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestRepositoryManager, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_cleanup() -> Result<(), TestError> {
    ///     let mut manager = TestRepositoryManager::new("token".to_string()).await?;
    ///     let repo = manager.create_repository("cleanup-test").await?;
    ///
    ///     manager.cleanup().await?;
    ///
    ///     // Repository should be deleted
    ///     Ok(())
    /// }
    /// ```
    pub async fn cleanup(&mut self) -> TestResult<()> {
        let mut errors = Vec::new();

        // Delete all created repositories
        for repo_name in &self.created_repositories {
            if let Err(e) = self
                .github_client
                .repos(&self.organization, repo_name)
                .delete()
                .await
            {
                errors.push(format!("Failed to delete {}: {}", repo_name, e));
            }
        }

        self.created_repositories.clear();

        if !errors.is_empty() {
            return Err(TestError::cleanup_failed(
                "repositories",
                &errors.join("; "),
            ));
        }

        Ok(())
    }

    /// Gets the organization name for this manager.
    ///
    /// # Returns
    ///
    /// The GitHub organization name used for test repositories.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use merge_warden_integration_tests::TestRepositoryManager;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = TestRepositoryManager::new("123456".to_string(), "key".to_string(), "glitchgrove".to_string(), "prefix".to_string()).await?;
    /// assert_eq!(manager.organization(), "glitchgrove");
    /// # Ok(())
    /// # }
    /// ```
    pub fn organization(&self) -> &str {
        &self.organization
    }

    /// Gets the number of repositories created by this manager.
    ///
    /// # Returns
    ///
    /// The count of repositories created during this session.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use merge_warden_integration_tests::TestRepositoryManager;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut manager = TestRepositoryManager::new("123456".to_string(), "key".to_string(), "glitchgrove".to_string(), "prefix".to_string()).await?;
    /// assert_eq!(manager.repository_count(), 0);
    ///
    /// let repo = manager.create_repository("test").await?;
    /// assert_eq!(manager.repository_count(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn repository_count(&self) -> usize {
        self.created_repositories.len()
    }

    /// Adds content to repository.
    pub async fn add_content(
        &self,
        repository: &TestRepository,
        files: &[(String, String, FileAction)],
    ) -> TestResult<()> {
        for (path, content, action) in files {
            match action {
                FileAction::Add => {
                    self.add_file(
                        repository,
                        &repository.default_branch,
                        path,
                        content,
                        &format!("Add {}", path),
                    )
                    .await?;
                }
                _ => {
                    // For now, just handle Add action
                }
            }
        }
        Ok(())
    }

    /// Creates a branch in the repository.
    pub async fn create_branch(
        &self,
        repository: &TestRepository,
        branch_name: &str,
        from_branch: &str,
    ) -> TestResult<()> {
        // Get the SHA of the from_branch
        let from_ref = self
            .github_client
            .repos(&repository.organization, &repository.name)
            .get_ref(&octocrab::params::repos::Reference::Branch(
                from_branch.to_string(),
            ))
            .await
            .map_err(|e| TestError::github_api_error("get_ref", &e.to_string()))?;

        // Get the SHA from the ref object
        let from_sha = match &from_ref.object {
            octocrab::models::repos::Object::Commit { sha, .. } => sha.clone(),
            octocrab::models::repos::Object::Tag { sha, .. } => sha.clone(),
            _ => {
                return Err(TestError::github_api_error(
                    "create_branch",
                    "Unknown object type in ref",
                ))
            }
        };

        // Create new branch
        self.github_client
            .repos(&repository.organization, &repository.name)
            .create_ref(
                &octocrab::params::repos::Reference::Branch(branch_name.to_string()),
                from_sha,
            )
            .await
            .map_err(|e| TestError::github_api_error("create_ref", &e.to_string()))?;

        Ok(())
    }

    /// Adds a file to a branch.
    pub async fn add_file(
        &self,
        repository: &TestRepository,
        branch: &str,
        path: &str,
        content: &str,
        commit_message: &str,
    ) -> TestResult<()> {
        use base64::{engine::general_purpose, Engine as _};

        // Encode content as base64
        let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());

        // Fetch existing file SHA so we can update rather than create when the file
        // already exists (e.g. auto_init creates README.md on repo creation).
        let existing_sha = self
            .github_client
            .repos(&repository.organization, &repository.name)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await
            .ok()
            .and_then(|f| f.items.into_iter().next())
            .map(|item| item.sha);

        if let Some(sha) = existing_sha {
            self.github_client
                .repos(&repository.organization, &repository.name)
                .update_file(path, commit_message, &encoded_content, &sha)
                .branch(branch)
                .send()
                .await
                .map_err(|e| TestError::github_api_error("update_file", &e.to_string()))?;
        } else {
            self.github_client
                .repos(&repository.organization, &repository.name)
                .create_file(path, commit_message, &encoded_content)
                .branch(branch)
                .send()
                .await
                .map_err(|e| TestError::github_api_error("create_file", &e.to_string()))?;
        }

        Ok(())
    }

    /// Updates a file in a branch, or creates it if it does not yet exist (upsert).
    pub async fn update_file(
        &self,
        repository: &TestRepository,
        branch: &str,
        path: &str,
        content: &str,
        commit_message: &str,
    ) -> TestResult<()> {
        use base64::{engine::general_purpose, Engine as _};

        // Attempt to fetch the current file SHA for the update API.
        // If the file does not exist (e.g. fresh test repo) fall through to create_file.
        let existing_sha = self
            .github_client
            .repos(&repository.organization, &repository.name)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await
            .ok()
            .and_then(|f| f.items.into_iter().next())
            .map(|item| item.sha);

        let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());

        if existing_sha.is_none() {
            // File does not exist yet — create it
            self.github_client
                .repos(&repository.organization, &repository.name)
                .create_file(path, commit_message, &encoded_content)
                .branch(branch)
                .send()
                .await
                .map_err(|e| TestError::github_api_error("create_file_upsert", &e.to_string()))?;
            return Ok(());
        }

        let file_sha = existing_sha.unwrap();

        // Update file
        self.github_client
            .repos(&repository.organization, &repository.name)
            .update_file(path, commit_message, &encoded_content, &file_sha)
            .branch(branch)
            .send()
            .await
            .map_err(|e| TestError::github_api_error("update_file", &e.to_string()))?;

        Ok(())
    }

    /// Creates a pull request.
    pub async fn create_pull_request(
        &self,
        repository: &TestRepository,
        spec: &crate::utils::PullRequestSpec,
    ) -> TestResult<crate::utils::TestPullRequest> {
        let pr = self
            .github_client
            .pulls(&repository.organization, &repository.name)
            .create(&spec.title, &spec.source_branch, &spec.target_branch)
            .body(&spec.body)
            .draft(spec.draft)
            .send()
            .await
            .map_err(|e| TestError::github_api_error("create_pull_request", &e.to_string()))?;

        // Add labels if specified
        if !spec.labels.is_empty() {
            self.github_client
                .issues(&repository.organization, &repository.name)
                .add_labels(pr.number, &spec.labels)
                .await
                .map_err(|e| TestError::github_api_error("add_labels", &e.to_string()))?;
        }

        Ok(crate::utils::TestPullRequest {
            number: pr.number,
            id: pr.id.0,
            title: pr.title.unwrap_or_default(),
            body: pr.body.unwrap_or_default(),
            head: spec.source_branch.clone(),
            base: spec.target_branch.clone(),
            repo_full_name: repository.full_name.clone(),
        })
    }

    /// Gets checks for a pull request.
    pub async fn get_pr_checks(
        &self,
        repository: &TestRepository,
        pr_number: u64,
    ) -> TestResult<Vec<crate::environment::PullRequestCheck>> {
        // Get PR to get head SHA
        let pr = self
            .github_client
            .pulls(&repository.organization, &repository.name)
            .get(pr_number)
            .await
            .map_err(|e| TestError::github_api_error("get_pull_request", &e.to_string()))?;

        let head_sha = pr.head.sha;

        // Get check runs - use head SHA as Commitish
        let check_runs = self
            .github_client
            .checks(&repository.organization, &repository.name)
            .list_check_runs_for_git_ref(head_sha.into())
            .send()
            .await
            .map_err(|e| TestError::github_api_error("list_check_runs", &e.to_string()))?;

        let checks = check_runs
            .check_runs
            .into_iter()
            .map(|run| crate::environment::PullRequestCheck {
                id: run.id.to_string(),
                name: run.name,
                conclusion: run.conclusion.map(|c| c.to_string()),
                details_url: run.details_url.map(|u| u.to_string()),
                output: crate::environment::CheckOutput {
                    summary: run.output.summary.clone().unwrap_or_default(),
                    text: run.output.text.clone(),
                },
            })
            .collect();

        Ok(checks)
    }

    /// Gets comments for a pull request.
    pub async fn get_pr_comments(
        &self,
        repository: &TestRepository,
        pr_number: u64,
    ) -> TestResult<Vec<crate::environment::PullRequestComment>> {
        let comments_page = self
            .github_client
            .issues(&repository.organization, &repository.name)
            .list_comments(pr_number)
            .send()
            .await
            .map_err(|e| TestError::github_api_error("list_comments", &e.to_string()))?;

        let comments = comments_page
            .items
            .into_iter()
            .map(|comment| crate::environment::PullRequestComment {
                id: comment.id.0,
                body: comment.body.unwrap_or_default(),
                user: crate::environment::CommentUser {
                    login: comment.user.login,
                    id: comment.user.id.0,
                },
                created_at: comment.created_at.to_rfc3339(),
            })
            .collect();

        Ok(comments)
    }

    /// Gets labels for a pull request.
    pub async fn get_pr_labels(
        &self,
        repository: &TestRepository,
        pr_number: u64,
    ) -> TestResult<Vec<crate::environment::PullRequestLabel>> {
        let issue = self
            .github_client
            .issues(&repository.organization, &repository.name)
            .get(pr_number)
            .await
            .map_err(|e| TestError::github_api_error("get_issue", &e.to_string()))?;

        let labels = issue
            .labels
            .into_iter()
            .map(|label| crate::environment::PullRequestLabel {
                id: label.id.0,
                name: label.name,
                color: label.color,
            })
            .collect();

        Ok(labels)
    }
}

/// Specification for repository configuration during setup.
///
/// This struct defines how a test repository should be configured with
/// merge-warden settings and repository policies.
#[derive(Debug, Clone)]
pub struct RepositorySpec {
    /// Custom merge-warden.toml configuration content
    pub merge_warden_config: Option<String>,
    /// Branch protection settings
    pub branch_protection: BranchProtectionSpec,
    /// Repository visibility settings
    pub visibility: RepositoryVisibility,
    /// Initial branch structure
    pub branches: Vec<String>,
}

impl Default for RepositorySpec {
    fn default() -> Self {
        Self {
            merge_warden_config: None,
            branch_protection: BranchProtectionSpec::default(),
            visibility: RepositoryVisibility::Private,
            branches: vec!["main".to_string()],
        }
    }
}

/// Branch protection configuration for test repositories.
#[derive(Debug, Clone)]
pub struct BranchProtectionSpec {
    /// Whether to require status checks
    pub require_status_checks: bool,
    /// Required status check contexts
    pub required_checks: Vec<String>,
    /// Whether to require pull request reviews
    pub require_pr_reviews: bool,
    /// Number of required approving reviews
    pub required_review_count: u32,
    /// Whether to dismiss stale reviews
    pub dismiss_stale_reviews: bool,
    /// Whether to enforce restrictions for administrators
    pub enforce_for_admins: bool,
}

impl Default for BranchProtectionSpec {
    fn default() -> Self {
        Self {
            require_status_checks: true,
            required_checks: vec!["merge-warden".to_string()],
            require_pr_reviews: true,
            required_review_count: 1,
            dismiss_stale_reviews: true,
            enforce_for_admins: false,
        }
    }
}

/// Repository visibility settings.
#[derive(Debug, Clone, PartialEq)]
pub enum RepositoryVisibility {
    /// Repository is private (recommended for testing)
    Private,
    /// Repository is public
    Public,
    /// Repository is internal (organization members only)
    Internal,
}

/// Specification for file changes in repository operations.
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Path to the file within the repository
    pub path: String,
    /// Content of the file
    pub content: String,
    /// Action to perform on the file
    pub action: FileAction,
}

/// Actions that can be performed on repository files.
#[derive(Debug, Clone, PartialEq)]
pub enum FileAction {
    /// Add a new file
    Add,
    /// Modify an existing file
    Modify,
    /// Delete an existing file
    Delete,
}
