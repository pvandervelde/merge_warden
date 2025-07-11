#
# REQUIRED VARIABLES
#

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string

  validation {
    condition     = can(regex("^[a-z0-9-]+$", var.environment))
    error_message = "Environment must contain only lowercase letters, numbers, and hyphens."
  }
}

variable "location" {
  description = "Azure region for resources"
  type        = string
  default     = "australiaeast"
}

variable "github_app_id" {
  description = "GitHub App ID"
  type        = string
}

variable "github_app_private_key" {
  description = "GitHub App private key content"
  type        = string
  sensitive   = true
}

variable "github_webhook_secret" {
  description = "GitHub webhook secret"
  type        = string
  sensitive   = true
}

#
# OPTIONAL VARIABLES
#

variable "tags" {
  description = "Tags to apply to all resources"
  type        = map(string)
  default     = {}
}

variable "resource_prefix" {
  description = "Prefix for resource names"
  type        = string
  default     = ""
}

#
# APPLICATION CONFIGURATION
#

variable "enforce_title_convention" {
  description = "Enable PR title validation"
  type        = bool
  default     = true
}

variable "require_work_items" {
  description = "Require work item references in PRs"
  type        = bool
  default     = true
}

#
# PR SIZE CONFIGURATION
#

variable "pr_size_enabled" {
  description = "Enable PR size labeling"
  type        = bool
  default     = true
}

variable "pr_size_fail_on_oversized" {
  description = "Fail checks for oversized PRs"
  type        = bool
  default     = false
}

variable "pr_size_label_prefix" {
  description = "Prefix for PR size labels"
  type        = string
  default     = "size/"
}

variable "pr_size_add_comment" {
  description = "Add educational comments for oversized PRs"
  type        = bool
  default     = true
}

variable "pr_size_excluded_file_patterns" {
  description = "File patterns to exclude from size calculations"
  type        = list(string)
  default     = ["*.md", "*.txt", "docs/*"]
}

# Size thresholds
variable "pr_size_xs_threshold" {
  description = "Extra small PR threshold (lines)"
  type        = number
  default     = 10
}

variable "pr_size_small_threshold" {
  description = "Small PR threshold (lines)"
  type        = number
  default     = 50
}

variable "pr_size_medium_threshold" {
  description = "Medium PR threshold (lines)"
  type        = number
  default     = 100
}

variable "pr_size_large_threshold" {
  description = "Large PR threshold (lines)"
  type        = number
  default     = 250
}

variable "pr_size_extra_large_threshold" {
  description = "Extra large PR threshold (lines)"
  type        = number
  default     = 500
}

#
# BYPASS RULES
#

variable "bypass_title_enabled" {
  description = "Enable title validation bypass rules"
  type        = bool
  default     = true
}

variable "bypass_title_users" {
  description = "Users who can bypass title validation"
  type        = list(string)
  default     = ["dependabot[bot]", "renovate[bot]"]
}

variable "bypass_work_item_enabled" {
  description = "Enable work item validation bypass rules"
  type        = bool
  default     = true
}

variable "bypass_work_item_users" {
  description = "Users who can bypass work item validation"
  type        = list(string)
  default     = ["dependabot[bot]", "renovate[bot]"]
}

#
# LOGGING
#

variable "log_level" {
  description = "Application log level"
  type        = string
  default     = "info"

  validation {
    condition     = contains(["trace", "debug", "info", "warn", "error"], var.log_level)
    error_message = "Log level must be one of: trace, debug, info, warn, error."
  }
}

variable "rust_log" {
  description = "Rust-specific logging configuration"
  type        = string
  default     = "info,merge_warden=debug"
}
