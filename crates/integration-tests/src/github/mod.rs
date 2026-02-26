//! GitHub integration components for integration testing.
//!
//! This module provides comprehensive GitHub integration for testing the Merge Warden bot,
//! including repository management, webhook configuration, and GitHub App installation.

pub mod repository_manager;

pub use repository_manager::{FileAction, FileChange, RepositorySpec, TestRepositoryManager};

use std::collections::HashMap;
use std::time::Duration;

use crate::environment::{BotConfiguration, TestRepository};
use crate::errors::{TestError, TestResult};

/// GitHub App and webhook configuration for bot testing.
///
/// The `TestBotInstance` manages all aspects of setting up a GitHub App for testing,
/// including authentication, webhook configuration, and permission verification.
///
/// # Architecture
///
/// The bot instance handles several key areas:
/// - **Authentication**: JWT token generation and GitHub App authentication
/// - **Installation**: Installing the GitHub App on test repositories
/// - **Webhooks**: Setting up webhook endpoints and signature validation
/// - **Permissions**: Verifying and managing bot permissions
///
/// # Examples
///
/// ```rust
/// use merge_warden_integration_tests::{TestBotInstance, TestError};
///
/// #[tokio::test]
/// async fn test_bot_setup() -> Result<(), TestError> {
///     let bot = TestBotInstance::new_for_testing().await?;
///     let config = bot.configure_for_repository(&test_repo).await?;
///     assert!(!config.installation_id.is_empty());
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct TestBotInstance {
    /// GitHub App ID for authentication
    app_id: String,
    /// GitHub App private key content
    #[allow(dead_code)]
    private_key: String,
    /// Webhook secret for signature validation
    #[allow(dead_code)]
    webhook_secret: String,
    /// Base URL for webhook endpoints
    base_webhook_url: String,
    /// Optional ngrok tunnel for local development
    ngrok_tunnel: Option<String>,
    /// GitHub API client for app operations (org-scoped installation token)
    github_client: octocrab::Octocrab,
}

impl TestBotInstance {
    /// Creates a new bot instance configured for testing.
    ///
    /// This method initializes a GitHub App instance using test credentials
    /// and prepares it for integration testing operations.
    ///
    /// # Environment Variables Required
    ///
    /// - `REPO_CREATION_APP_ID`: GitHub App ID for testing
    /// - `REPO_CREATION_APP_PRIVATE_KEY`: GitHub App private key content
    /// - `GITHUB_TEST_WEBHOOK_SECRET`: Webhook secret for signature validation
    ///
    /// # Environment Variables Optional
    ///
    /// - `LOCAL_WEBHOOK_ENDPOINT`: Local webhook endpoint (default: "http://localhost:7071/api/webhook")
    /// - `NGROK_AUTH_TOKEN`: Ngrok authentication token for tunnel creation
    ///
    /// # Returns
    ///
    /// A configured `TestBotInstance` ready for repository setup.
    ///
    /// # Errors
    ///
    /// Returns `TestError::InvalidConfiguration` if:
    /// - Required environment variables are missing
    /// - GitHub App credentials are invalid
    /// - Private key cannot be parsed
    ///
    /// Returns `TestError::AuthenticationError` if:
    /// - GitHub App authentication fails
    /// - JWT token generation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestBotInstance, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_bot_initialization() -> Result<(), TestError> {
    ///     let bot = TestBotInstance::new_for_testing().await?;
    ///     assert!(!bot.app_id().is_empty());
    ///     Ok(())
    /// }
    /// ```
    /// Creates a `TestBotInstance` from `TestConfig`, authenticating as the
    /// Merge Warden GitHub App.
    ///
    /// Generates a JWT from the Merge Warden app credentials and exchanges it
    /// for an installation access token scoped to the test organisation, then
    /// builds an authenticated octocrab client. The instance is ready to read
    /// PR state and verify that Merge Warden has performed the expected actions.
    ///
    /// # Parameters
    ///
    /// - `config`: Test configuration containing Merge Warden app credentials
    ///   and organisation name.
    ///
    /// # Errors
    ///
    /// Returns `TestError::AuthenticationError` if the private key is invalid
    /// or JWT signing fails.
    ///
    /// Returns `TestError::GitHubApiError` if the installation cannot be found
    /// or the access token request fails.
    pub async fn from_config(config: &crate::environment::TestConfig) -> TestResult<Self> {
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
        use std::time::{SystemTime, UNIX_EPOCH};

        // In mock-services mode, skip real JWT auth and return a stub instance.
        if config.use_mock_services {
            let github_client = octocrab::Octocrab::builder().build().map_err(|e| {
                TestError::environment_error("build_mock_bot_client", &e.to_string())
            })?;
            return Ok(TestBotInstance {
                app_id: config.merge_warden_app_id.clone(),
                private_key: config.merge_warden_app_private_key.clone(),
                webhook_secret: config.merge_warden_webhook_secret.clone(),
                base_webhook_url: config.local_webhook_endpoint.clone(),
                ngrok_tunnel: None,
                github_client,
            });
        }

        let encoding_key = EncodingKey::from_rsa_pem(
            config.merge_warden_app_private_key.as_bytes(),
        )
        .map_err(|e| TestError::authentication_error("merge_warden_private_key", &e.to_string()))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let app_id_num: u64 = config.merge_warden_app_id.parse().map_err(|_| {
            TestError::InvalidConfiguration(
                "MERGE_WARDEN_APP_ID must be a valid integer".to_string(),
            )
        })?;

        #[derive(serde::Serialize)]
        struct JwtClaims {
            iat: i64,
            exp: i64,
            iss: u64,
        }

        let claims = JwtClaims {
            iat: now - 60,
            exp: now + 540,
            iss: app_id_num,
        };

        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
            .map_err(|e| TestError::authentication_error("merge_warden_jwt", &e.to_string()))?;

        let jwt_client = octocrab::Octocrab::builder()
            .personal_token(jwt)
            .build()
            .map_err(|e| TestError::environment_error("build_mw_jwt_client", &e.to_string()))?;

        let installation: serde_json::Value = jwt_client
            .get(
                format!("/orgs/{}/installation", config.github_organization),
                None::<&()>,
            )
            .await
            .map_err(|e| TestError::github_api_error("get_mw_org_installation", &e.to_string()))?;

        let installation_id = installation["id"].as_u64().ok_or_else(|| {
            TestError::environment_error(
                "parse_mw_installation_id",
                "No installation id found in response",
            )
        })?;

        let token_response: serde_json::Value = jwt_client
            .post(
                format!("/app/installations/{}/access_tokens", installation_id),
                Some(&serde_json::json!({})),
            )
            .await
            .map_err(|e| {
                TestError::github_api_error("create_mw_installation_access_token", &e.to_string())
            })?;

        let access_token = token_response["token"]
            .as_str()
            .ok_or_else(|| {
                TestError::environment_error(
                    "parse_mw_access_token",
                    "No token field in installation token response",
                )
            })?
            .to_string();

        let github_client = octocrab::Octocrab::builder()
            .personal_token(access_token)
            .build()
            .map_err(|e| {
                TestError::environment_error("build_mw_authenticated_client", &e.to_string())
            })?;

        Ok(TestBotInstance {
            app_id: config.merge_warden_app_id.clone(),
            private_key: config.merge_warden_app_private_key.clone(),
            webhook_secret: config.merge_warden_webhook_secret.clone(),
            base_webhook_url: config.local_webhook_endpoint.clone(),
            ngrok_tunnel: None,
            github_client,
        })
    }

    /// Configures the bot for testing with a specific repository.
    ///
    /// This method performs the complete bot setup process for a test repository:
    /// 1. Installs the GitHub App on the repository
    /// 2. Generates installation access tokens
    /// 3. Configures webhook endpoints
    /// 4. Verifies required permissions
    /// 5. Sets up local tunnel if needed for development testing
    ///
    /// # Parameters
    ///
    /// - `repository`: The test repository to configure bot access for
    ///
    /// # Returns
    ///
    /// A `BotConfiguration` containing all setup details including access tokens,
    /// webhook URLs, and permission information.
    ///
    /// # Errors
    ///
    /// Returns `TestError::GitHubApiError` if:
    /// - GitHub App installation fails
    /// - Repository access is denied
    /// - Webhook creation fails
    ///
    /// Returns `TestError::AuthenticationError` if:
    /// - Installation token generation fails
    /// - Permission verification fails
    ///
    /// Returns `TestError::EnvironmentError` if:
    /// - Local tunnel setup fails
    /// - Webhook endpoint is not accessible
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestBotInstance, TestRepository, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_repository_configuration() -> Result<(), TestError> {
    ///     let bot = TestBotInstance::new_for_testing().await?;
    ///     let repo = create_test_repository().await?;
    ///
    ///     let config = bot.configure_for_repository(&repo).await?;
    ///
    ///     assert!(!config.installation_id.is_empty());
    ///     assert!(!config.access_token.is_empty());
    ///     assert!(config.permissions.contains_key("pull_requests"));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn configure_for_repository(
        &mut self,
        _repository: &TestRepository,
    ) -> TestResult<BotConfiguration> {
        // With the embedded webhook server the MW app is installed at the organisation
        // level and `start_local_webhook_server` already set `base_webhook_url`. There
        // is no per-repository installation step required — just return the current
        // state so callers can inspect the active configuration.
        Ok(BotConfiguration {
            installation_id: self.app_id.clone(),
            access_token: String::new(), // raw token not stored; server uses its own copy
            webhook_url: self.base_webhook_url.clone(),
            webhook_secret: self.webhook_secret.clone(),
            permissions: HashMap::new(),
        })
    }

    /// Sets up a local tunnel for webhook testing during development.
    ///
    /// This method creates an ngrok tunnel to forward webhooks from GitHub to
    /// the local development environment, enabling end-to-end testing without
    /// deploying to a public endpoint.
    ///
    /// # Returns
    ///
    /// The public tunnel URL that can be used as a webhook endpoint.
    ///
    /// # Errors
    ///
    /// Returns `TestError::EnvironmentError` if:
    /// - Ngrok is not installed or accessible
    /// - Tunnel creation fails
    /// - Local webhook endpoint is not accessible
    ///
    /// # Note
    ///
    /// This method requires ngrok to be installed and optionally configured
    /// with an authentication token for persistent tunnels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestBotInstance, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_local_tunnel() -> Result<(), TestError> {
    ///     let mut bot = TestBotInstance::new_for_testing().await?;
    ///     let tunnel_url = bot.setup_local_tunnel().await?;
    ///
    ///     assert!(tunnel_url.starts_with("https://"));
    ///     assert!(tunnel_url.contains("ngrok"));
    ///
    ///     Ok(())
    /// }
    /// ```
    /// Starts an embedded axum HTTP server that processes webhooks using the core
    /// Merge Warden business logic.
    ///
    /// The server binds to a random localhost port, verifies HMAC-SHA256 signatures,
    /// parses pull_request payloads, loads per-repo merge-warden.toml configuration,
    /// and runs the full PR validation pipeline — writing check runs and comments back
    /// to GitHub using the installation-access-token client stored on this instance.
    ///
    /// No Azure Key Vault or App Configuration infrastructure is required.
    ///
    /// # Returns
    ///
    /// The `http://127.0.0.1:{port}/api/webhook` URL on success.
    pub async fn start_local_webhook_server(&mut self) -> TestResult<String> {
        use axum::{extract::State, http::HeaderMap, routing::post, Router};
        use merge_warden_core::{
            config::{
                load_merge_warden_config, ApplicationDefaults,
                CurrentPullRequestValidationConfiguration,
            },
            MergeWarden, WebhookPayload,
        };
        use merge_warden_developer_platforms::github::GitHubProvider;
        use std::sync::Arc;
        use tokio::net::TcpListener;

        struct WebhookState {
            github_client: octocrab::Octocrab,
            webhook_secret: String,
        }

        async fn webhook_handler(
            State(state): State<Arc<WebhookState>>,
            headers: HeaderMap,
            body: String,
        ) -> axum::http::StatusCode {
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            type HmacSha256 = Hmac<Sha256>;

            // Verify HMAC-SHA256 signature
            let signature = match headers.get("X-Hub-Signature-256") {
                Some(v) => v.to_str().unwrap_or("").to_string(),
                None => return axum::http::StatusCode::UNAUTHORIZED,
            };
            let mut mac = match HmacSha256::new_from_slice(state.webhook_secret.as_bytes()) {
                Ok(m) => m,
                Err(_) => return axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            mac.update(body.as_bytes());
            let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
            if signature != expected {
                return axum::http::StatusCode::UNAUTHORIZED;
            }

            // Parse payload
            let payload: WebhookPayload = match serde_json::from_str(&body) {
                Ok(p) => p,
                Err(_) => return axum::http::StatusCode::BAD_REQUEST,
            };

            // Only process relevant PR actions
            let action = payload.action.as_str();
            if !matches!(
                action,
                "opened" | "edited" | "ready_for_review" | "reopened" | "unlocked" | "synchronize"
            ) {
                return axum::http::StatusCode::OK;
            }

            let pr = match payload.pull_request {
                Some(p) => p,
                None => return axum::http::StatusCode::BAD_REQUEST,
            };
            let repo = match payload.repository {
                Some(r) => r,
                None => return axum::http::StatusCode::BAD_REQUEST,
            };

            let parts: Vec<&str> = repo.full_name.split('/').collect();
            if parts.len() != 2 {
                return axum::http::StatusCode::BAD_REQUEST;
            }
            let owner = parts[0];

            // Build provider from shared client (org-scoped installation token)
            let provider = GitHubProvider::new(state.github_client.clone());
            let app_defaults = ApplicationDefaults::default();

            let validation_config = match load_merge_warden_config(
                owner,
                &repo.name,
                ".github/merge-warden.toml",
                &provider,
                &app_defaults,
            )
            .await
            {
                Ok(cfg) => cfg.to_validation_config(&app_defaults.bypass_rules),
                Err(_) => CurrentPullRequestValidationConfiguration::default(),
            };

            let warden = MergeWarden::with_config(provider, validation_config);
            let _ = warden
                .process_pull_request(owner, &repo.name, pr.number)
                .await;

            axum::http::StatusCode::OK
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| TestError::environment_error("bind_webhook_server", &e.to_string()))?;
        let port = listener
            .local_addr()
            .map_err(|e| TestError::environment_error("webhook_server_addr", &e.to_string()))?
            .port();

        let url = format!("http://127.0.0.1:{}/api/webhook", port);

        let state = Arc::new(WebhookState {
            github_client: self.github_client.clone(),
            webhook_secret: self.webhook_secret.clone(),
        });

        let app = Router::new()
            .route("/api/webhook", post(webhook_handler))
            .with_state(state);

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap_or_else(|e| {
                eprintln!("Embedded webhook server error: {}", e);
            });
        });

        self.base_webhook_url = url.clone();
        self.ngrok_tunnel = Some(url.clone());
        Ok(url)
    }

    /// Sets up a webhook endpoint for integration testing.
    ///
    /// Prefers `LOCAL_WEBHOOK_ENDPOINT` (env var / repo secret pointing at a deployed
    /// Merge Warden instance) when set to a non-localhost URL. Otherwise starts an
    /// embedded webhook server on a random localhost port via [`start_local_webhook_server`].
    pub async fn setup_local_tunnel(&mut self) -> TestResult<String> {
        if let Ok(url) = std::env::var("LOCAL_WEBHOOK_ENDPOINT") {
            if !url.is_empty() && !url.starts_with("http://localhost") {
                self.base_webhook_url = url.clone();
                self.ngrok_tunnel = Some(url.clone());
                return Ok(url);
            }
        }

        self.start_local_webhook_server().await
    }

    /// Verifies that the bot has all required permissions on a repository.
    ///
    /// This method checks that the GitHub App installation has all the permissions
    /// necessary for Merge Warden operation, including pull request access,
    /// issue management, and check status updates.
    ///
    /// # Parameters
    ///
    /// - `repository`: The repository to verify permissions for
    ///
    /// # Returns
    ///
    /// A map of permission names to access levels (e.g., "read", "write", "admin").
    ///
    /// # Errors
    ///
    /// Returns `TestError::AuthenticationError` if:
    /// - Installation access token is invalid
    /// - Required permissions are missing
    /// - Permission verification API calls fail
    ///
    /// # Required Permissions
    ///
    /// - `issues`: write (for commenting and labeling)
    /// - `pull_requests`: write (for status checks and comments)
    /// - `contents`: read (for reading configuration and file changes)
    /// - `metadata`: read (for repository information)
    /// - `checks`: write (for status check updates)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestBotInstance, TestRepository, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_permission_verification() -> Result<(), TestError> {
    ///     let bot = TestBotInstance::new_for_testing().await?;
    ///     let repo = create_test_repository().await?;
    ///
    ///     let permissions = bot.verify_permissions(&repo).await?;
    ///
    ///     assert_eq!(permissions.get("issues"), Some(&"write".to_string()));
    ///     assert_eq!(permissions.get("pull_requests"), Some(&"write".to_string()));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn verify_permissions(
        &self,
        _repository: &TestRepository,
    ) -> TestResult<HashMap<String, String>> {
        // TODO: implement - Verify bot permissions on repository
        todo!("Verify bot permissions")
    }

    /// Simulates webhook delivery to test bot response.
    ///
    /// This method sends a simulated webhook payload to the configured webhook
    /// endpoint, allowing testing of bot response without requiring actual
    /// GitHub events.
    ///
    /// # Parameters
    ///
    /// - `event_type`: The GitHub event type (e.g., "pull_request", "issue_comment")
    /// - `payload`: The JSON payload for the webhook event
    ///
    /// # Returns
    ///
    /// The HTTP response from the webhook endpoint, including status code and body.
    ///
    /// # Errors
    ///
    /// Returns `TestError::NetworkError` if:
    /// - Webhook endpoint is not accessible
    /// - HTTP request fails
    /// - Network connectivity issues occur
    ///
    /// Returns `TestError::AuthenticationError` if:
    /// - Webhook signature validation fails
    /// - Authentication headers are incorrect
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestBotInstance, TestError};
    /// use serde_json::json;
    ///
    /// #[tokio::test]
    /// async fn test_webhook_simulation() -> Result<(), TestError> {
    ///     let bot = TestBotInstance::new_for_testing().await?;
    ///
    ///     let payload = json!({
    ///         "action": "opened",
    ///         "pull_request": {
    ///             "id": 123,
    ///             "title": "feat: test feature",
    ///             "body": "Test PR description"
    ///         }
    ///     });
    ///
    ///     let response = bot.simulate_webhook("pull_request", &payload).await?;
    ///     assert_eq!(response.status_code, 200);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn simulate_webhook(
        &self,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> TestResult<WebhookResponse> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        if self.base_webhook_url.is_empty() {
            return Err(TestError::environment_error(
                "simulate_webhook",
                "Webhook endpoint URL is not configured. Call setup_local_tunnel() or set \
                 WEBHOOK_ENDPOINT_URL before simulating webhooks.",
            ));
        }

        let payload_bytes =
            serde_json::to_vec(payload).map_err(|e| TestError::InternalError(e.to_string()))?;

        // Sign the payload with the webhook secret (matches GitHub's signature scheme)
        let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
            .map_err(|e| TestError::environment_error("simulate_webhook_hmac", &e.to_string()))?;
        mac.update(&payload_bytes);
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let start = std::time::Instant::now();

        let client = reqwest::Client::new();
        let response = client
            .post(&self.base_webhook_url)
            .header("Content-Type", "application/json")
            .header("X-GitHub-Event", event_type)
            .header("X-Hub-Signature-256", &signature)
            .header("X-GitHub-Delivery", uuid::Uuid::new_v4().to_string())
            .body(payload_bytes)
            .send()
            .await
            .map_err(|e| TestError::NetworkError(e.to_string()))?;

        let processing_time = start.elapsed();
        let status_code = response.status().as_u16();

        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        let body = response
            .text()
            .await
            .map_err(|e| TestError::NetworkError(e.to_string()))?;

        Ok(WebhookResponse {
            status_code,
            headers,
            body,
            processing_time,
        })
    }

    /// Gets the GitHub App ID for this bot instance.
    ///
    /// # Returns
    ///
    /// The GitHub App ID as a string.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use merge_warden_integration_tests::{TestBotInstance, environment::TestConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config: TestConfig = unimplemented!();
    /// let bot = TestBotInstance::from_config(&config).await?;
    /// let app_id = bot.app_id();
    /// assert!(!app_id.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Gets the current webhook endpoint URL.
    ///
    /// # Returns
    ///
    /// The webhook endpoint URL, which may be a local tunnel URL during development
    /// or a configured endpoint URL.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use merge_warden_integration_tests::{TestBotInstance, environment::TestConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config: TestConfig = unimplemented!();
    /// let bot = TestBotInstance::from_config(&config).await?;
    /// let webhook_url = bot.webhook_endpoint();
    /// assert!(webhook_url.starts_with("http"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn webhook_endpoint(&self) -> &str {
        self.ngrok_tunnel.as_ref().unwrap_or(&self.base_webhook_url)
    }

    /// Gets the check runs for a pull request using the Merge Warden App token.
    ///
    /// The Repo-Creation App token does not have `checks:read` permission, so check
    /// runs must be queried through the MW App installation token held by this
    /// instance.
    pub async fn get_pr_checks(
        &self,
        repository: &TestRepository,
        pr_number: u64,
    ) -> TestResult<Vec<crate::environment::PullRequestCheck>> {
        let pr = self
            .github_client
            .pulls(&repository.organization, &repository.name)
            .get(pr_number)
            .await
            .map_err(|e| TestError::github_api_error("get_pull_request", &e.to_string()))?;

        let head_sha = pr.head.sha;

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

    /// Generates a JWT token for GitHub App authentication.
    ///
    /// This method creates a JWT token signed with the GitHub App's private key
    /// for authenticating with GitHub APIs.
    ///
    /// # Returns
    ///
    /// A JWT token string valid for GitHub App authentication.
    ///
    /// # Errors
    ///
    /// Returns `TestError::AuthenticationError` if:
    /// - Private key cannot be parsed
    /// - JWT token generation fails
    /// - Token signing fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use merge_warden_integration_tests::{TestBotInstance, TestError};
    ///
    /// #[tokio::test]
    /// async fn test_jwt_generation() -> Result<(), TestError> {
    ///     let bot = TestBotInstance::new_for_testing().await?;
    ///     let jwt_token = bot.generate_jwt_token()?;
    ///
    ///     assert!(!jwt_token.is_empty());
    ///     // JWT tokens have three parts separated by dots
    ///     assert_eq!(jwt_token.split('.').count(), 3);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn generate_jwt_token(&self) -> TestResult<String> {
        // TODO: implement - Generate JWT token for GitHub App auth
        todo!("Generate JWT token for GitHub App authentication")
    }
}

/// Response from webhook endpoint during testing.
///
/// This struct captures the HTTP response from webhook delivery for validation
/// and debugging purposes.
#[derive(Debug, Clone)]
pub struct WebhookResponse {
    /// HTTP status code returned by the webhook endpoint
    pub status_code: u16,
    /// Response headers from the webhook endpoint
    pub headers: HashMap<String, String>,
    /// Response body content
    pub body: String,
    /// Time taken to process the webhook
    pub processing_time: Duration,
}

impl WebhookResponse {
    /// Checks if the webhook response indicates success.
    ///
    /// # Returns
    ///
    /// `true` if the status code indicates success (2xx), `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::WebhookResponse;
    /// # use std::collections::HashMap;
    /// # use std::time::Duration;
    /// let response = WebhookResponse {
    ///     status_code: 200,
    ///     headers: HashMap::new(),
    ///     body: "OK".to_string(),
    ///     processing_time: Duration::from_millis(100),
    /// };
    /// assert!(response.is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// Checks if the response time is within acceptable limits.
    ///
    /// # Parameters
    ///
    /// - `limit`: Maximum acceptable processing time
    ///
    /// # Returns
    ///
    /// `true` if processing time is within the limit, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use merge_warden_integration_tests::WebhookResponse;
    /// # use std::collections::HashMap;
    /// # use std::time::Duration;
    /// let response = WebhookResponse {
    ///     status_code: 200,
    ///     headers: HashMap::new(),
    ///     body: "OK".to_string(),
    ///     processing_time: Duration::from_millis(100),
    /// };
    /// assert!(response.is_within_time_limit(Duration::from_secs(1)));
    /// assert!(!response.is_within_time_limit(Duration::from_millis(50)));
    /// ```
    pub fn is_within_time_limit(&self, limit: Duration) -> bool {
        self.processing_time <= limit
    }
}
