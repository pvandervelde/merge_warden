data "azurerm_client_config" "current" {}

# Generate location short codes for naming
locals {
  location_codes = {
    "australiacentral"   = "auc"
    "australiacentral2"  = "auc2"
    "australiaeast"      = "aue"
    "australiasoutheast" = "ause"
    "brazilsouth"        = "brs"
    "canadacentral"      = "cac"
    "canadaeast"         = "cae"
    "centralindia"       = "inc"
    "centralus"          = "usc"
    "eastasia"           = "ase"
    "eastus"             = "use"
    "eastus2"            = "use2"
    "francecentral"      = "frc"
    "francesouth"        = "frs"
    "germanynorth"       = "den"
    "germanywestcentral" = "dewc"
    "japaneast"          = "jpe"
    "japanwest"          = "jpw"
    "koreacentral"       = "krc"
    "koreasouth"         = "kre"
    "northcentralus"     = "usnc"
    "northeurope"        = "eun"
    "norwayeast"         = "noe"
    "norwaywest"         = "now"
    "southafricanorth"   = "zan"
    "southafricawest"    = "zaw"
    "southcentralus"     = "ussc"
    "southeastasia"      = "asse"
    "southindia"         = "ins"
    "switzerlandnorth"   = "chn"
    "switzerlandwest"    = "chw"
    "uaecentral"         = "aec"
    "uaenorth"           = "aen"
    "uksouth"            = "uks"
    "ukwest"             = "ukw"
    "westcentralus"      = "uswc"
    "westeurope"         = "euw"
    "westindia"          = "inw"
    "westus"             = "usw"
    "westus2"            = "usw2"
  }

  location_short = lookup(local.location_codes, var.location, "unk")
  env_short      = substr(var.environment, 0, 1)

  # Resource naming
  name_prefix = var.resource_prefix != "" ? var.resource_prefix : "${local.env_short}-${local.location_short}"
  app_name    = "merge-warden"

  # Common tags
  common_tags = {
    Environment = var.environment
    Location    = var.location
    ManagedBy   = "terraform"
    Application = local.app_name
  }
}

#
# RESOURCE GROUP
#

resource "azurerm_resource_group" "main" {
  name     = "${local.name_prefix}-${local.app_name}-rg"
  location = var.location

  tags = merge(local.common_tags, var.tags)
}

#
# STORAGE ACCOUNT
#

resource "azurerm_storage_account" "main" {
  name                     = "${local.env_short}${local.location_short}${replace(local.app_name, "-", "")}sa"
  resource_group_name      = azurerm_resource_group.main.name
  location                 = azurerm_resource_group.main.location
  account_tier             = "Standard"
  account_replication_type = "LRS"

  tags = merge(local.common_tags, var.tags)
}

#
# APPLICATION INSIGHTS
#

resource "azurerm_application_insights" "main" {
  name                = "${local.name_prefix}-${local.app_name}-insights"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  application_type    = "web"

  tags = merge(local.common_tags, var.tags)
}

#
# KEY VAULT
#

resource "azurerm_key_vault" "main" {
  name                        = "${local.env_short}${local.location_short}${replace(local.app_name, "-", "")}kv"
  location                    = azurerm_resource_group.main.location
  resource_group_name         = azurerm_resource_group.main.name
  enabled_for_disk_encryption = true
  tenant_id                   = data.azurerm_client_config.current.tenant_id
  soft_delete_retention_days  = 7
  purge_protection_enabled    = false
  sku_name                    = "standard"

  # Access policy for current user/service principal
  access_policy {
    tenant_id = data.azurerm_client_config.current.tenant_id
    object_id = data.azurerm_client_config.current.object_id

    key_permissions = ["Get"]
    secret_permissions = [
      "Get", "List", "Set", "Delete", "Purge"
    ]
    storage_permissions = ["Get"]
  }

  tags = merge(local.common_tags, var.tags)
}

# Store GitHub App secrets
resource "azurerm_key_vault_secret" "github_app_id" {
  name         = "GithubAppId"
  value        = var.github_app_id
  key_vault_id = azurerm_key_vault.main.id
}

resource "azurerm_key_vault_secret" "github_app_private_key" {
  name         = "GithubAppPrivateKey"
  value        = var.github_app_private_key
  key_vault_id = azurerm_key_vault.main.id
}

resource "azurerm_key_vault_secret" "github_webhook_secret" {
  name         = "GithubWebhookSecret"
  value        = var.github_webhook_secret
  key_vault_id = azurerm_key_vault.main.id
}

#
# APP CONFIGURATION
#

resource "azurerm_app_configuration" "main" {
  name                = "${local.name_prefix}-${local.app_name}-appconfig"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  sku                 = "free"

  tags = merge(local.common_tags, var.tags)
}

# Application configuration keys
resource "azurerm_app_configuration_key" "enforce_title_convention" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "application:enforce_title_convention"
  value                  = var.enforce_title_convention ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "require_work_items" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "application:require_work_items"
  value                  = var.require_work_items ? "true" : "false"
  type                   = "kv"
}

# Logging configuration
resource "azurerm_app_configuration_key" "log_level" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "logging:level"
  value                  = var.log_level
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "rust_log" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "logging:rust_log"
  value                  = var.rust_log
  type                   = "kv"
}

# Bypass rules configuration
resource "azurerm_app_configuration_key" "bypass_title_enabled" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "bypass_rules:title:enabled"
  value                  = var.bypass_title_enabled ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "bypass_title_users" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "bypass_rules:title:users"
  value                  = jsonencode(var.bypass_title_users)
  type                   = "kv"
  content_type           = "application/json"
}

resource "azurerm_app_configuration_key" "bypass_work_item_enabled" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "bypass_rules:work_item:enabled"
  value                  = var.bypass_work_item_enabled ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "bypass_work_item_users" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "bypass_rules:work_item:users"
  value                  = jsonencode(var.bypass_work_item_users)
  type                   = "kv"
  content_type           = "application/json"
}

# PR size configuration
resource "azurerm_app_configuration_key" "pr_size_enabled" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "pr_size:enabled"
  value                  = var.pr_size_enabled ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_fail_on_oversized" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "pr_size:fail_on_oversized"
  value                  = var.pr_size_fail_on_oversized ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_label_prefix" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "pr_size:label_prefix"
  value                  = var.pr_size_label_prefix
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_add_comment" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "pr_size:add_comment"
  value                  = var.pr_size_add_comment ? "true" : "false"
  type                   = "kv"
}

resource "azurerm_app_configuration_key" "pr_size_excluded_file_patterns" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "pr_size:excluded_file_patterns"
  value                  = jsonencode(var.pr_size_excluded_file_patterns)
  type                   = "kv"
  content_type           = "application/json"
}

# PR size thresholds
locals {
  pr_size_thresholds = {
    xs          = var.pr_size_xs_threshold
    small       = var.pr_size_small_threshold
    medium      = var.pr_size_medium_threshold
    large       = var.pr_size_large_threshold
    extra_large = var.pr_size_extra_large_threshold
  }
}

resource "azurerm_app_configuration_key" "pr_size_thresholds" {
  for_each = { for k, v in local.pr_size_thresholds : k => v if v != null }

  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "pr_size:thresholds:${each.key}"
  value                  = tostring(each.value)
  type                   = "kv"
}

#
# FUNCTION APP
#

# App Service Plan
resource "azurerm_service_plan" "main" {
  name                = "${local.name_prefix}-${local.app_name}-asp"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  os_type             = "Linux"
  sku_name            = "Y1" # Consumption plan

  tags = merge(local.common_tags, var.tags)
}

# Linux Function App
resource "azurerm_linux_function_app" "main" {
  name                = "${local.name_prefix}-${local.app_name}-function"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name

  service_plan_id            = azurerm_service_plan.main.id
  storage_account_name       = azurerm_storage_account.main.name
  storage_account_access_key = azurerm_storage_account.main.primary_access_key

  identity {
    type = "SystemAssigned"
  }

  app_settings = {
    "FUNCTIONS_WORKER_RUNTIME" = "custom"

    # Infrastructure connection settings
    "KEY_VAULT_NAME"      = azurerm_key_vault.main.name
    "APP_CONFIG_ENDPOINT" = azurerm_app_configuration.main.endpoint

    # Application Insights
    "APPINSIGHTS_INSTRUMENTATIONKEY"        = azurerm_application_insights.main.instrumentation_key
    "APPLICATIONINSIGHTS_CONNECTION_STRING" = azurerm_application_insights.main.connection_string
  }

  site_config {
    application_insights_connection_string = azurerm_application_insights.main.connection_string
    application_insights_key               = azurerm_application_insights.main.instrumentation_key
  }

  tags = merge(local.common_tags, var.tags)
}

# Function App access to Key Vault
resource "azurerm_key_vault_access_policy" "function_app" {
  key_vault_id = azurerm_key_vault.main.id
  tenant_id    = data.azurerm_client_config.current.tenant_id
  object_id    = azurerm_linux_function_app.main.identity[0].principal_id

  secret_permissions = ["Get"]
}

# Function App access to App Configuration
resource "azurerm_role_assignment" "function_app_appconfig_reader" {
  scope                = azurerm_app_configuration.main.id
  role_definition_name = "App Configuration Data Reader"
  principal_id         = azurerm_linux_function_app.main.identity[0].principal_id
}
