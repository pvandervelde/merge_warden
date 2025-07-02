#
# ENVIRONMENT
#

variable "category" {
  default     = "mergew"
  description = "The name of the category that all the resources are running in."
}

variable "environment" {
  default     = "production"
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
  default     = []
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
  default     = false
}

variable "pr_size_fail_on_oversized" {
  description = "Whether to fail the check for oversized PRs (XXL category)."
  type        = bool
  default     = false
}

variable "pr_size_label_prefix" {
  description = "Label prefix for size labels (defaults to 'size/')."
  type        = string
  default     = "size/"
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
