//! # Validation Result Types
//!
//! This module provides enhanced result types for validation operations that include
//! bypass information for audit trails and user communication.
//!
//! The primary type is [`ValidationResult`] which replaces simple boolean returns
//! with detailed information about validation outcomes and any bypasses that were used.

use serde::{Deserialize, Serialize};

/// Result of a validation check including bypass information
///
/// This type provides comprehensive information about validation outcomes,
/// including whether the validation passed due to actually valid content
/// or because a bypass rule was applied.
///
/// # Examples
///
/// ## Valid content without bypass
/// ```
/// use merge_warden_core::validation_result::ValidationResult;
///
/// let result = ValidationResult::valid();
/// assert!(result.is_valid);
/// assert!(!result.bypass_used);
/// assert!(result.bypass_info.is_none());
/// ```
///
/// ## Invalid content without bypass
/// ```
/// use merge_warden_core::validation_result::ValidationResult;
///
/// let result = ValidationResult::invalid();
/// assert!(!result.is_valid);
/// assert!(!result.bypass_used);
/// assert!(result.bypass_info.is_none());
/// ```
///
/// ## Valid due to bypass
/// ```
/// use merge_warden_core::validation_result::{ValidationResult, BypassInfo, BypassRuleType};
///
/// let bypass_info = BypassInfo {
///     rule_type: BypassRuleType::TitleConvention,
///     user: "release-bot".to_string(),
/// };
///
/// let result = ValidationResult::bypassed(bypass_info);
/// assert!(result.is_valid);
/// assert!(result.bypass_used);
/// assert!(result.bypass_info.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the validation passed (either valid content or bypassed)
    pub is_valid: bool,

    /// Whether a bypass rule was used to make this validation pass
    pub bypass_used: bool,

    /// Detailed information about the bypass, if used
    pub bypass_info: Option<BypassInfo>,
}

/// Information about a bypass that was used during validation
///
/// This type provides audit trail information when a bypass rule is applied,
/// including which rule was bypassed and which user had the bypass permission.
///
/// # Examples
///
/// ```
/// use merge_warden_core::validation_result::{BypassInfo, BypassRuleType};
///
/// let bypass_info = BypassInfo {
///     rule_type: BypassRuleType::TitleConvention,
///     user: "emergency-deploy".to_string(),
/// };
///
/// println!("User {} bypassed {:?}", bypass_info.user, bypass_info.rule_type);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BypassInfo {
    /// The type of validation rule that was bypassed
    pub rule_type: BypassRuleType,

    /// The username of the user who had bypass permissions
    pub user: String,
}

/// Types of validation rules that can be bypassed
///
/// This enum identifies which specific validation rule was bypassed,
/// enabling detailed audit logging and rule-specific handling.
///
/// # Examples
///
/// ```
/// use merge_warden_core::validation_result::BypassRuleType;
///
/// let rule = BypassRuleType::TitleConvention;
/// match rule {
///     BypassRuleType::TitleConvention => println!("Title validation was bypassed"),
///     BypassRuleType::WorkItemReference => println!("Work item validation was bypassed"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BypassRuleType {
    /// Conventional commit title format validation was bypassed
    TitleConvention,

    /// Work item reference validation was bypassed
    WorkItemReference,
}

impl ValidationResult {
    /// Creates a validation result indicating the content is valid without any bypass
    ///
    /// Use this when validation passes due to actually valid content.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::validation_result::ValidationResult;
    ///
    /// let result = ValidationResult::valid();
    /// assert!(result.is_valid);
    /// assert!(!result.bypass_used);
    /// ```
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            bypass_used: false,
            bypass_info: None,
        }
    }

    /// Creates a validation result indicating the content is invalid
    ///
    /// Use this when validation fails and no bypass rule applies.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::validation_result::ValidationResult;
    ///
    /// let result = ValidationResult::invalid();
    /// assert!(!result.is_valid);
    /// assert!(!result.bypass_used);
    /// ```
    pub fn invalid() -> Self {
        Self {
            is_valid: false,
            bypass_used: false,
            bypass_info: None,
        }
    }

    /// Creates a validation result indicating validation was bypassed
    ///
    /// Use this when validation would normally fail but a bypass rule allows it to pass.
    /// This provides full audit trail information about the bypass.
    ///
    /// # Arguments
    ///
    /// * `bypass_info` - Details about which rule was bypassed and by which user
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::validation_result::{ValidationResult, BypassInfo, BypassRuleType};
    ///
    /// let bypass_info = BypassInfo {
    ///     rule_type: BypassRuleType::TitleConvention,
    ///     user: "admin".to_string(),
    /// };
    ///
    /// let result = ValidationResult::bypassed(bypass_info);
    /// assert!(result.is_valid);
    /// assert!(result.bypass_used);
    /// assert_eq!(result.bypass_info.as_ref().unwrap().user, "admin");
    /// ```
    pub fn bypassed(bypass_info: BypassInfo) -> Self {
        Self {
            is_valid: true,
            bypass_used: true,
            bypass_info: Some(bypass_info),
        }
    }

    /// Convenience method to check if the validation result is valid
    ///
    /// Returns true if the validation passed (either through valid content or bypass).
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::validation_result::ValidationResult;
    ///
    /// let valid = ValidationResult::valid();
    /// assert!(valid.is_valid());
    ///
    /// let invalid = ValidationResult::invalid();
    /// assert!(!invalid.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Convenience method to check if a bypass was used
    ///
    /// Returns true if validation passed due to a bypass rule, false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::validation_result::{ValidationResult, BypassInfo, BypassRuleType};
    ///
    /// let bypass_info = BypassInfo {
    ///     rule_type: BypassRuleType::TitleConvention,
    ///     user: "admin".to_string(),
    /// };
    ///
    /// let bypassed = ValidationResult::bypassed(bypass_info);
    /// assert!(bypassed.was_bypassed());
    ///
    /// let valid = ValidationResult::valid();
    /// assert!(!valid.was_bypassed());
    /// ```
    pub fn was_bypassed(&self) -> bool {
        self.bypass_used
    }

    /// Convenience method to get bypass information
    ///
    /// Returns the bypass information if a bypass was used, None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::validation_result::{ValidationResult, BypassInfo, BypassRuleType};
    ///
    /// let bypass_info = BypassInfo {
    ///     rule_type: BypassRuleType::TitleConvention,
    ///     user: "admin".to_string(),
    /// };
    ///
    /// let bypassed = ValidationResult::bypassed(bypass_info.clone());
    /// assert_eq!(bypassed.bypass_info(), Some(&bypass_info));
    ///
    /// let valid = ValidationResult::valid();
    /// assert_eq!(valid.bypass_info(), None);
    /// ```
    pub fn bypass_info(&self) -> Option<&BypassInfo> {
        self.bypass_info.as_ref()
    }
}

impl BypassInfo {
    /// Returns the username of the user who had bypass permissions
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::validation_result::{BypassInfo, BypassRuleType};
    ///
    /// let bypass_info = BypassInfo {
    ///     rule_type: BypassRuleType::TitleConvention,
    ///     user: "admin".to_string(),
    /// };
    ///
    /// assert_eq!(bypass_info.user_login(), Some("admin"));
    /// ```
    pub fn user_login(&self) -> Option<&str> {
        Some(&self.user)
    }

    /// Returns a description of the bypass based on the rule type
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::validation_result::{BypassInfo, BypassRuleType};
    ///
    /// let bypass_info = BypassInfo {
    ///     rule_type: BypassRuleType::TitleConvention,
    ///     user: "admin".to_string(),
    /// };
    ///
    /// assert_eq!(bypass_info.description(), Some("Title validation bypassed"));
    /// ```
    pub fn description(&self) -> Option<&str> {
        match self.rule_type {
            BypassRuleType::TitleConvention => Some("Title validation bypassed"),
            BypassRuleType::WorkItemReference => Some("Work item validation bypassed"),
        }
    }
}

impl std::fmt::Display for BypassRuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BypassRuleType::TitleConvention => write!(f, "Title Convention"),
            BypassRuleType::WorkItemReference => write!(f, "Work Item Reference"),
        }
    }
}

#[cfg(test)]
#[path = "validation_result_tests.rs"]
mod tests;
