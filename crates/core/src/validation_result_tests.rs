use super::*;

#[test]
fn test_validation_result_valid() {
    let result = ValidationResult::valid();

    assert!(result.is_valid);
    assert!(!result.bypass_used);
    assert!(result.bypass_info.is_none());
}

#[test]
fn test_validation_result_invalid() {
    let result = ValidationResult::invalid();

    assert!(!result.is_valid);
    assert!(!result.bypass_used);
    assert!(result.bypass_info.is_none());
}

#[test]
fn test_validation_result_bypassed_title_convention() {
    let bypass_info = BypassInfo {
        rule_type: BypassRuleType::TitleConvention,
        user: "admin".to_string(),
    };

    let result = ValidationResult::bypassed(bypass_info.clone());

    assert!(result.is_valid);
    assert!(result.bypass_used);
    assert!(result.bypass_info.is_some());

    let info = result.bypass_info.unwrap();
    assert_eq!(info.rule_type, BypassRuleType::TitleConvention);
    assert_eq!(info.user, "admin");
}

#[test]
fn test_validation_result_bypassed_work_item_reference() {
    let bypass_info = BypassInfo {
        rule_type: BypassRuleType::WorkItemReference,
        user: "release-bot".to_string(),
    };

    let result = ValidationResult::bypassed(bypass_info.clone());

    assert!(result.is_valid);
    assert!(result.bypass_used);
    assert!(result.bypass_info.is_some());

    let info = result.bypass_info.unwrap();
    assert_eq!(info.rule_type, BypassRuleType::WorkItemReference);
    assert_eq!(info.user, "release-bot");
}

#[test]
fn test_bypass_info_equality() {
    let info1 = BypassInfo {
        rule_type: BypassRuleType::TitleConvention,
        user: "user1".to_string(),
    };

    let info2 = BypassInfo {
        rule_type: BypassRuleType::TitleConvention,
        user: "user1".to_string(),
    };

    let info3 = BypassInfo {
        rule_type: BypassRuleType::WorkItemReference,
        user: "user1".to_string(),
    };

    assert_eq!(info1, info2);
    assert_ne!(info1, info3);
}

#[test]
fn test_bypass_rule_type_display() {
    assert_eq!(
        BypassRuleType::TitleConvention.to_string(),
        "Title Convention"
    );
    assert_eq!(
        BypassRuleType::WorkItemReference.to_string(),
        "Work Item Reference"
    );
}

#[test]
fn test_bypass_rule_type_debug() {
    let rule = BypassRuleType::TitleConvention;
    let debug_output = format!("{:?}", rule);
    assert_eq!(debug_output, "TitleConvention");
}

#[test]
fn test_validation_result_clone() {
    let bypass_info = BypassInfo {
        rule_type: BypassRuleType::TitleConvention,
        user: "test-user".to_string(),
    };

    let result1 = ValidationResult::bypassed(bypass_info);
    let result2 = result1.clone();

    assert_eq!(result1, result2);
    assert!(result2.bypass_used);
    assert_eq!(result2.bypass_info.as_ref().unwrap().user, "test-user");
}

#[test]
fn test_validation_result_serialization() {
    let bypass_info = BypassInfo {
        rule_type: BypassRuleType::TitleConvention,
        user: "serialization-test".to_string(),
    };

    let result = ValidationResult::bypassed(bypass_info);

    // Test serialization to JSON
    let json = serde_json::to_string(&result).expect("Should serialize to JSON");
    assert!(json.contains("serialization-test"));
    assert!(json.contains("TitleConvention"));

    // Test deserialization from JSON
    let deserialized: ValidationResult =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(result, deserialized);
}

#[test]
fn test_validation_result_valid_serialization() {
    let result = ValidationResult::valid();

    let json = serde_json::to_string(&result).expect("Should serialize to JSON");
    let deserialized: ValidationResult =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(result, deserialized);
    assert!(deserialized.is_valid);
    assert!(!deserialized.bypass_used);
}

#[test]
fn test_validation_result_invalid_serialization() {
    let result = ValidationResult::invalid();

    let json = serde_json::to_string(&result).expect("Should serialize to JSON");
    let deserialized: ValidationResult =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(result, deserialized);
    assert!(!deserialized.is_valid);
    assert!(!deserialized.bypass_used);
}

#[test]
fn test_bypass_info_debug_format() {
    let bypass_info = BypassInfo {
        rule_type: BypassRuleType::WorkItemReference,
        user: "debug-test".to_string(),
    };

    let debug_output = format!("{:?}", bypass_info);
    assert!(debug_output.contains("WorkItemReference"));
    assert!(debug_output.contains("debug-test"));
}

#[test]
fn test_validation_result_pattern_matching() {
    let valid_result = ValidationResult::valid();
    let invalid_result = ValidationResult::invalid();

    let bypass_info = BypassInfo {
        rule_type: BypassRuleType::TitleConvention,
        user: "pattern-test".to_string(),
    };
    let bypassed_result = ValidationResult::bypassed(bypass_info);

    // Test pattern matching on different result types
    match valid_result.bypass_used {
        true => panic!("Valid result should not have bypass"),
        false => (), // Expected
    }

    match invalid_result.is_valid {
        true => panic!("Invalid result should not be valid"),
        false => (), // Expected
    }

    match bypassed_result.bypass_info {
        Some(info) => assert_eq!(info.user, "pattern-test"),
        None => panic!("Bypassed result should have bypass info"),
    }
}

// ============================================================================
// ADR-006: ValidationResult::config_error
//
// See docs/spec/interfaces/core-config-validation.md
// #check-time-behaviour-on-patternvalidationerror
// ============================================================================

#[test]
fn test_validation_result_config_error_is_not_valid() {
    let result = ValidationResult::config_error("pattern failed to compile".to_string());
    assert!(
        !result.is_valid,
        "a configuration-error result must never be valid"
    );
}

#[test]
fn test_validation_result_config_error_is_not_a_bypass() {
    let result = ValidationResult::config_error("pattern failed to compile".to_string());
    assert!(
        !result.bypass_used,
        "a configuration-error result must not be reported as a bypass"
    );
    assert!(result.bypass_info.is_none());
}

#[test]
fn test_validation_result_config_error_carries_the_message_verbatim() {
    let result = ValidationResult::config_error("the configured pattern is invalid".to_string());
    assert_eq!(
        result.config_error,
        Some("the configured pattern is invalid".to_string())
    );
}

#[test]
fn test_validation_result_config_error_is_distinguishable_from_invalid() {
    // [security] A compile failure must never be conflated with an ordinary
    // "content did not match" failure: the two must be distinguishable at
    // the result-type level, not merely by message text.
    let config_error = ValidationResult::config_error("broken pattern".to_string());
    let plain_invalid = ValidationResult::invalid();

    assert_eq!(config_error.is_valid, plain_invalid.is_valid); // both false
    assert!(
        config_error.config_error.is_some(),
        "config_error must be Some for a configuration failure"
    );
    assert!(
        plain_invalid.config_error.is_none(),
        "config_error must be None for an ordinary content-mismatch failure"
    );
    assert_ne!(config_error, plain_invalid);
}

#[test]
fn test_validation_result_valid_has_no_config_error() {
    assert!(ValidationResult::valid().config_error.is_none());
}

#[test]
fn test_validation_result_invalid_has_no_config_error() {
    assert!(ValidationResult::invalid().config_error.is_none());
}

#[test]
fn test_validation_result_bypassed_has_no_config_error() {
    let bypass_info = BypassInfo {
        rule_type: BypassRuleType::TitleConvention,
        user: "release-bot".to_string(),
    };
    assert!(ValidationResult::bypassed(bypass_info)
        .config_error
        .is_none());
}

#[test]
fn test_validation_result_config_error_serialization_round_trips() {
    let result = ValidationResult::config_error("pattern too long".to_string());
    let json = serde_json::to_string(&result).expect("should serialize to JSON");
    let deserialized: ValidationResult =
        serde_json::from_str(&json).expect("should deserialize from JSON");
    assert_eq!(result, deserialized);
    assert_eq!(
        deserialized.config_error,
        Some("pattern too long".to_string())
    );
}

#[test]
fn test_validation_result_config_error_clone() {
    let result = ValidationResult::config_error("broken".to_string());
    let cloned = result.clone();
    assert_eq!(result, cloned);
}
