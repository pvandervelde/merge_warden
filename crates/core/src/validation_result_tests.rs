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
