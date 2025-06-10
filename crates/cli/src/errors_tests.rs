use super::*;
use anyhow::anyhow;

#[test]
fn test_config_error_display() {
    let err = CliError::ConfigError("bad config".to_string());
    assert_eq!(format!("{}", err), "Configuration error: bad config");
}

#[test]
fn test_auth_error_display() {
    let err = CliError::AuthError("bad auth".to_string());
    assert_eq!(format!("{}", err), "Authentication error: bad auth");
}

#[test]
fn test_network_error_display() {
    let err = CliError::NetworkError("net fail".to_string());
    assert_eq!(format!("{}", err), "Network error: net fail");
}

#[test]
fn test_invalid_arguments_display() {
    let err = CliError::InvalidArguments("bad arg".to_string());
    assert_eq!(format!("{}", err), "Invalid arguments: bad arg");
}

#[test]
fn test_validation_failed_display() {
    let err = CliError::ValidationFailed("fail".to_string());
    assert_eq!(format!("{}", err), "Validation failed: fail");
}

#[test]
fn test_other_error_display() {
    let err = CliError::Other("other".to_string());
    assert_eq!(format!("{}", err), "Error: other");
}

#[test]
fn test_from_anyhow_error() {
    let err: CliError = anyhow!("anyhow error").into();
    assert!(matches!(err, CliError::Other(_)));
}
