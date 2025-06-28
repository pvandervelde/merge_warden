//! Tests for the Azure App Configuration client

use super::*;
use std::time::Duration;

#[test]
fn test_invalid_endpoint_validation() {
    // Test invalid endpoints
    let invalid_endpoints = vec![
        "http://unsecure.azconfig.io", // Not HTTPS
        "https://not-azconfig.com",    // Wrong domain
        "invalid-url",                 // Not a URL
        "",                            // Empty
    ];

    for endpoint in invalid_endpoints {
        let result = AppConfigClient::new(endpoint, Duration::from_secs(300));
        assert!(
            matches!(result, Err(AppConfigError::InvalidEndpoint(_))),
            "Expected InvalidEndpoint error for: {}",
            endpoint
        );
    }
}

#[test]
fn test_valid_endpoint_validation() {
    // Note: This test creates a client but doesn't make network calls
    // since we don't have real Azure credentials in tests
    let valid_endpoints = vec![
        "https://myapp.azconfig.io",
        "https://test-config.azconfig.io",
        "https://prod-merge-warden.azconfig.io",
    ];

    for endpoint in valid_endpoints {
        // Just test that the client can be created without endpoint validation errors
        // The actual token acquisition will fail in tests, but that's expected
        let client_creation = AppConfigClient::new(endpoint, Duration::from_secs(300));

        // We expect success since the endpoint format is valid
        assert!(
            client_creation.is_ok(),
            "Expected successful client creation for valid endpoint: {}",
            endpoint
        );
    }
}

#[tokio::test]
async fn test_cache_status_empty() {
    let client =
        AppConfigClient::new("https://test.azconfig.io", Duration::from_secs(300)).unwrap();

    let status = client.get_cache_status().await;
    assert_eq!(status.total_entries, 0);
    assert_eq!(status.expired_entries, 0);
    assert_eq!(status.hit_count, 0);
    assert_eq!(status.miss_count, 0);
}

#[test]
fn test_config_value_serialization() {
    let item = ConfigValue {
        key: "test:key".to_string(),
        value: "test-value".to_string(),
        content_type: Some("text/plain".to_string()),
        etag: Some("abc123".to_string()),
        label: None,
    };

    // Test that we can serialize and deserialize
    let json = serde_json::to_string(&item).unwrap();
    let deserialized: ConfigValue = serde_json::from_str(&json).unwrap();

    assert_eq!(item.key, deserialized.key);
    assert_eq!(item.value, deserialized.value);
    assert_eq!(item.content_type, deserialized.content_type);
    assert_eq!(item.etag, deserialized.etag);
    assert_eq!(item.label, deserialized.label);
}

#[test]
fn test_app_config_error_display() {
    let errors = vec![
        AppConfigError::KeyNotFound("missing-key".to_string()),
        AppConfigError::InvalidEndpoint("bad-url".to_string()),
        AppConfigError::Authentication("auth failed".to_string()),
        AppConfigError::ApiError {
            status: reqwest::StatusCode::BAD_REQUEST,
            message: "Bad request".to_string(),
        },
        AppConfigError::ParseError {
            key: "test-key".to_string(),
            error: "parse failed".to_string(),
        },
    ];

    for error in errors {
        let error_string = error.to_string();
        assert!(
            !error_string.is_empty(),
            "Error message should not be empty"
        );
        println!("Error: {}", error_string); // For manual verification
    }
}

#[test]
fn test_cache_status_display() {
    let status = CacheStatus {
        total_entries: 10,
        expired_entries: 2,
        hit_count: 50,
        miss_count: 5,
    };

    let display = status.to_string();
    assert!(display.contains("10 total entries"));
    assert!(display.contains("2 expired"));
    assert!(display.contains("50 hits"));
    assert!(display.contains("5 misses"));
}

#[test]
fn test_bypass_rules_parsing() {
    let client =
        AppConfigClient::new("https://test.azconfig.io", Duration::from_secs(300)).unwrap();

    // Create a mock configuration map
    let mut config_map = HashMap::new();

    // Add bypass rules configuration
    config_map.insert(
        "bypass_rules:title:enabled".to_string(),
        ConfigValue {
            key: "bypass_rules:title:enabled".to_string(),
            value: "true".to_string(),
            content_type: None,
            etag: None,
            label: None,
        },
    );

    config_map.insert(
        "bypass_rules:title:users".to_string(),
        ConfigValue {
            key: "bypass_rules:title:users".to_string(),
            value: r#"["admin", "release-bot"]"#.to_string(),
            content_type: Some("application/json".to_string()),
            etag: None,
            label: None,
        },
    );

    config_map.insert(
        "bypass_rules:work_item:enabled".to_string(),
        ConfigValue {
            key: "bypass_rules:work_item:enabled".to_string(),
            value: "false".to_string(),
            content_type: None,
            etag: None,
            label: None,
        },
    );

    config_map.insert(
        "bypass_rules:work_item:users".to_string(),
        ConfigValue {
            key: "bypass_rules:work_item:users".to_string(),
            value: "[]".to_string(),
            content_type: Some("application/json".to_string()),
            etag: None,
            label: None,
        },
    );

    // Test parsing
    let result = client.parse_bypass_rules(&config_map).unwrap();

    // Verify title bypass rule
    assert!(result.title_convention().enabled());
    assert_eq!(
        result.title_convention().users(),
        vec!["admin", "release-bot"]
    );

    // Verify work item bypass rule
    assert!(!result.work_item_convention().enabled());
    assert!(result.work_item_convention().users().is_empty());
}
