//! # Validation Checks
//!
//! This module contains the validation checks that are performed on pull requests.
//!
//! The checks are organized into submodules:
//! - `title`: Validates that PR titles follow the Conventional Commits format
//! - `work_item`: Validates that PR descriptions reference a work item or issue
//!
//! These checks are used by the `MergeWarden` to determine if a PR is valid
//! and can be merged.

use crate::{
    config::{BypassRule, CurrentPullRequestValidationConfiguration, VALID_PR_TYPES},
    size::PrSizeInfo,
    validation_result::{BypassInfo, BypassRuleType, ValidationResult},
};
use merge_warden_developer_platforms::models::{PullRequest, PullRequestFile, User};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::OnceLock;

/// Compiled once at first use. Handles all four supported closing-keyword formats:
/// `#NNN`, `GH-NNN`, full GitHub URL, and `owner/repo#NNN` (including dots in names).
static CLOSING_ISSUE_REGEX: OnceLock<Regex> = OnceLock::new();

/// Returns the compiled closing-issue regex, initialising it on first call.
fn closing_issue_regex() -> &'static Regex {
    CLOSING_ISSUE_REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)(fixes|closes|resolves)\s+(#(\d+)|GH-(\d+)|https://github\.com/([^/\s]+)/([^/\s]+)/issues/(\d+)|([a-zA-Z0-9_.-]+)/([a-zA-Z0-9_.-]+)#(\d+))",
        )
        .expect("CLOSING_ISSUE_REGEX is a valid regex")
    })
}

/// Compiled once at first use. Matches any issue reference keyword — both closing
/// (`fixes`, `closes`, `resolves`) and informational (`references`, `relates to`) —
/// in all four supported reference formats.
static ANY_ISSUE_REGEX: OnceLock<Regex> = OnceLock::new();

/// Returns the compiled any-issue regex, initialising it on first call.
fn any_issue_regex() -> &'static Regex {
    ANY_ISSUE_REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)(fixes|closes|resolves|references|relates\s+to)\s+(#(\d+)|GH-(\d+)|https://github\.com/([^/\s]+)/([^/\s]+)/issues/(\d+)|([a-zA-Z0-9_.-]+)/([a-zA-Z0-9_.-]+)#(\d+))",
        )
        .expect("ANY_ISSUE_REGEX is a valid regex")
    })
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;

/// A parsed issue reference extracted from a pull request body.
///
/// Carries enough information to fetch the issue from the appropriate repository,
/// which may differ from the repository the PR lives in.
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::IssueReference;
///
/// let same = IssueReference::SameRepo { issue_number: 42 };
/// assert_eq!(same.issue_number(), 42);
///
/// let cross = IssueReference::CrossRepo {
///     owner: "acme".to_string(),
///     repo: "widgets".to_string(),
///     issue_number: 7,
/// };
/// assert_eq!(cross.issue_number(), 7);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueReference {
    /// Issue in the same repository as the PR.
    SameRepo {
        /// Issue number.
        issue_number: u64,
    },
    /// Issue in a different repository.
    CrossRepo {
        /// Repository owner.
        owner: String,
        /// Repository name.
        repo: String,
        /// Issue number.
        issue_number: u64,
    },
}

impl IssueReference {
    /// Returns the issue number regardless of reference kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::checks::IssueReference;
    ///
    /// let r = IssueReference::SameRepo { issue_number: 99 };
    /// assert_eq!(r.issue_number(), 99);
    /// ```
    pub fn issue_number(&self) -> u64 {
        match self {
            Self::SameRepo { issue_number } | Self::CrossRepo { issue_number, .. } => *issue_number,
        }
    }
}

/// A structured diagnosis of why a PR title failed conventional-commit validation.
///
/// Contains the list of specific issues detected in the title and, when possible,
/// a best-effort corrected title string.
///
/// `issues` may contain multiple entries when several problems are observed
/// simultaneously (e.g. [`TitleIssue::LeadingWhitespace`] and
/// [`TitleIssue::UppercaseType`] on `" FEAT: add login"`).
///
/// `suggested_fix` is `None` when no actionable correction can be inferred,
/// for example when [`TitleIssue::NoTypePrefix`] or [`TitleIssue::EmptyDescription`]
/// is reported.
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::{TitleDiagnosis, TitleIssue};
///
/// let diagnosis = TitleDiagnosis {
///     issues: vec![TitleIssue::UppercaseType { found: "FEAT".to_string() }],
///     suggested_fix: Some("feat: add login".to_string()),
/// };
/// assert_eq!(diagnosis.issues.len(), 1);
/// assert!(diagnosis.suggested_fix.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitleDiagnosis {
    /// The specific issues detected in the PR title.
    ///
    /// May contain multiple entries when several problems are present simultaneously.
    pub issues: Vec<TitleIssue>,

    /// A best-effort corrected title string, or `None` when no fix can be inferred.
    pub suggested_fix: Option<String>,
}

/// The result of validating a PR title, combining the validation outcome with
/// an optional structured diagnosis.
///
/// This type is returned by [`check_pr_title`] and `MergeWarden::check_title`.
/// It replaces the plain [`crate::validation_result::ValidationResult`] for title
/// checks so that call sites can surface actionable feedback to PR authors.
///
/// `diagnosis` is:
/// - `Some` when the title is **invalid** and not bypassed — contains the specific issues
/// - `None` when the title is valid, or when validation was bypassed
///
/// The delegation methods [`is_valid`], [`was_bypassed`], and [`bypass_info`] forward
/// to the inner [`crate::validation_result::ValidationResult`] so that existing call
/// sites in `lib.rs` do not need to change.
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::TitleValidationResult;
/// use merge_warden_core::validation_result::ValidationResult;
///
/// let result = TitleValidationResult {
///     validation: ValidationResult::valid(),
///     diagnosis: None,
/// };
/// assert!(result.is_valid());
/// assert!(!result.was_bypassed());
/// assert!(result.bypass_info().is_none());
/// ```
///
/// [`is_valid`]: TitleValidationResult::is_valid
/// [`was_bypassed`]: TitleValidationResult::was_bypassed
/// [`bypass_info`]: TitleValidationResult::bypass_info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitleValidationResult {
    /// The underlying validation outcome (valid, invalid, or bypassed).
    pub validation: ValidationResult,

    /// Structured diagnosis, present only when the title is invalid and not bypassed.
    pub diagnosis: Option<TitleDiagnosis>,
}

impl TitleValidationResult {
    /// Returns `true` if validation passed (either valid content or bypassed).
    ///
    /// Delegates to [`ValidationResult::is_valid`][crate::validation_result::ValidationResult::is_valid].
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::checks::TitleValidationResult;
    /// use merge_warden_core::validation_result::ValidationResult;
    ///
    /// let result = TitleValidationResult { validation: ValidationResult::valid(), diagnosis: None };
    /// assert!(result.is_valid());
    /// ```
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validation.is_valid()
    }

    /// Returns `true` if validation passed due to a bypass rule.
    ///
    /// Delegates to [`ValidationResult::was_bypassed`][crate::validation_result::ValidationResult::was_bypassed].
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::checks::TitleValidationResult;
    /// use merge_warden_core::validation_result::{BypassInfo, BypassRuleType, ValidationResult};
    ///
    /// let bypass_info = BypassInfo {
    ///     rule_type: BypassRuleType::TitleConvention,
    ///     user: "release-bot".to_string(),
    /// };
    /// let result = TitleValidationResult {
    ///     validation: ValidationResult::bypassed(bypass_info),
    ///     diagnosis: None,
    /// };
    /// assert!(result.was_bypassed());
    /// ```
    #[must_use]
    pub fn was_bypassed(&self) -> bool {
        self.validation.was_bypassed()
    }

    /// Returns the bypass information if a bypass was used, or `None` otherwise.
    ///
    /// Delegates to [`ValidationResult::bypass_info`][crate::validation_result::ValidationResult::bypass_info].
    ///
    /// # Examples
    ///
    /// ```
    /// use merge_warden_core::checks::TitleValidationResult;
    /// use merge_warden_core::validation_result::{BypassInfo, BypassRuleType, ValidationResult};
    ///
    /// let bypass_info = BypassInfo {
    ///     rule_type: BypassRuleType::TitleConvention,
    ///     user: "release-bot".to_string(),
    /// };
    /// let result = TitleValidationResult {
    ///     validation: ValidationResult::bypassed(bypass_info.clone()),
    ///     diagnosis: None,
    /// };
    /// assert_eq!(result.bypass_info(), Some(&bypass_info));
    /// ```
    #[must_use]
    pub fn bypass_info(&self) -> Option<&BypassInfo> {
        self.validation.bypass_info()
    }
}

/// A specific issue found in a PR title that explains why the title does not conform
/// to the Conventional Commits format.
///
/// Each variant carries the data needed to produce a specific human-readable message
/// and, in many cases, a suggested corrected title.
///
/// Several variants can apply simultaneously; for example a title of `" FEAT: add login"`
/// produces both [`LeadingWhitespace`][TitleIssue::LeadingWhitespace] and
/// [`UppercaseType`][TitleIssue::UppercaseType].
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::TitleIssue;
///
/// let issue = TitleIssue::UppercaseType { found: "FEAT".to_string() };
/// assert!(matches!(issue, TitleIssue::UppercaseType { .. }));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TitleIssue {
    /// The description portion of the title is absent or whitespace-only.
    ///
    /// Triggered when the title contains a valid type and colon (e.g. `"feat: "`)
    /// but no non-whitespace description follows.  No `suggested_fix` is produced
    /// because the user must supply meaningful content.
    ///
    /// # Examples
    ///
    /// - `"feat: "` — colon with trailing space only
    /// - `"feat:  "` — colon with multiple trailing spaces
    EmptyDescription,

    /// The scope is empty.
    ///
    /// Triggered when parentheses are present but contain no scope name, e.g. `feat():`.
    /// The `suggested_fix` removes the empty parentheses entirely.
    ///
    /// # Examples
    ///
    /// - `"feat(): add login"` — parentheses with no scope name
    EmptyScope,

    /// The scope contains characters outside `[a-z0-9_-]`.
    ///
    /// The `suggested_fix` lowercases the scope and replaces spaces with `-`.
    ///
    /// # Examples
    ///
    /// - `"feat(Auth): add login"` — uppercase letters in scope
    /// - `"feat(user service): x"` — space inside scope
    InvalidScope {
        /// The raw scope string as extracted from the title (without surrounding parentheses).
        scope: String,
    },

    /// The title begins with one or more whitespace characters.
    ///
    /// Detection trims the title before applying all subsequent checks, so this
    /// variant may appear alongside others such as [`UppercaseType`][TitleIssue::UppercaseType].
    ///
    /// # Examples
    ///
    /// - `" feat: add login"` — one leading space
    LeadingWhitespace,

    /// A recognised type is present but is not followed by `:`.
    ///
    /// The `suggested_fix` inserts `:` after the type token.
    ///
    /// # Examples
    ///
    /// - `"feat add login"` — type without colon separator
    MissingColon,

    /// The `:` separator is present but the character immediately after it is not a space.
    ///
    /// The `suggested_fix` inserts the missing space.
    ///
    /// # Examples
    ///
    /// - `"feat:add login"` — no space after colon
    MissingSpaceAfterColon,

    /// No recognisable type prefix was found at the start of the title.
    ///
    /// This is the fallback variant emitted when none of the other patterns match.
    /// No `suggested_fix` is produced.
    ///
    /// # Examples
    ///
    /// - `"Add login functionality"` — plain sentence, no type prefix
    /// - `""` — empty title
    /// - `"   "` — whitespace-only title
    NoTypePrefix,

    /// The type token does not appear in the approved list and did not match a known synonym.
    ///
    /// `nearest_valid` is `Some` when the token is a known typo or synonym (e.g.
    /// `"feature"` → `"feat"`), and `None` when the token is completely unrecognised.
    /// When `nearest_valid` is `Some`, the `suggested_fix` replaces the token; otherwise
    /// `suggested_fix` is `None`.
    ///
    /// # Examples
    ///
    /// - `"feature: add login"` → `found: "feature"`, `nearest_valid: Some("feat")`
    /// - `"xyz: add login"` → `found: "xyz"`, `nearest_valid: None`
    UnrecognizedType {
        /// The unrecognised type token as extracted from the title.
        found: String,
        /// The nearest valid type from the approved list, if a known synonym mapping exists.
        nearest_valid: Option<String>,
    },

    /// The type token is a correctly-spelled conventional commit type but is not lowercase.
    ///
    /// The `suggested_fix` lowercases the type token.
    ///
    /// # Examples
    ///
    /// - `"FEAT: add login"` → `found: "FEAT"`
    /// - `"Fix: bug"` → `found: "Fix"`
    UppercaseType {
        /// The type token as it appears in the title (wrong case).
        found: String,
    },

    /// There is whitespace between the type/scope token and the `:` separator.
    ///
    /// The `suggested_fix` removes the extra whitespace.
    ///
    /// # Examples
    ///
    /// - `"feat : add login"` — space before colon
    /// - `"feat(auth) : add login"` — space before colon with scope present
    WhitespaceBeforeColon {
        /// The prefix (type + optional scope) including the trailing whitespace, as extracted.
        found: String,
    },
}

impl fmt::Display for TitleIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LeadingWhitespace => write!(
                f,
                "The title starts with whitespace \u{2014} please remove the leading spaces."
            ),
            Self::WhitespaceBeforeColon { found } => write!(
                f,
                "There is whitespace before the `:` separator (found `{found}`) \u{2014} remove the extra space."
            ),
            Self::UppercaseType { found } => write!(
                f,
                "The type `{found}` must be lowercase (e.g. use `{}` instead).",
                found.to_lowercase()
            ),
            Self::UnrecognizedType {
                found,
                nearest_valid: Some(nv),
            } => write!(
                f,
                "The type `{found}` is not a recognized conventional commit type \u{2014} did you mean `{nv}`?"
            ),
            Self::UnrecognizedType {
                found,
                nearest_valid: None,
            } => write!(
                f,
                "The type `{found}` is not a recognized conventional commit type."
            ),
            Self::MissingColon => write!(
                f,
                "A `:` separator is required between the type/scope and the description (e.g. `feat: ...`)."
            ),
            Self::MissingSpaceAfterColon => write!(
                f,
                "A space is required after the `:` separator (e.g. `feat: description`)."
            ),
            Self::EmptyDescription => write!(
                f,
                "The description after `:` is missing or blank \u{2014} please add a short summary."
            ),
            Self::EmptyScope => write!(
                f,
                "The scope is empty \u{2014} either add a scope name (e.g. `feat(auth): ...`) or remove the parentheses completely (e.g. `feat: ...`)."
            ),
            Self::InvalidScope { scope } => write!(
                f,
                "The scope `{scope}` contains invalid characters; scopes must only use lowercase letters, digits, `_`, and `-`."
            ),
            Self::NoTypePrefix => write!(
                f,
                "No conventional commit type prefix was found at the start of the title."
            ),
        }
    }
}

/// Common typo / synonym mappings from a known-wrong type word to the correct one.
///
/// Used by [`diagnose_pr_title`] to populate [`TitleIssue::UnrecognizedType::nearest_valid`]
/// and to build `suggested_fix` strings.
const TYPE_TYPO_MAP: &[(&str, &str)] = &[
    ("bug", "fix"),
    ("bugfix", "fix"),
    ("dep", "chore"),
    ("dependencies", "chore"),
    ("enhancement", "feat"),
    ("feature", "feat"),
    ("hotfix", "fix"),
];

/// Constructs a best-effort corrected title from the working (trimmed) title and the
/// list of issues that were diagnosed.
///
/// Returns `None` when the problems are unresolvable (e.g. `NoTypePrefix`,
/// `EmptyDescription`).
fn build_suggested_fix(working: &str, issues: &[TitleIssue]) -> Option<String> {
    // Unresolvable issues: no fix possible.
    // UnrecognizedType without a nearest_valid is also unresolvable — we don't know
    // what type the author intended, so we cannot suggest a corrected title.
    let unresolvable = issues.iter().any(|i| {
        matches!(
            i,
            TitleIssue::NoTypePrefix
                | TitleIssue::EmptyDescription
                | TitleIssue::UnrecognizedType {
                    nearest_valid: None,
                    ..
                }
        )
    });
    if unresolvable {
        return None;
    }

    let mut result = working.to_string();

    // Apply corrections in reverse order of their lexical position so that
    // index-based mutations don't invalidate later positions.

    // Fix UppercaseType or UnrecognizedType — replace the type token at the start.
    let token_end = result.find(['(', '!', ':', ' ']).unwrap_or(result.len());
    let replacement_token: Option<String> = issues.iter().find_map(|i| match i {
        TitleIssue::UppercaseType { found } => Some(found.to_lowercase()),
        TitleIssue::UnrecognizedType { nearest_valid, .. } => nearest_valid.clone(),
        _ => None,
    });
    if let Some(ref rep) = replacement_token {
        result = format!("{}{}", rep, &result[token_end..]);
    }

    // Fix WhitespaceBeforeColon — remove whitespace directly before the colon.
    if issues
        .iter()
        .any(|i| matches!(i, TitleIssue::WhitespaceBeforeColon { .. }))
    {
        // After possibly replacing the type token, find and strip whitespace before `:`.
        if let Some(colon_pos) = result.find(':') {
            let trimmed_prefix = result[..colon_pos].trim_end().to_string();
            let after_colon = result[colon_pos..].to_string();
            result = format!("{trimmed_prefix}{after_colon}");
        }
    }

    // Fix EmptyScope — remove the empty parentheses `()`.
    if issues.iter().any(|i| matches!(i, TitleIssue::EmptyScope)) {
        result = result.replacen("()", "", 1);
    }

    // Fix InvalidScope — lowercase the scope and replace spaces with `-`.
    if let Some(TitleIssue::InvalidScope { scope }) = issues
        .iter()
        .find(|i| matches!(i, TitleIssue::InvalidScope { .. }))
    {
        let fixed_scope = scope.to_lowercase().replace(' ', "-");
        // Replace only the first occurrence of `(<scope>)` in the result.
        let needle = format!("({scope})");
        let replacement = format!("({fixed_scope})");
        result = result.replacen(&needle, &replacement, 1);
    }

    // Fix MissingColon — insert `:` after the type token (and optional scope/!).
    if issues.iter().any(|i| matches!(i, TitleIssue::MissingColon)) {
        // Find the end of the type+scope+! prefix.
        let prefix_end = result.find(' ').unwrap_or(result.len());
        result = format!("{}:{}", &result[..prefix_end], &result[prefix_end..]);
    }

    // Fix MissingSpaceAfterColon — insert a space after the colon.
    if issues
        .iter()
        .any(|i| matches!(i, TitleIssue::MissingSpaceAfterColon))
    {
        if let Some(colon_pos) = result.find(':') {
            let after = &result[colon_pos + 1..];
            if !after.starts_with(' ') {
                result = format!("{}: {}", &result[..colon_pos], after);
            }
        }
    }

    Some(result)
}

/// Checks for missing-colon, missing-space-after-colon, and empty-description issues.
///
/// `MissingColon` is reported whenever there is no `:` in the title, regardless of any other
/// type issues already recorded (e.g. `UppercaseType` or `UnrecognizedType`). This allows the
/// caller to receive a complete picture of all problems at once.
///
/// Appends the relevant [`TitleIssue`] variant(s) found in the working title.
fn check_colon_issues(working: &str, colon_pos: Option<usize>, issues: &mut Vec<TitleIssue>) {
    if colon_pos.is_none() {
        // ── Step 5: Missing colon ─────────────────────────────────────────────
        issues.push(TitleIssue::MissingColon);
    } else if let Some(colon) = colon_pos {
        let after_colon = &working[colon + 1..];
        if !after_colon.is_empty() && !after_colon.starts_with(' ') {
            // ── Step 6: Missing space after colon ─────────────────────────────
            issues.push(TitleIssue::MissingSpaceAfterColon);
        } else if after_colon.trim().is_empty() {
            // ── Step 7: Empty description ──────────────────────────────────────
            issues.push(TitleIssue::EmptyDescription);
        }
    }
}

/// Checks for an empty or invalid scope in the working title.
///
/// Appends [`TitleIssue::EmptyScope`] when the parentheses contain no scope name, or
/// [`TitleIssue::InvalidScope`] when the scope contains characters outside `[a-z0-9_-]`.
fn check_scope(working: &str, issues: &mut Vec<TitleIssue>) {
    if let Some(scope_start) = working.find('(') {
        let scope_end = working[scope_start..].find(')').map(|i| scope_start + i);
        if let Some(end) = scope_end {
            let scope_content = &working[scope_start + 1..end];
            if scope_content.is_empty() {
                // Empty parentheses: e.g. `feat(): add login`
                issues.push(TitleIssue::EmptyScope);
            } else {
                let scope_invalid = scope_content
                    .chars()
                    .any(|c| !matches!(c, 'a'..='z' | '0'..='9' | '_' | '-'));
                if scope_invalid {
                    issues.push(TitleIssue::InvalidScope {
                        scope: scope_content.to_string(),
                    });
                }
            }
        }
    }
}

/// Analyses a PR title that is known to be invalid and returns a structured diagnosis
/// describing every detected problem and, where possible, a suggested corrected title.
///
/// This is a pure function with no I/O or mutable state.  It is called by
/// [`check_pr_title`] on the failure path.
///
/// Multiple [`TitleIssue`] entries may be returned simultaneously when the title
/// exhibits several problems at once (e.g. leading whitespace **and** an uppercase type).
/// Detection continues after each non-fatal check so that the caller receives a
/// complete picture.
///
/// # Arguments
///
/// * `title` - The raw PR title string, exactly as received from the provider.
///
/// # Returns
///
/// A [`TitleDiagnosis`] containing the list of issues found and, when possible,
/// a best-effort corrected title string.
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::{diagnose_pr_title, TitleIssue};
///
/// let diagnosis = diagnose_pr_title(" FEAT: add login");
/// assert!(diagnosis.issues.contains(&TitleIssue::LeadingWhitespace));
/// assert!(diagnosis.issues.contains(&TitleIssue::UppercaseType { found: "FEAT".to_string() }));
/// assert_eq!(diagnosis.suggested_fix.as_deref(), Some("feat: add login"));
/// ```
#[must_use]
pub fn diagnose_pr_title(title: &str) -> TitleDiagnosis {
    let mut issues: Vec<TitleIssue> = Vec::new();

    // ── Step 1: Leading whitespace ────────────────────────────────────────────
    let leading_ws = title.starts_with(|c: char| c.is_whitespace());
    if leading_ws {
        issues.push(TitleIssue::LeadingWhitespace);
    }
    // All subsequent work is performed on the trimmed title.
    let working = title.trim();

    // If the working string is empty after trimming, there is no prefix at all.
    if working.is_empty() {
        issues.push(TitleIssue::NoTypePrefix);
        return TitleDiagnosis {
            issues,
            suggested_fix: None,
        };
    }

    // ── Extract the candidate type token (chars before `(`, `!`, `:`, or space) ──
    let token_end = working.find(['(', '!', ':', ' ']).unwrap_or(working.len());
    let raw_token = &working[..token_end];

    // ── Step 2: Whitespace before colon ──────────────────────────────────────
    // Look for whitespace immediately before the first `:` in the prefix region
    // (the part before the description, i.e. up to and including the colon).
    let colon_pos = working.find(':');
    if let Some(pos) = colon_pos {
        if pos > 0 {
            let char_before = working[..pos]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace);
            if char_before {
                let prefix_with_space = &working[..pos];
                issues.push(TitleIssue::WhitespaceBeforeColon {
                    found: prefix_with_space.to_string(),
                });
            }
        }
    }

    // ── Step 3 & 4: UnrecognizedType / UppercaseType ─────────────────────────
    let token_lower = raw_token.to_lowercase();
    let is_valid_exact = VALID_PR_TYPES.contains(&raw_token);
    let is_valid_lower = VALID_PR_TYPES.contains(&token_lower.as_str());

    if !raw_token.is_empty() && !is_valid_exact {
        if is_valid_lower {
            // ── Step 4: Uppercase type ────────────────────────────────────────
            issues.push(TitleIssue::UppercaseType {
                found: raw_token.to_string(),
            });
        } else {
            // ── Step 3: Unrecognised type ─────────────────────────────────────
            // Only diagnose as UnrecognizedType when there is a colon (indicating a
            // conventional-commit attempt) or the token is a known synonym/typo.
            // Otherwise the title is plain prose and will fall through to NoTypePrefix.
            let nearest_valid = TYPE_TYPO_MAP
                .iter()
                .find(|(typo, _)| *typo == token_lower.as_str())
                .map(|(_, correct)| (*correct).to_string());
            if colon_pos.is_some() || nearest_valid.is_some() {
                issues.push(TitleIssue::UnrecognizedType {
                    found: raw_token.to_string(),
                    nearest_valid,
                });
            }
        }
    }

    // ── Steps 5–8 only make sense when we have a potentially-valid type token ──
    // (i.e. the token is valid lowercase, or we are past the UppercaseType branch)
    let effective_token = if is_valid_lower || is_valid_exact {
        Some(token_lower.as_str().to_string())
    } else {
        // Unrecognised type — look it up in the typo map to get a correctable form.
        TYPE_TYPO_MAP
            .iter()
            .find(|(typo, _)| *typo == token_lower.as_str())
            .map(|(_, correct)| (*correct).to_string())
    };

    if let Some(ref _eff_token) = effective_token {
        check_scope(working, &mut issues);
        check_colon_issues(working, colon_pos, &mut issues);
    }

    // ── Step 9: NoTypePrefix fallback ────────────────────────────────────────
    // Only push NoTypePrefix when we genuinely could not identify any recognisable
    // conventional-commit structure.  Guarding on `effective_token.is_none()` prevents
    // valid titles (where `effective_token` is `Some`) from being misidentified as
    // having no prefix just because they raised no issues.
    if (issues.is_empty() && effective_token.is_none())
        || (issues.len() == 1
            && issues[0] == TitleIssue::LeadingWhitespace
            && effective_token.is_none())
    {
        issues.push(TitleIssue::NoTypePrefix);
        return TitleDiagnosis {
            issues,
            suggested_fix: None,
        };
    }

    // ── Build suggested_fix ───────────────────────────────────────────────────
    // If no issues were found, the title is valid and no fix is needed.
    if issues.is_empty() {
        return TitleDiagnosis {
            issues,
            suggested_fix: None,
        };
    }
    let suggested_fix = build_suggested_fix(working, &issues);

    TitleDiagnosis {
        issues,
        suggested_fix,
    }
}

/// Validates that the PR title follows the Conventional Commits format with bypass support.
///
/// This function checks if the PR title follows the Conventional Commits format.
/// If bypass rules are provided and the PR author is allowed to bypass title validation,
/// the function will return a successful result with bypass information.
///
/// # Arguments
///
/// * `pr` - The pull request to validate
/// * `bypass_rule` - The bypass rule for title validation
/// * `current_configuration` - The current validation configuration
///
/// # Returns
///
/// A [`TitleValidationResult`] whose `diagnosis` field is:
/// - `None` when the title is valid or validation was bypassed
/// - `Some` when the title is invalid, containing structured feedback and an optional
///   suggested-fix string
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{PullRequest, User};
/// use merge_warden_core::checks::check_pr_title;
/// use merge_warden_core::config::{BypassRule, CurrentPullRequestValidationConfiguration};
///
/// // Regular validation — valid title
/// let pr = PullRequest {
///     number: 123,
///     title: "feat(auth): add GitHub login".to_string(),
///     draft: false,
///     body: Some("This PR adds GitHub login functionality.".to_string()),
///     author: None,
///     milestone_number: None,
/// };
///
/// let bypass_rule = BypassRule::default();
/// let config = CurrentPullRequestValidationConfiguration::default();
/// let result = check_pr_title(&pr, &bypass_rule, &config);
/// assert!(result.is_valid());
/// assert!(!result.was_bypassed());
/// assert!(result.diagnosis.is_none());
///
/// // Bypass validation for authorized user with invalid title
/// let pr_with_bad_title = PullRequest {
///     number: 124,
///     title: "fix urgent bug".to_string(),
///     draft: false,
///     body: Some("Emergency fix".to_string()),
///     author: Some(User {
///         id: 123,
///         login: "emergency-bot".to_string(),
///     }),
///     milestone_number: None,
/// };
///
/// let bypass_rule = BypassRule::new(true, vec!["emergency-bot".to_string()]);
/// let result = check_pr_title(&pr_with_bad_title, &bypass_rule, &config);
/// assert!(result.is_valid());
/// assert!(result.was_bypassed());
/// assert!(result.diagnosis.is_none());
/// ```
#[must_use]
pub fn check_pr_title(
    pr: &PullRequest,
    bypass_rule: &BypassRule,
    current_configuration: &CurrentPullRequestValidationConfiguration,
) -> TitleValidationResult {
    let user = pr.author.as_ref();

    // Check if user can bypass title validation
    if bypass_rule.can_bypass_validation(user) {
        let bypass_info = BypassInfo {
            rule_type: BypassRuleType::TitleConvention,
            user: user.unwrap().login.clone(), // Safe unwrap since can_bypass_validation checks user existence
        };

        return TitleValidationResult {
            validation: ValidationResult::bypassed(bypass_info),
            diagnosis: None,
        };
    }

    // Otherwise, perform normal validation.
    // NOTE: The title_pattern regex is recompiled on every call. Since the pattern is
    // configuration-derived (not static), OnceLock is not suitable here. A per-instance
    // cache keyed by pattern string would improve throughput under high load.
    // This is a known performance gap — tracked for future optimisation.
    let regex = match Regex::new(&current_configuration.title_pattern) {
        Ok(r) => r,
        Err(_) => {
            return TitleValidationResult {
                validation: ValidationResult::invalid(),
                diagnosis: Some(diagnose_pr_title(&pr.title)),
            }
        }
    };

    if regex.is_match(&pr.title) {
        TitleValidationResult {
            validation: ValidationResult::valid(),
            diagnosis: None,
        }
    } else {
        let diagnosis = diagnose_pr_title(&pr.title);
        TitleValidationResult {
            validation: ValidationResult::invalid(),
            diagnosis: Some(diagnosis),
        }
    }
}

/// Checks if the PR body contains a reference to a work item or GitHub issue,
/// with support for bypass rules.
///
/// This function first checks if the PR author can bypass work item validation
/// according to the configured bypass rules. If bypass is allowed, the function
/// returns a successful result with bypass information. Otherwise, it performs
/// the standard work item reference validation.
///
/// # Arguments
///
/// * `pr` - The pull request to check
/// * `bypass_rules` - The bypass rules configuration
///
/// # Returns
///
/// A `ValidationResult` indicating whether a work item reference was found
/// or if the validation was bypassed
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{PullRequest, User};
/// use merge_warden_core::checks::check_work_item_reference;
/// use merge_warden_core::config::{BypassRule, CurrentPullRequestValidationConfiguration};
///
/// // PR author who can bypass validation
/// let bypass_user = User {
///     id: 123,
///     login: "bypass-user".to_string(),
/// };
///
/// let pr_with_bypass = PullRequest {
///     number: 123,
///     title: "feat: emergency fix".to_string(),
///     draft: false,
///     body: Some("Emergency fix without work item".to_string()),
///     author: Some(bypass_user),
///     milestone_number: None,
/// };
///
/// let bypass_rule = BypassRule::new(true, vec!["bypass-user".to_string()]);
/// let config = CurrentPullRequestValidationConfiguration::default();
///
/// let result = check_work_item_reference(&pr_with_bypass, &bypass_rule, &config);
/// assert!(result.is_valid()); // Bypassed, so returns true
/// assert!(result.was_bypassed()); // Indicates bypass was used
/// ```
pub fn check_work_item_reference(
    pr: &PullRequest,
    bypass_rules: &BypassRule,
    current_configuration: &CurrentPullRequestValidationConfiguration,
) -> ValidationResult {
    // Check if the user can bypass work item validation
    let user = pr.author.as_ref();
    if bypass_rules.can_bypass_validation(user) {
        let bypass_info = BypassInfo {
            rule_type: BypassRuleType::WorkItemReference,
            user: user.unwrap().login.clone(), // Safe unwrap since can_bypass_validation checks user existence
        };

        return ValidationResult::bypassed(bypass_info);
    }

    // If no bypass, perform normal validation
    match &pr.body {
        Some(body) => {
            let regex = match Regex::new(current_configuration.work_item_reference_pattern.as_str())
            {
                Ok(r) => r,
                Err(_) => return ValidationResult::invalid(),
            };

            if regex.is_match(body) {
                ValidationResult::valid()
            } else {
                ValidationResult::invalid()
            }
        }
        None => ValidationResult::invalid(),
    }
}

/// Validates PR size based on file changes and configuration.
///
/// This function analyzes the size of a pull request by examining the files changed
/// and calculating the total lines modified. It supports file exclusion patterns,
/// can optionally fail the check for oversized PRs, and supports bypass rules for
/// automated tools that may legitimately create large PRs.
///
/// # Arguments
///
/// * `pr_files` - List of files changed in the pull request
/// * `user` - The user who created the pull request (for bypass checking)
/// * `bypass_rule` - Bypass rule for size validation (allows specific users to bypass size checks)
/// * `config` - Current validation configuration containing size check settings
///
/// # Returns
///
/// A `ValidationResult` indicating the size validation status
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::{PullRequestFile, User};
/// use merge_warden_core::checks::check_pr_size;
/// use merge_warden_core::config::{BypassRule, CurrentPullRequestValidationConfiguration, PrSizeCheckConfig};
///
/// let files = vec![
///     PullRequestFile {
///         filename: "src/main.rs".to_string(),
///         additions: 10,
///         deletions: 5,
///         changes: 15,
///         status: "modified".to_string(),
///     },
///     PullRequestFile {
///         filename: "README.md".to_string(),
///         additions: 2,
///         deletions: 1,
///         changes: 3,
///         status: "modified".to_string(),
///     },
/// ];
///
/// let user = User {
///     login: "developer".to_string(),
///     id: 123,
/// };
/// let bypass_rule = BypassRule::default();
/// let mut config = CurrentPullRequestValidationConfiguration::default();
/// config.pr_size_check.enabled = true;
/// config.pr_size_check.excluded_file_patterns = vec!["*.md".to_string()];
///
/// let result = check_pr_size(&files, Some(&user), &bypass_rule, &config);
/// // Only src/main.rs counts (15 lines), README.md is excluded
/// assert!(result.is_valid()); // 15 lines is XS, should be valid
/// ```
pub fn check_pr_size(
    pr_files: &[PullRequestFile],
    user: Option<&User>,
    bypass_rule: &BypassRule,
    config: &CurrentPullRequestValidationConfiguration,
) -> ValidationResult {
    // If size checking is disabled, always return valid
    if !config.pr_size_check.enabled {
        return ValidationResult::valid();
    }

    // Check if the user can bypass size validation
    if bypass_rule.can_bypass_validation(user) {
        return ValidationResult::valid();
    }

    // Calculate size info with file exclusions
    let size_info = PrSizeInfo::from_files_with_exclusions(
        pr_files,
        &config.pr_size_check.get_effective_thresholds(),
        &config.pr_size_check.excluded_file_patterns,
    );

    // Check if we should fail for oversized PRs
    if config.pr_size_check.fail_on_oversized && size_info.is_oversized() {
        ValidationResult::invalid()
    } else {
        ValidationResult::valid()
    }
}

/// Extracts the first closing-keyword issue reference from a pull request body.
///
/// Scans `body` for `fixes`, `closes`, or `resolves` references in all supported
/// formats. Returns the first match found, or `None` if no closing reference is
/// present. Informational keywords (`references`, `relates to`) are intentionally
/// excluded — they satisfy the work-item link check but are not used for metadata
/// propagation.
///
/// # Arguments
///
/// * `body` - The pull request body text to scan.
///
/// # Returns
///
/// The first closing-keyword issue reference found, or `None`.
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::{extract_closing_issue_reference, IssueReference};
///
/// assert_eq!(
///     extract_closing_issue_reference("fixes #42"),
///     Some(IssueReference::SameRepo { issue_number: 42 }),
/// );
///
/// // Informational keywords are not closing references
/// assert_eq!(extract_closing_issue_reference("relates to #99"), None);
/// ```
pub fn extract_closing_issue_reference(body: &str) -> Option<IssueReference> {
    // Capture group layout:
    //   1: keyword  (fixes|closes|resolves)
    //   2: full reference text
    //   3: issue number from #NNN            (same-repo)
    //   4: issue number from GH-NNN          (same-repo)
    //   5: owner from full GitHub URL        (cross-repo)
    //   6: repo  from full GitHub URL        (cross-repo)
    //   7: issue number from full GitHub URL (cross-repo)
    //   8: owner from owner/repo#NNN         (cross-repo, dots allowed)
    //   9: repo  from owner/repo#NNN         (cross-repo, dots allowed)
    //  10: issue number from owner/repo#NNN  (cross-repo)
    let regex = closing_issue_regex();

    for cap in regex.captures_iter(body) {
        // #NNN — same-repo hash reference
        if let Some(n) = cap.get(3) {
            if let Ok(issue_number) = n.as_str().parse::<u64>() {
                return Some(IssueReference::SameRepo { issue_number });
            }
        }

        // GH-NNN — same-repo GH-prefixed reference
        if let Some(n) = cap.get(4) {
            if let Ok(issue_number) = n.as_str().parse::<u64>() {
                return Some(IssueReference::SameRepo { issue_number });
            }
        }

        // https://github.com/owner/repo/issues/NNN
        if let (Some(owner), Some(repo), Some(n)) = (cap.get(5), cap.get(6), cap.get(7)) {
            if let Ok(issue_number) = n.as_str().parse::<u64>() {
                return Some(IssueReference::CrossRepo {
                    owner: owner.as_str().to_string(),
                    repo: repo.as_str().to_string(),
                    issue_number,
                });
            }
        }

        // owner/repo#NNN
        if let (Some(owner), Some(repo), Some(n)) = (cap.get(8), cap.get(9), cap.get(10)) {
            if let Ok(issue_number) = n.as_str().parse::<u64>() {
                return Some(IssueReference::CrossRepo {
                    owner: owner.as_str().to_string(),
                    repo: repo.as_str().to_string(),
                    issue_number,
                });
            }
        }
    }

    None
}

/// Extracts the first issue reference from a pull request body, matching **any** supported
/// keyword — both closing (`fixes`, `closes`, `resolves`) and informational
/// (`references`, `relates to`).
///
/// Use this when the intent is to propagate issue metadata (milestone, project) onto a PR,
/// where an issue may require multiple PRs and all keyword forms should trigger propagation.
///
/// Use [`extract_closing_issue_reference`] instead when you specifically need a reference
/// that will close the issue on merge.
///
/// # Arguments
///
/// * `body` - The pull request body text to scan.
///
/// # Returns
///
/// The first issue reference found (closing or informational), or `None`.
///
/// # Examples
///
/// ```
/// use merge_warden_core::checks::{extract_any_issue_reference, IssueReference};
///
/// assert_eq!(
///     extract_any_issue_reference("references #42"),
///     Some(IssueReference::SameRepo { issue_number: 42 }),
/// );
///
/// assert_eq!(
///     extract_any_issue_reference("fixes #7"),
///     Some(IssueReference::SameRepo { issue_number: 7 }),
/// );
///
/// assert_eq!(extract_any_issue_reference("no reference here"), None);
/// ```
pub fn extract_any_issue_reference(body: &str) -> Option<IssueReference> {
    // Capture group layout (same as closing regex, but broader keyword set):
    //   1: keyword  (fixes|closes|resolves|references|relates to)
    //   2: full reference text
    //   3: issue number from #NNN            (same-repo)
    //   4: issue number from GH-NNN          (same-repo)
    //   5: owner from full GitHub URL        (cross-repo)
    //   6: repo  from full GitHub URL        (cross-repo)
    //   7: issue number from full GitHub URL (cross-repo)
    //   8: owner from owner/repo#NNN         (cross-repo, dots allowed)
    //   9: repo  from owner/repo#NNN         (cross-repo, dots allowed)
    //  10: issue number from owner/repo#NNN  (cross-repo)
    let regex = any_issue_regex();

    for cap in regex.captures_iter(body) {
        // #NNN — same-repo hash reference
        if let Some(n) = cap.get(3) {
            if let Ok(issue_number) = n.as_str().parse::<u64>() {
                return Some(IssueReference::SameRepo { issue_number });
            }
        }

        // GH-NNN — same-repo GH-prefixed reference
        if let Some(n) = cap.get(4) {
            if let Ok(issue_number) = n.as_str().parse::<u64>() {
                return Some(IssueReference::SameRepo { issue_number });
            }
        }

        // https://github.com/owner/repo/issues/NNN
        if let (Some(owner), Some(repo), Some(n)) = (cap.get(5), cap.get(6), cap.get(7)) {
            if let Ok(issue_number) = n.as_str().parse::<u64>() {
                return Some(IssueReference::CrossRepo {
                    owner: owner.as_str().to_string(),
                    repo: repo.as_str().to_string(),
                    issue_number,
                });
            }
        }

        // owner/repo#NNN
        if let (Some(owner), Some(repo), Some(n)) = (cap.get(8), cap.get(9), cap.get(10)) {
            if let Ok(issue_number) = n.as_str().parse::<u64>() {
                return Some(IssueReference::CrossRepo {
                    owner: owner.as_str().to_string(),
                    repo: repo.as_str().to_string(),
                    issue_number,
                });
            }
        }
    }

    None
}
