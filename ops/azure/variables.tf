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
