#
# ENVIRONMENT
#

variable "category" {
  default     = "mergew"
  description = "The name of the category that all the resources are running in."
}

variable "environment" {
  default     = "test"
  description = "The name of the environment that all the resources are running in."
}

#
# GITHUB
#

variable "github_app_id" {
  description = "Github App ID"
  type        = string
}

variable "github_app_private_key_path" {
  description = "Path to the file containing the Github App Private Key."
  type        = string
  sensitive   = true # Mark the path itself potentially sensitive, though the content is the real secret
}

variable "github_webhook_secret" {
  description = "The secret used to validate the GitHub webhook."
  type        = string
}

#
# LOCATION
#

variable "location" {
  default     = "australiaeast"
  description = "The full name of the Azure region in which the resources should be created."
}

#
# META
#

variable "meta_git_sha" {
  description = "The commit ID of the current commit from which the plan is being created."
  type        = string
}

variable "meta_version" {
  description = "The version of the infrastructure as it is being generated."
  type        = string
}

#
# TAGS
#

variable "tags" {
  description = "Tags to apply to all resources created."
  type        = map(string)
  default     = {}
}

#
# BYPASS RULES
#

variable "bypass_rules_title_enabled" {
  description = "Whether title validation bypass rules are enabled by default."
  type        = bool
  default     = true
}

variable "bypass_rules_title_users" {
  description = "List of users who can bypass title validation by default."
  type        = list(string)
  default     = []
}

variable "bypass_rules_work_item_enabled" {
  description = "Whether work item validation bypass rules are enabled by default."
  type        = bool
  default     = true
}

variable "bypass_rules_work_item_users" {
  description = "List of users who can bypass work item validation by default."
  type        = list(string)
  default     = ["renovate[bot]", "dependabot[bot]"]
}

#
# APPLICATION CONFIGURATION
#

variable "enforce_title_convention" {
  description = "Whether to enforce pull request title convention validation."
  type        = bool
  default     = true
}

variable "require_work_items" {
  description = "Whether to require work item references in pull requests."
  type        = bool
  default     = true
}

#
# PR SIZE CHECKING CONFIGURATION
#

variable "pr_size_enabled" {
  description = "Whether PR size checking is enabled."
  type        = bool
  default     = true
}

variable "pr_size_fail_on_oversized" {
  description = "Whether to fail the check for oversized PRs (XXL category)."
  type        = bool
  default     = false
}

variable "pr_size_label_prefix" {
  description = "Label prefix for size labels (defaults to 'size/')."
  type        = string
  default     = "size: "
}

variable "pr_size_add_comment" {
  description = "Whether to add educational comments for oversized PRs."
  type        = bool
  default     = true
}

variable "pr_size_excluded_file_patterns" {
  description = "File patterns to exclude from size calculations (e.g., ['*.md', '*.txt'])."
  type        = list(string)
  default     = []
}

# Size thresholds (optional - uses defaults if not specified)
variable "pr_size_xs_threshold" {
  description = "Threshold for extra small PRs (lines of code). Leave null to use default."
  type        = number
  default     = null
}

variable "pr_size_small_threshold" {
  description = "Threshold for small PRs (lines of code). Leave null to use default."
  type        = number
  default     = null
}

variable "pr_size_medium_threshold" {
  description = "Threshold for medium PRs (lines of code). Leave null to use default."
  type        = number
  default     = null
}

variable "pr_size_large_threshold" {
  description = "Threshold for large PRs (lines of code). Leave null to use default."
  type        = number
  default     = null
}

variable "pr_size_extra_large_threshold" {
  description = "Threshold for extra large PRs (lines of code). Leave null to use default."
  type        = number
  default     = null
}

variable "log_level" {
  description = "Application log level (trace, debug, info, warn, error)."
  type        = string
  default     = "info"

  validation {
    condition     = contains(["trace", "debug", "info", "warn", "error"], var.log_level)
    error_message = "Log level must be one of: trace, debug, info, warn, error."
  }
}

variable "rust_log" {
  description = "Rust-specific logging configuration."
  type        = string
  default     = "info"
}

#
# CHANGE TYPE LABELS CONFIGURATION
#

variable "change_type_labels_enabled" {
  description = "Whether smart change type label detection is enabled."
  type        = bool
  default     = true
}

# Conventional commit mappings
variable "change_type_labels_feat_mappings" {
  description = "Repository labels to search for 'feat' commits."
  type        = list(string)
  default     = ["enhancement", "feature"]
}

variable "change_type_labels_fix_mappings" {
  description = "Repository labels to search for 'fix' commits."
  type        = list(string)
  default     = ["bug", "bugfix", "fix"]
}

variable "change_type_labels_docs_mappings" {
  description = "Repository labels to search for 'docs' commits."
  type        = list(string)
  default     = ["documentation", "docs"]
}

variable "change_type_labels_style_mappings" {
  description = "Repository labels to search for 'style' commits."
  type        = list(string)
  default     = ["style", "formatting"]
}

variable "change_type_labels_refactor_mappings" {
  description = "Repository labels to search for 'refactor' commits."
  type        = list(string)
  default     = ["refactor", "refactoring", "code quality"]
}

variable "change_type_labels_perf_mappings" {
  description = "Repository labels to search for 'perf' commits."
  type        = list(string)
  default     = ["performance", "optimization"]
}

variable "change_type_labels_test_mappings" {
  description = "Repository labels to search for 'test' commits."
  type        = list(string)
  default     = ["test", "tests", "testing"]
}

variable "change_type_labels_chore_mappings" {
  description = "Repository labels to search for 'chore' commits."
  type        = list(string)
  default     = ["chore", "maintenance", "housekeeping"]
}

variable "change_type_labels_ci_mappings" {
  description = "Repository labels to search for 'ci' commits."
  type        = list(string)
  default     = ["ci", "continuous integration"]
}

variable "change_type_labels_build_mappings" {
  description = "Repository labels to search for 'build' commits."
  type        = list(string)
  default     = ["build", "dependencies"]
}

variable "change_type_labels_revert_mappings" {
  description = "Repository labels to search for 'revert' commits."
  type        = list(string)
  default     = ["revert"]
}

# Fallback label settings
variable "change_type_labels_fallback_name_format" {
  description = "Format for creating fallback labels (uses {change_type} placeholder)."
  type        = string
  default     = "type: {change_type}"
}

variable "change_type_labels_fallback_create_if_missing" {
  description = "Whether to create labels when none exist in the repository."
  type        = bool
  default     = true
}

# Color scheme for fallback labels
variable "change_type_labels_color_feat" {
  description = "Hex color for 'feat' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_fix" {
  description = "Hex color for 'fix' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_docs" {
  description = "Hex color for 'docs' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_style" {
  description = "Hex color for 'style' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_refactor" {
  description = "Hex color for 'refactor' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_perf" {
  description = "Hex color for 'perf' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_test" {
  description = "Hex color for 'test' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_chore" {
  description = "Hex color for 'chore' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_ci" {
  description = "Hex color for 'ci' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_build" {
  description = "Hex color for 'build' fallback labels."
  type        = string
  default     = "#fcc37b"
}

variable "change_type_labels_color_revert" {
  description = "Hex color for 'revert' fallback labels."
  type        = string
  default     = "#fcc37b"
}

# Detection strategy
variable "change_type_labels_detection_exact_match" {
  description = "Enable exact name matching against repository labels."
  type        = bool
  default     = true
}

variable "change_type_labels_detection_prefix_match" {
  description = "Enable prefix matching (e.g., 'type:feat' matches 'feat')."
  type        = bool
  default     = true
}

variable "change_type_labels_detection_description_match" {
  description = "Enable description matching (e.g., label description ending with '(type: feat)')."
  type        = bool
  default     = true
}

variable "change_type_labels_detection_common_prefixes" {
  description = "Prefixes to check during prefix matching."
  type        = list(string)
  default     = ["type:", "kind:", "category:"]
}
