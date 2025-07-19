//! Azure App Configuration REST API client implementation
//!
//! This module provides a custom REST client for Azure App Configuration that uses
//! managed identity for authentication and implements caching with TTL for performance.
//!
//! The client avoids dependencies on potentially outdated third-party Azure App Configuration
//! libraries by directly interfacing with the REST API.

use azure_core::credentials::TokenCredential;
use azure_identity::{ManagedIdentityCredential, ManagedIdentityCredentialOptions};
use merge_warden_core::config::{
    ApplicationDefaults, BypassRule, BypassRules, ChangeTypeLabelConfig,
    ConventionalCommitMappings, FallbackLabelSettings, LabelDetectionStrategy, PrSizeCheckConfig,
};
use merge_warden_core::size::SizeThresholds;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument};

#[cfg(test)]
#[path = "app_config_client_tests.rs"]
mod tests;

/// Errors that can occur when interacting with Azure App Configuration
#[derive(Error, Debug)]
pub enum AppConfigError {
    /// Authentication with Azure App Configuration failed
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// HTTP request to Azure App Configuration failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Azure App Configuration API returned an error response
    #[error("Azure App Configuration API error: {status} - {message}")]
    ApiError {
        /// HTTP status code returned by the API
        status: StatusCode,
        /// Error message from the API response
        message: String,
    },

    /// Failed to parse a configuration value
    #[error("Failed to parse configuration value: {key} - {error}")]
    ParseError {
        /// The configuration key that failed to parse
        key: String,
        /// The parsing error details
        error: String,
    },

    /// Invalid Azure App Configuration endpoint URL
    #[error("Invalid endpoint URL: {0}")]
    InvalidEndpoint(String),

    /// Configuration key was not found
    #[error("Configuration key not found: {0}")]
    KeyNotFound(String),
}

/// Represents a key-value pair from Azure App Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    /// The configuration key name
    pub key: String,
    /// The configuration value as a string
    pub value: String,
    /// Optional content type for the value
    pub content_type: Option<String>,
    /// Optional ETag for version control
    pub etag: Option<String>,
    /// Optional label for environment-specific configuration
    pub label: Option<String>,
}

/// Response from Azure App Configuration REST API for multiple key-value pairs
#[derive(Debug, Deserialize)]
struct ConfigListResponse {
    /// Array of configuration key-value pairs
    items: Vec<ConfigValue>,
}

/// Cache entry with TTL
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached configuration value
    value: ConfigValue,
    /// Expiration timestamp for cache invalidation
    expires_at: Instant,
}

/// Cache status information for monitoring and debugging
#[derive(Debug, Clone)]
pub struct CacheStatus {
    /// Total number of cached entries
    pub total_entries: usize,
    /// Number of expired cache entries
    pub expired_entries: usize,
    /// Number of cache hits
    pub hit_count: u64,
    /// Number of cache misses
    pub miss_count: u64,
}

/// Azure App Configuration REST client with caching support
pub struct AppConfigClient {
    /// Azure App Configuration endpoint URL
    endpoint: String,
    /// Managed identity credential for authentication
    credential: Arc<ManagedIdentityCredential>,
    /// HTTP client for making REST API calls
    http_client: Client,
    /// In-memory cache for configuration values
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Time-to-live for cached entries
    cache_ttl: Duration,
    /// Cache hit/miss statistics (hits, misses)
    cache_stats: Arc<RwLock<(u64, u64)>>,
}

impl AppConfigClient {
    /// Creates a new Azure App Configuration client
    ///
    /// # Arguments
    /// * `endpoint` - The Azure App Configuration endpoint URL (e.g., "https://myconfig.azconfig.io")
    /// * `cache_ttl` - Time-to-live for cached values
    ///
    /// # Returns
    /// A configured `AppConfigClient` instance
    ///
    /// # Errors
    /// Returns `AppConfigError::Authentication` if the managed identity credential cannot be created
    /// Returns `AppConfigError::InvalidEndpoint` if the endpoint URL is invalid
    pub fn new(endpoint: &str, cache_ttl: Duration) -> Result<Self, AppConfigError> {
        // Validate endpoint URL
        if !endpoint.starts_with("https://") || !endpoint.contains(".azconfig.io") {
            return Err(AppConfigError::InvalidEndpoint(
                format!("Expected Azure App Configuration endpoint like 'https://name.azconfig.io', got: {}", endpoint)
            ));
        }

        let credential =
            ManagedIdentityCredential::new(Some(ManagedIdentityCredentialOptions::default()))
                .map_err(|e| AppConfigError::Authentication(e.to_string()))?;

        let http_client = Client::new();

        Ok(Self {
            endpoint: endpoint.to_string(),
            credential,
            http_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            cache_stats: Arc::new(RwLock::new((0, 0))),
        })
    }

    /// Retrieves a single configuration value by key
    ///
    /// # Arguments
    /// * `key` - The configuration key to retrieve
    /// * `label` - Optional label filter
    ///
    /// # Returns
    /// The configuration value if found
    ///
    /// # Errors
    /// Returns `AppConfigError::KeyNotFound` if the key doesn't exist
    /// Returns other `AppConfigError` variants for network or authentication issues
    #[instrument(skip(self), fields(endpoint = %self.endpoint))]
    pub async fn get_value(
        &self,
        key: &str,
        label: Option<&str>,
    ) -> Result<ConfigValue, AppConfigError> {
        let cache_key = format!("{}:{}", key, label.unwrap_or(""));

        // Check cache first
        if let Some(cached) = self.get_from_cache(&cache_key).await {
            debug!(key = key, "Configuration value retrieved from cache");
            return Ok(cached);
        }

        // Fetch from Azure App Configuration
        let config_value = self.fetch_single_value(key, label).await?;

        // Cache the result
        self.cache_value(&cache_key, &config_value).await;

        info!(
            key = key,
            "Configuration value retrieved from Azure App Configuration"
        );
        Ok(config_value)
    }

    /// Retrieves multiple configuration values by key prefix
    ///
    /// # Arguments
    /// * `key_prefix` - The key prefix to filter by (e.g., "app:" to get all keys starting with "app:")
    /// * `label` - Optional label filter
    ///
    /// # Returns
    /// A vector of configuration values matching the prefix
    #[instrument(skip(self), fields(endpoint = %self.endpoint))]
    pub async fn get_values_by_prefix(
        &self,
        key_prefix: &str,
        label: Option<&str>,
    ) -> Result<Vec<ConfigValue>, AppConfigError> {
        debug!(
            prefix = key_prefix,
            "Fetching configuration values by prefix"
        );

        let token = self.get_access_token().await?;

        let mut url = format!("{}/kv?key={}*", self.endpoint, key_prefix);
        if let Some(label) = label {
            url.push_str(&format!("&label={}", label));
        }

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token.token.secret()))
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppConfigError::ApiError {
                status,
                message: error_text,
            });
        }

        let config_response: ConfigListResponse = response.json().await?;

        // Cache individual values
        for value in &config_response.items {
            let cache_key = format!("{}:{}", value.key, value.label.as_deref().unwrap_or(""));
            self.cache_value(&cache_key, value).await;
        }

        info!(
            prefix = key_prefix,
            count = config_response.items.len(),
            "Configuration values retrieved by prefix"
        );

        Ok(config_response.items)
    }

    /// Loads configuration from Azure App Configuration and maps it to ApplicationDefaults
    ///
    /// This method retrieves all relevant configuration keys and handles missing values
    /// by falling back to hardcoded defaults.
    ///
    /// # Returns
    /// An `ApplicationDefaults` struct populated with values from Azure App Configuration
    #[instrument(skip(self), fields(endpoint = %self.endpoint))]
    pub async fn load_application_defaults(&self) -> Result<ApplicationDefaults, AppConfigError> {
        info!("Loading application configuration from Azure App Configuration");

        // Load all relevant configuration keys
        let bypass_values = self
            .get_values_by_prefix("bypass_rules:", None)
            .await
            .unwrap_or_default();
        let app_values = self
            .get_values_by_prefix("application:", None)
            .await
            .unwrap_or_default();
        let pr_size_values = self
            .get_values_by_prefix("pr_size:", None)
            .await
            .unwrap_or_default();
        let change_type_values = self
            .get_values_by_prefix("change_type_labels:", None)
            .await
            .unwrap_or_default();

        // Convert to map for easier lookup
        let mut config_map = HashMap::new();
        for value in bypass_values
            .into_iter()
            .chain(app_values.into_iter())
            .chain(pr_size_values.into_iter())
            .chain(change_type_values.into_iter())
        {
            config_map.insert(value.key.clone(), value);
        }

        // Parse bypass rules
        let bypass_rules = self.parse_bypass_rules(&config_map)?;

        // Parse application settings with fallbacks to hardcoded defaults
        let enable_title_validation = self
            .parse_bool_value(&config_map, "application:enforce_title_convention")
            .unwrap_or(false);

        let enable_work_item_validation = self
            .parse_bool_value(&config_map, "application:require_work_items")
            .unwrap_or(false);

        // Parse PR size configuration
        let pr_size_check = self.parse_pr_size_config(&config_map)?;

        // Parse change type labels configuration
        let change_type_labels = self.parse_change_type_labels_config(&config_map)?;

        // For patterns and labels, we fall back to the ApplicationDefaults::default() values
        // if they're not present in App Configuration yet
        let defaults = ApplicationDefaults::default();

        info!(
            enable_title_validation = enable_title_validation,
            enable_work_item_validation = enable_work_item_validation,
            enable_pr_size_checking = pr_size_check.enabled,
            enable_change_type_labels = change_type_labels.enabled,
            "Application configuration loaded successfully"
        );

        let result = ApplicationDefaults {
            enable_title_validation,
            default_title_pattern: defaults.default_title_pattern,
            default_invalid_title_label: defaults.default_invalid_title_label,
            enable_work_item_validation,
            default_work_item_pattern: defaults.default_work_item_pattern,
            default_missing_work_item_label: defaults.default_missing_work_item_label,
            bypass_rules,
            pr_size_check: pr_size_check.clone(),
            change_type_labels,
        };

        Ok(result)
    }

    /// Gets the current cache status for monitoring
    pub async fn get_cache_status(&self) -> CacheStatus {
        let cache = self.cache.read().await;
        let stats = self.cache_stats.read().await;
        let now = Instant::now();

        let expired_count = cache
            .values()
            .filter(|entry| entry.expires_at <= now)
            .count();

        CacheStatus {
            total_entries: cache.len(),
            expired_entries: expired_count,
            hit_count: stats.0,
            miss_count: stats.1,
        }
    }

    /// Clears expired entries from the cache
    pub async fn cleanup_cache(&self) {
        let mut cache = self.cache.write().await;
        let now = Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);
        debug!("Cache cleanup completed");
    }

    /// Helper method to fetch a single value from Azure App Configuration
    async fn fetch_single_value(
        &self,
        key: &str,
        label: Option<&str>,
    ) -> Result<ConfigValue, AppConfigError> {
        let token = self.get_access_token().await?;

        let mut url = format!("{}/kv/{}", self.endpoint, key);
        if let Some(label) = label {
            url.push_str(&format!("?label={}", label));
        }

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token.token.secret()))
            .header("Accept", "application/json")
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let config_value: ConfigValue = response.json().await?;
                Ok(config_value)
            }
            StatusCode::NOT_FOUND => Err(AppConfigError::KeyNotFound(key.to_string())),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(AppConfigError::ApiError {
                    status,
                    message: error_text,
                })
            }
        }
    }

    /// Gets an access token using managed identity
    async fn get_access_token(
        &self,
    ) -> Result<azure_core::credentials::AccessToken, AppConfigError> {
        self.credential
            .get_token(&["https://azconfig.io/.default"])
            .await
            .map_err(|e| AppConfigError::Authentication(e.to_string()))
    }

    /// Retrieves a value from cache if it exists and hasn't expired
    async fn get_from_cache(&self, cache_key: &str) -> Option<ConfigValue> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(cache_key) {
            if entry.expires_at > Instant::now() {
                // Cache hit
                let mut stats = self.cache_stats.write().await;
                stats.0 += 1;
                return Some(entry.value.clone());
            }
        }

        // Cache miss
        let mut stats = self.cache_stats.write().await;
        stats.1 += 1;
        None
    }

    /// Caches a configuration value with TTL
    async fn cache_value(&self, cache_key: &str, value: &ConfigValue) {
        let mut cache = self.cache.write().await;
        cache.insert(
            cache_key.to_string(),
            CacheEntry {
                value: value.clone(),
                expires_at: Instant::now() + self.cache_ttl,
            },
        );
    }

    /// Parses PR size configuration from the configuration map
    fn parse_pr_size_config(
        &self,
        config_map: &HashMap<String, ConfigValue>,
    ) -> Result<PrSizeCheckConfig, AppConfigError> {
        let enabled = self
            .parse_bool_value(config_map, "pr_size:enabled")
            .unwrap_or(false);

        let fail_on_oversized = self
            .parse_bool_value(config_map, "pr_size:fail_on_oversized")
            .unwrap_or(false);

        let label_prefix = config_map
            .get("pr_size:label_prefix")
            .map(|v| v.value.clone())
            .unwrap_or_else(|| "size/".to_string());

        let add_comment = self
            .parse_bool_value(config_map, "pr_size:add_comment")
            .unwrap_or(true);

        let excluded_file_patterns = self
            .parse_json_array_value(config_map, "pr_size:excluded_file_patterns")
            .unwrap_or_default();

        // Parse size thresholds if provided
        let thresholds = self.parse_size_thresholds(config_map);

        Ok(PrSizeCheckConfig {
            enabled,
            thresholds,
            fail_on_oversized,
            excluded_file_patterns,
            label_prefix,
            add_comment,
        })
    }

    /// Parses size thresholds from the configuration map
    fn parse_size_thresholds(
        &self,
        config_map: &HashMap<String, ConfigValue>,
    ) -> Option<SizeThresholds> {
        let xs = self.parse_u32_value(config_map, "pr_size:thresholds:xs");
        let s = self.parse_u32_value(config_map, "pr_size:thresholds:small");
        let m = self.parse_u32_value(config_map, "pr_size:thresholds:medium");
        let l = self.parse_u32_value(config_map, "pr_size:thresholds:large");
        let xl = self.parse_u32_value(config_map, "pr_size:thresholds:extra_large");

        // Only create SizeThresholds if at least one threshold is provided
        if xs.is_some() || s.is_some() || m.is_some() || l.is_some() || xl.is_some() {
            let default_thresholds = SizeThresholds::default();
            Some(SizeThresholds {
                xs: xs.unwrap_or(default_thresholds.xs),
                s: s.unwrap_or(default_thresholds.s),
                m: m.unwrap_or(default_thresholds.m),
                l: l.unwrap_or(default_thresholds.l),
                xl: xl.unwrap_or(default_thresholds.xl),
            })
        } else {
            None
        }
    }

    /// Parses a u32 value from the configuration map
    fn parse_u32_value(&self, config_map: &HashMap<String, ConfigValue>, key: &str) -> Option<u32> {
        config_map.get(key)?.value.parse().ok()
    }

    /// Parses bypass rules from the configuration map
    fn parse_bypass_rules(
        &self,
        config_map: &HashMap<String, ConfigValue>,
    ) -> Result<BypassRules, AppConfigError> {
        let title_enabled = self
            .parse_bool_value(config_map, "bypass_rules:title:enabled")
            .unwrap_or(false);

        let title_users = self
            .parse_json_array_value(config_map, "bypass_rules:title:users")
            .unwrap_or_default();

        let work_item_enabled = self
            .parse_bool_value(config_map, "bypass_rules:work_item:enabled")
            .unwrap_or(false);

        let work_item_users = self
            .parse_json_array_value(config_map, "bypass_rules:work_item:users")
            .unwrap_or_default();

        Ok(BypassRules::new(
            BypassRule::new(title_enabled, title_users),
            BypassRule::new(work_item_enabled, work_item_users),
        ))
    }

    /// Parses a boolean value from the configuration map
    fn parse_bool_value(
        &self,
        config_map: &HashMap<String, ConfigValue>,
        key: &str,
    ) -> Option<bool> {
        config_map.get(key)?.value.parse().ok()
    }

    /// Parses change type labels configuration from the configuration map
    fn parse_change_type_labels_config(
        &self,
        config_map: &HashMap<String, ConfigValue>,
    ) -> Result<ChangeTypeLabelConfig, AppConfigError> {
        let enabled = self
            .parse_bool_value(config_map, "change_type_labels:enabled")
            .unwrap_or(true);

        // Parse conventional commit mappings
        let conventional_commit_mappings = ConventionalCommitMappings {
            feat: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:feat")
                .unwrap_or_else(|| {
                    vec![
                        "enhancement".to_string(),
                        "feature".to_string(),
                        "new feature".to_string(),
                    ]
                }),
            fix: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:fix")
                .unwrap_or_else(|| {
                    vec!["bug".to_string(), "bugfix".to_string(), "fix".to_string()]
                }),
            docs: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:docs")
                .unwrap_or_else(|| vec!["documentation".to_string(), "docs".to_string()]),
            style: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:style")
                .unwrap_or_else(|| vec!["style".to_string(), "formatting".to_string()]),
            refactor: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:refactor")
                .unwrap_or_else(|| {
                    vec![
                        "refactor".to_string(),
                        "refactoring".to_string(),
                        "code quality".to_string(),
                    ]
                }),
            perf: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:perf")
                .unwrap_or_else(|| vec!["performance".to_string(), "optimization".to_string()]),
            test: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:test")
                .unwrap_or_else(|| {
                    vec![
                        "test".to_string(),
                        "tests".to_string(),
                        "testing".to_string(),
                    ]
                }),
            chore: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:chore")
                .unwrap_or_else(|| {
                    vec![
                        "chore".to_string(),
                        "maintenance".to_string(),
                        "housekeeping".to_string(),
                    ]
                }),
            ci: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:ci")
                .unwrap_or_else(|| {
                    vec![
                        "ci".to_string(),
                        "continuous integration".to_string(),
                        "build".to_string(),
                    ]
                }),
            build: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:build")
                .unwrap_or_else(|| vec!["build".to_string(), "dependencies".to_string()]),
            revert: self
                .parse_json_array_value(config_map, "change_type_labels:mappings:revert")
                .unwrap_or_else(|| vec!["revert".to_string()]),
        };

        // Parse fallback label settings
        let name_format = config_map
            .get("change_type_labels:fallback:name_format")
            .map(|v| v.value.clone())
            .unwrap_or_else(|| "type: {change_type}".to_string());

        let create_if_missing = self
            .parse_bool_value(config_map, "change_type_labels:fallback:create_if_missing")
            .unwrap_or(true);

        // Parse color scheme with defaults
        let mut color_scheme = HashMap::new();
        let default_colors = [
            ("feat", "#0075ca"),
            ("fix", "#d73a4a"),
            ("docs", "#0052cc"),
            ("style", "#f9d0c4"),
            ("refactor", "#fef2c0"),
            ("perf", "#a2eeef"),
            ("test", "#d4edda"),
            ("chore", "#e1e4e8"),
            ("ci", "#fbca04"),
            ("build", "#c5def5"),
            ("revert", "#b60205"),
        ];

        for (commit_type, default_color) in default_colors {
            let color = config_map
                .get(&format!("change_type_labels:colors:{}", commit_type))
                .map(|v| v.value.clone())
                .unwrap_or_else(|| default_color.to_string());
            color_scheme.insert(commit_type.to_string(), color);
        }

        let fallback_label_settings = FallbackLabelSettings {
            name_format,
            color_scheme,
            create_if_missing,
        };

        // Parse detection strategy
        let exact_match = self
            .parse_bool_value(config_map, "change_type_labels:detection:exact_match")
            .unwrap_or(true);

        let prefix_match = self
            .parse_bool_value(config_map, "change_type_labels:detection:prefix_match")
            .unwrap_or(true);

        let description_match = self
            .parse_bool_value(config_map, "change_type_labels:detection:description_match")
            .unwrap_or(true);

        let common_prefixes = self
            .parse_json_array_value(config_map, "change_type_labels:detection:common_prefixes")
            .unwrap_or_else(|| {
                vec![
                    "type:".to_string(),
                    "kind:".to_string(),
                    "category:".to_string(),
                ]
            });

        let detection_strategy = LabelDetectionStrategy {
            exact_match,
            prefix_match,
            description_match,
            common_prefixes,
        };

        Ok(ChangeTypeLabelConfig {
            enabled,
            conventional_commit_mappings,
            fallback_label_settings,
            detection_strategy,
        })
    }

    /// Parses a JSON array value from the configuration map
    fn parse_json_array_value(
        &self,
        config_map: &HashMap<String, ConfigValue>,
        key: &str,
    ) -> Option<Vec<String>> {
        let config_value = config_map.get(key)?;
        if config_value.content_type.as_deref() == Some("application/json") {
            serde_json::from_str(&config_value.value).ok()
        } else {
            None
        }
    }
}

impl std::fmt::Display for CacheStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Status: {} total entries, {} expired, {} hits, {} misses",
            self.total_entries, self.expired_entries, self.hit_count, self.miss_count
        )
    }
}
