use super::AzureFunctionsError;

#[test]
fn test_config_error_display() {
    let error = AzureFunctionsError::ConfigError("Invalid configuration".to_string());
    assert_eq!(
        format!("{}", error),
        "Configuration error: Invalid configuration"
    );
}

#[test]
fn test_auth_error_display() {
    let error = AzureFunctionsError::AuthError("Authentication failed".to_string());
    assert_eq!(
        format!("{}", error),
        "Authentication error: Authentication failed"
    );
}

#[test]
fn test_network_error_display() {
    let error = AzureFunctionsError::NetworkError("Network unreachable".to_string());
    assert_eq!(format!("{}", error), "Network error: Network unreachable");
}

#[test]
fn test_invalid_arguments_display() {
    let error = AzureFunctionsError::InvalidArguments("Missing argument".to_string());
    assert_eq!(format!("{}", error), "Invalid arguments: Missing argument");
}

#[test]
fn test_validation_failed_display() {
    let error = AzureFunctionsError::ValidationFailed("Validation error".to_string());
    assert_eq!(format!("{}", error), "Validation failed: Validation error");
}

#[test]
fn test_other_error_display() {
    let error = AzureFunctionsError::Other("Unknown error".to_string());
    assert_eq!(format!("{}", error), "Error: Unknown error");
}
