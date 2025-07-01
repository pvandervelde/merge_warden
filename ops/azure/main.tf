terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 4.0"
    }
  }

  backend "azurerm" {
    resource_group_name  = "p-aue-tf-tfstate-rg"
    storage_account_name = "paueterraformstate"
    container_name       = "p-aue-tf-tfstate-sc"
    key                  = "prod.mergew.tfstate"
  }
}

provider "azurerm" {
  features {}

  subscription_id = "c3420a9b-5638-4c5e-9f3a-b54263bd3662"

}

#
# LOCALS
#

locals {
  location_map = {
    australiacentral   = "auc",
    australiacentral2  = "auc2",
    australiaeast      = "aue",
    australiasoutheast = "ause",
    brazilsouth        = "brs",
    canadacentral      = "cac",
    canadaeast         = "cae",
    centralindia       = "inc",
    centralus          = "usc",
    eastasia           = "ase",
    eastus             = "use",
    eastus2            = "use2",
    francecentral      = "frc",
    francesouth        = "frs",
    germanynorth       = "den",
    germanywestcentral = "dewc",
    japaneast          = "jpe",
    japanwest          = "jpw",
    koreacentral       = "krc",
    koreasouth         = "kre",
    northcentralus     = "usnc",
    northeurope        = "eun",
    norwayeast         = "noe",
    norwaywest         = "now",
    southafricanorth   = "zan",
    southafricawest    = "zaw",
    southcentralus     = "ussc",
    southeastasia      = "asse",
    southindia         = "ins",
    switzerlandnorth   = "chn",
    switzerlandwest    = "chw",
    uaecentral         = "aec",
    uaenorth           = "aen",
    uksouth            = "uks",
    ukwest             = "ukw",
    westcentralus      = "uswc",
    westeurope         = "euw",
    westindia          = "inw",
    westus             = "usw",
    westus2            = "usw2",
  }
}

locals {
  environment_short = substr(var.environment, 0, 1)
  location_short    = lookup(local.location_map, var.location, "aue")
}

# Name prefixes
locals {
  name_prefix    = "${local.environment_short}-${local.location_short}"
  name_prefix_tf = "${local.name_prefix}-tf-${var.category}"
}

locals {
  common_tags = {
    category    = "${var.category}"
    environment = "${var.environment}"
    location    = "${var.location}"
    git_sha     = "${var.meta_git_sha}"
    version     = "${var.meta_version}"
  }

  extra_tags = {
  }
}

#
# Data
#

data "http" "github_meta" {
  url = "https://api.github.com/meta"
}

data "azurerm_client_config" "current" {}

locals {
  github_hook_ips = jsondecode(data.http.github_meta.response_body).hooks
}

#
# Resource group
#

resource "azurerm_resource_group" "rg" {
  location = var.location
  name     = "${local.name_prefix_tf}-rg"

  tags = merge(
    local.common_tags,
    local.extra_tags,
    var.tags,
    {
      "purpose" = "${var.category}"
  })
}

#
# Storage accounts
#

resource "azurerm_storage_account" "sa" {
  name                     = "${local.environment_short}${local.location_short}tf${var.category}sa"
  resource_group_name      = azurerm_resource_group.rg.name
  location                 = azurerm_resource_group.rg.location
  account_tier             = "Standard"
  account_replication_type = "LRS"

  tags = merge(
    local.common_tags,
    local.extra_tags,
    var.tags,
    {
      "purpose" = "${var.category}"
  })
}

#
# Key vault
#

resource "azurerm_key_vault" "kv" {
  name                        = "${local.environment_short}${local.location_short}${var.category}"
  location                    = azurerm_resource_group.rg.location
  resource_group_name         = azurerm_resource_group.rg.name
  enabled_for_disk_encryption = true
  tenant_id                   = data.azurerm_client_config.current.tenant_id
  soft_delete_retention_days  = 7
  purge_protection_enabled    = false

  sku_name = "standard"

  access_policy {
    tenant_id = data.azurerm_client_config.current.tenant_id
    object_id = data.azurerm_client_config.current.object_id

    key_permissions = [
      "Get",
    ]

    secret_permissions = [
      "Get",
      "List",
      "Set",
      "Delete",
      "Purge",
    ]

    storage_permissions = [
      "Get",
    ]
  }

  tags = merge(
    local.common_tags,
    local.extra_tags,
    var.tags,
    {
      "purpose" = "${var.category}"
  })
}

resource "azurerm_key_vault_secret" "github_app_id" {
  name         = "GithubAppId"
  value        = var.github_app_id
  key_vault_id = azurerm_key_vault.kv.id
}

resource "azurerm_key_vault_secret" "github_app_private_key" {
  name         = "GithubAppPrivateKey"
  value        = file(var.github_app_private_key_path)
  key_vault_id = azurerm_key_vault.kv.id
}

resource "azurerm_key_vault_secret" "github_webhook_secret" {
  name         = "GithubWebhookSecret"
  value        = var.github_webhook_secret
  key_vault_id = azurerm_key_vault.kv.id
}

#
# App Insights
#

resource "azurerm_application_insights" "appinsights" {
  name                = "${local.name_prefix}-tf-${var.category}-insights"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  application_type    = "web"
  # Once set the workspace_id cannot be changed. Given that we didn't originally set it, we need
  # to set it to the value that Azure gave it. Otherwise terraform will try to set it to null,
  # which will result in an error.
  # see: https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs/resources/application_insights#workspace_id-1
  workspace_id = "/subscriptions/c3420a9b-5638-4c5e-9f3a-b54263bd3662/resourceGroups/ai_p-aue-tf-mergew-insights_0f848fbd-11b2-44f3-8eea-4dccb8585e69_managed/providers/Microsoft.OperationalInsights/workspaces/managed-p-aue-tf-mergew-insights-ws"

  tags = merge(
    local.common_tags,
    local.extra_tags,
    var.tags
  )
}

#
# Function app
#

resource "azurerm_service_plan" "asp" {
  name                = "${local.name_prefix}-tf-${var.category}-asp"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  os_type             = "Linux"
  sku_name            = "Y1"

  tags = merge(
    local.common_tags,
    local.extra_tags,
  var.tags)
}

resource "azurerm_linux_function_app" "fa" {
  name                = "${local.name_prefix}-tf-${var.category}-function"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name

  service_plan_id            = azurerm_service_plan.asp.id
  storage_account_name       = azurerm_storage_account.sa.name
  storage_account_access_key = azurerm_storage_account.sa.primary_access_key

  identity {
    type = "SystemAssigned"
  }

  app_settings = {
    "FUNCTIONS_WORKER_RUNTIME" = "custom"

    # Infrastructure connection settings
    "KEY_VAULT_NAME"      = azurerm_key_vault.kv.name
    "APP_CONFIG_ENDPOINT" = azurerm_app_configuration.app_config.endpoint

    # App insight environment variables
    "APPINSIGHTS_INSTRUMENTATIONKEY"        = azurerm_application_insights.appinsights.instrumentation_key
    "APPLICATIONINSIGHTS_CONNECTION_STRING" = azurerm_application_insights.appinsights.connection_string
  }

  site_config {
    application_insights_connection_string = azurerm_application_insights.appinsights.connection_string
    application_insights_key               = azurerm_application_insights.appinsights.instrumentation_key
  }

  # site_config {
  #   ip_restriction = concat([
  #       for ip in local.github_hook_ips : {
  #           ip_address = ip
  #           action     = "Allow"
  #           priority = 100
  #           name = "Allow Github"
  #       }
  #   ], [
  #       {
  #           ip_address = "0.0.0.0/0"
  #           action     = "Deny"
  #           priority = 200
  #           name = "Deny All"
  #       }
  #   ])
  # }

  tags = merge(
    local.common_tags,
    local.extra_tags,
  var.tags)
}

resource "azurerm_key_vault_access_policy" "function_app" {
  key_vault_id = azurerm_key_vault.kv.id
  tenant_id    = data.azurerm_client_config.current.tenant_id
  object_id    = azurerm_linux_function_app.fa.identity[0].principal_id

  secret_permissions = [
    "Get",
  ]
}

#
# App Configuration
#

## Azure App Configuration for centralized bypass rule storage
resource "azurerm_app_configuration" "app_config" {
  name                = "${local.name_prefix}-tf-${var.category}-appconfig"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  sku                 = "free"

  tags = merge(
    local.common_tags,
    local.extra_tags,
    var.tags,
    {
      "purpose" = "centralized-configuration"
    }
  )
}

## Configuration keys for bypass rules
resource "azurerm_app_configuration_key" "bypass_title_enabled" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "bypass_rules:title:enabled"
  value                  = var.bypass_rules_title_enabled ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "bypass_title_users" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "bypass_rules:title:users"
  value                  = jsonencode(var.bypass_rules_title_users)
  type                   = "kv"
  content_type           = "application/json"
}

resource "azurerm_app_configuration_key" "bypass_work_item_enabled" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "bypass_rules:work_item:enabled"
  value                  = var.bypass_rules_work_item_enabled ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "bypass_work_item_users" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "bypass_rules:work_item:users"
  value                  = jsonencode(var.bypass_rules_work_item_users)
  type                   = "kv"
  content_type           = "application/json"
}

## Application configuration keys
resource "azurerm_app_configuration_key" "enforce_title_convention" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "application:enforce_title_convention"
  value                  = var.enforce_title_convention ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "require_work_items" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "application:require_work_items"
  value                  = var.require_work_items ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "log_level" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "logging:level"
  value                  = var.log_level
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "rust_log" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "logging:rust_log"
  value                  = var.rust_log
  type                   = "kv"
}

## PR Size checking configuration keys
resource "azurerm_app_configuration_key" "pr_size_enabled" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "pr_size:enabled"
  value                  = var.pr_size_enabled ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_fail_on_oversized" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "pr_size:fail_on_oversized"
  value                  = var.pr_size_fail_on_oversized ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_label_prefix" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "pr_size:label_prefix"
  value                  = var.pr_size_label_prefix
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_add_comment" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "pr_size:add_comment"
  value                  = var.pr_size_add_comment ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_excluded_file_patterns" {
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "pr_size:excluded_file_patterns"
  value                  = jsonencode(var.pr_size_excluded_file_patterns)
  type                   = "kv"
  content_type           = "application/json"
}

# Size thresholds (only create if values are provided)
resource "azurerm_app_configuration_key" "pr_size_small_threshold" {
  count                  = var.pr_size_small_threshold != null ? 1 : 0
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "pr_size:thresholds:small"
  value                  = tostring(var.pr_size_small_threshold)
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_medium_threshold" {
  count                  = var.pr_size_medium_threshold != null ? 1 : 0
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "pr_size:thresholds:medium"
  value                  = tostring(var.pr_size_medium_threshold)
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_large_threshold" {
  count                  = var.pr_size_large_threshold != null ? 1 : 0
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "pr_size:thresholds:large"
  value                  = tostring(var.pr_size_large_threshold)
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_extra_large_threshold" {
  count                  = var.pr_size_extra_large_threshold != null ? 1 : 0
  configuration_store_id = azurerm_app_configuration.app_config.id
  key                    = "pr_size:thresholds:extra_large"
  value                  = tostring(var.pr_size_extra_large_threshold)
  type                   = "kv"
}

## Grant Function App access to App Configuration
resource "azurerm_role_assignment" "function_app_appconfig_reader" {
  scope                = azurerm_app_configuration.app_config.id
  role_definition_name = "App Configuration Data Reader"
  principal_id         = azurerm_linux_function_app.fa.identity[0].principal_id
}
