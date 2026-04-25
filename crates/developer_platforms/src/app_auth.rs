//! GitHub App authentication provider.
//!
//! Provides a concrete [`AuthenticationProvider`] implementation for GitHub Apps.
//! Uses RS256 JWT signing for app-level authentication and exchanges JWTs for
//! installation-scoped access tokens.
//!
//! # Usage
//!
//! ```rust,no_run
//! use merge_warden_developer_platforms::app_auth::AppAuthProvider;
//! use github_bot_sdk::{
//!     auth::InstallationId,
//!     client::{ClientConfig, GitHubClient},
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let auth = AppAuthProvider::new(
//!     12345,
//!     "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----",
//!     "https://api.github.com",
//! )?;
//!
//! let github_client = GitHubClient::builder(auth).build()?;
//! let installation_client = github_client
//!     .installation_by_id(InstallationId::new(678901))
//!     .await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use github_bot_sdk::{
    auth::{
        jwt::{JwtGenerator, RS256JwtGenerator},
        AuthenticationProvider, GitHubAppId, Installation, InstallationId, InstallationPermissions,
        InstallationToken, JsonWebToken, PrivateKey, Repository,
    },
    error::AuthError,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, instrument, warn};

/// JSON response from GitHub's create installation access token endpoint.
///
/// `POST /app/installations/{installation_id}/access_tokens`
#[derive(Deserialize)]
struct InstallationAccessTokenResponse {
    /// The installation access token.
    token: String,
    /// When the token expires (typically 1 hour from creation).
    expires_at: DateTime<Utc>,
}

/// Concrete GitHub App authentication provider.
///
/// Implements [`AuthenticationProvider`] for GitHub Apps using:
/// - RS256 JWT signing to authenticate as the GitHub App
/// - GitHub's REST API to exchange JWTs for installation-scoped access tokens
/// - In-memory caching to avoid redundant token exchanges
///
/// # Construction
///
/// ```rust,no_run
/// use merge_warden_developer_platforms::app_auth::AppAuthProvider;
///
/// let auth = AppAuthProvider::new(
///     12345,                                   // GitHub App ID
///     "-----BEGIN RSA PRIVATE KEY-----\n...", // RSA private key (PEM)
///     "https://api.github.com",               // GitHub API URL
/// ).expect("Invalid private key");
/// ```
pub struct AppAuthProvider {
    /// GitHub App identifier.
    app_id: GitHubAppId,
    /// JWT generator using the app's RSA private key.
    jwt_generator: RS256JwtGenerator,
    /// GitHub API base URL (without trailing slash).
    api_url: String,
    /// HTTP client for GitHub API calls.
    http_client: reqwest::Client,
    /// In-memory cache of installation tokens keyed by installation ID.
    token_cache: Arc<RwLock<HashMap<u64, InstallationToken>>>,
}

impl AppAuthProvider {
    /// Creates a new `AppAuthProvider` from the given GitHub App credentials.
    ///
    /// # Arguments
    ///
    /// * `app_id` - The numeric GitHub App ID
    /// * `private_key_pem` - PEM-encoded RSA private key (PKCS#1 format)
    /// * `api_url` - GitHub API base URL (e.g., `"https://api.github.com"`)
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::InvalidPrivateKey`] if the PEM string cannot be parsed as
    /// a valid RSA private key.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use merge_warden_developer_platforms::app_auth::AppAuthProvider;
    ///
    /// let pem = std::fs::read_to_string("/path/to/private-key.pem").unwrap();
    /// let auth = AppAuthProvider::new(12345, &pem, "https://api.github.com").unwrap();
    /// ```
    pub fn new(app_id: u64, private_key_pem: &str, api_url: &str) -> Result<Self, AuthError> {
        let private_key =
            PrivateKey::from_pem(private_key_pem).map_err(|e| AuthError::InvalidPrivateKey {
                message: format!("Failed to parse RSA private key: {}", e),
            })?;

        let jwt_generator = RS256JwtGenerator::new(private_key);
        let http_client = reqwest::Client::new();

        Ok(Self {
            app_id: GitHubAppId::new(app_id),
            jwt_generator,
            api_url: api_url.trim_end_matches('/').to_string(),
            http_client,
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generates a short-lived RS256 JWT for app-level API access.
    #[instrument(skip(self), fields(app_id = self.app_id.as_u64()))]
    async fn generate_app_jwt(&self) -> Result<JsonWebToken, AuthError> {
        debug!(app_id = self.app_id.as_u64(), "Generating app JWT");
        self.jwt_generator.generate_jwt(self.app_id).await
    }

    /// Exchanges an app JWT for an installation access token by calling
    /// `POST /app/installations/{installation_id}/access_tokens`.
    #[instrument(skip(self, jwt), fields(installation_id = installation_id.as_u64()))]
    async fn fetch_installation_token(
        &self,
        installation_id: InstallationId,
        jwt: &JsonWebToken,
    ) -> Result<InstallationToken, AuthError> {
        let url = format!(
            "{}/app/installations/{}/access_tokens",
            self.api_url,
            installation_id.as_u64()
        );

        debug!(
            installation_id = installation_id.as_u64(),
            "Fetching installation access token"
        );

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt.token()))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "merge-warden")
            .send()
            .await
            .map_err(|e| {
                error!(
                    installation_id = installation_id.as_u64(),
                    error = %e,
                    "HTTP request to GitHub failed"
                );
                AuthError::TokenExchangeFailed {
                    installation_id,
                    message: format!("HTTP request failed: {}", e),
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!(
                installation_id = installation_id.as_u64(),
                http_status = status.as_u16(),
                body = body.as_str(),
                "GitHub API denied installation token request"
            );
            return Err(AuthError::TokenExchangeFailed {
                installation_id,
                message: format!("GitHub API returned HTTP {}: {}", status.as_u16(), body),
            });
        }

        let token_response: InstallationAccessTokenResponse =
            response.json().await.map_err(|e| {
                error!(
                    installation_id = installation_id.as_u64(),
                    error = %e,
                    "Failed to parse installation token response"
                );
                AuthError::TokenExchangeFailed {
                    installation_id,
                    message: format!("Failed to parse token response: {}", e),
                }
            })?;

        debug!(
            installation_id = installation_id.as_u64(),
            expires_at = %token_response.expires_at,
            "Installation access token obtained"
        );

        Ok(InstallationToken::new(
            token_response.token,
            installation_id,
            token_response.expires_at,
            InstallationPermissions::default(),
            vec![],
        ))
    }

    /// Resolves the GitHub App installation ID for the given repository.
    ///
    /// Calls `GET /repos/{owner}/{repo}/installation` with a fresh app JWT and
    /// returns the numeric installation ID reported by GitHub.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (user or organisation)
    /// * `repo`  - Repository name
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::TokenExchangeFailed`] when:
    /// - The HTTP request fails (network error)
    /// - GitHub returns a non-2xx status code
    /// - The response JSON is missing the `"id"` field or it is not a `u64`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use merge_warden_developer_platforms::app_auth::AppAuthProvider;
    ///
    /// let auth = AppAuthProvider::new(12345, "-----BEGIN RSA PRIVATE KEY-----\n...", "https://api.github.com")?;
    /// let installation_id = auth.resolve_installation_id("my-org", "my-repo").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(owner, repo, app_id = self.app_id.as_u64()))]
    pub async fn resolve_installation_id(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<InstallationId, AuthError> {
        let jwt = self.generate_app_jwt().await?;

        let url = format!("{}/repos/{}/{}/installation", self.api_url, owner, repo);

        debug!(owner, repo, "Resolving GitHub App installation ID");

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt.token()))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "merge-warden")
            .send()
            .await
            .map_err(|e| {
                error!(
                    owner,
                    repo,
                    error = %e,
                    "HTTP request to resolve installation ID failed"
                );
                AuthError::TokenExchangeFailed {
                    installation_id: InstallationId::new(0),
                    message: format!("HTTP request failed: {}", e),
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!(
                owner,
                repo,
                http_status = status.as_u16(),
                body = body.as_str(),
                "GitHub API returned non-success status resolving installation ID"
            );
            return Err(AuthError::TokenExchangeFailed {
                installation_id: InstallationId::new(0),
                message: format!("GitHub API returned HTTP {}: {}", status.as_u16(), body),
            });
        }

        let body: serde_json::Value = response.json().await.map_err(|e| {
            error!(
                owner,
                repo,
                error = %e,
                "Failed to parse installation response body"
            );
            AuthError::TokenExchangeFailed {
                installation_id: InstallationId::new(0),
                message: format!("Failed to parse response body: {}", e),
            }
        })?;

        let id = body["id"].as_u64().ok_or_else(|| {
            warn!(
                owner,
                repo, "Installation response missing 'id' field or it is not a u64"
            );
            AuthError::TokenExchangeFailed {
                installation_id: InstallationId::new(0),
                message: "Response JSON missing 'id' field or it is not a u64".to_string(),
            }
        })?;

        debug!(
            owner,
            repo,
            installation_id = id,
            "Resolved GitHub App installation ID"
        );
        Ok(InstallationId::new(id))
    }
}

#[async_trait]
impl AuthenticationProvider for AppAuthProvider {
    /// Generates a short-lived RS256 JWT to authenticate as the GitHub App.
    async fn app_token(&self) -> Result<JsonWebToken, AuthError> {
        self.generate_app_jwt().await
    }

    /// Returns a valid installation token for the given installation.
    ///
    /// Returns the cached token if still valid; otherwise exchanges a fresh JWT
    /// for a new installation token and caches it.
    ///
    /// Uses a double-check locking pattern to avoid a thundering-herd: the common
    /// case (valid cached token) is served under a shared read lock, while the slow
    /// path acquires the write lock and re-checks before making the network call,
    /// so at most one concurrent fetch occurs per installation ID.
    async fn installation_token(
        &self,
        installation_id: InstallationId,
    ) -> Result<InstallationToken, AuthError> {
        // Fast path: serve from cache under a shared read lock.
        {
            let cache = self.token_cache.read().await;
            if let Some(token) = cache.get(&installation_id.as_u64()) {
                if !token.is_expired() {
                    debug!(
                        installation_id = installation_id.as_u64(),
                        "Using cached installation token"
                    );
                    return Ok(token.clone());
                }
            }
        }
        // Read lock is dropped here.

        // Slow path: acquire exclusive write lock and re-check before fetching.
        // This serialises concurrent callers so only one network round-trip is made
        // per installation ID when the cached token is missing or expired.
        let mut cache = self.token_cache.write().await;
        if let Some(token) = cache.get(&installation_id.as_u64()) {
            if !token.is_expired() {
                debug!(
                    installation_id = installation_id.as_u64(),
                    "Using installation token refreshed by concurrent task"
                );
                return Ok(token.clone());
            }
            warn!(
                installation_id = installation_id.as_u64(),
                "Cached installation token expired — refreshing"
            );
        }

        let jwt = self.generate_app_jwt().await?;
        let token = self.fetch_installation_token(installation_id, &jwt).await?;
        cache.insert(installation_id.as_u64(), token.clone());
        Ok(token)
    }

    /// Forces a refresh of the installation token, bypassing the cache.
    async fn refresh_installation_token(
        &self,
        installation_id: InstallationId,
    ) -> Result<InstallationToken, AuthError> {
        let jwt = self.generate_app_jwt().await?;
        let token = self.fetch_installation_token(installation_id, &jwt).await?;

        {
            let mut cache = self.token_cache.write().await;
            cache.insert(installation_id.as_u64(), token.clone());
        }

        Ok(token)
    }

    /// Lists all installations for this app.
    ///
    /// Not required for webhook-driven operation; returns an empty list.
    async fn list_installations(&self) -> Result<Vec<Installation>, AuthError> {
        Ok(vec![])
    }

    /// Lists repositories accessible to a specific installation.
    ///
    /// Not required for webhook-driven operation; returns an empty list.
    async fn get_installation_repositories(
        &self,
        _installation_id: InstallationId,
    ) -> Result<Vec<Repository>, AuthError> {
        Ok(vec![])
    }
}

#[cfg(test)]
#[path = "app_auth_tests.rs"]
mod tests;
