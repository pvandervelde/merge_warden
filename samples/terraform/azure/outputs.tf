output "resource_group_name" {
  description = "Name of the created resource group"
  value       = azurerm_resource_group.main.name
}

output "function_app_name" {
  description = "Name of the Function App"
  value       = azurerm_linux_function_app.main.name
}

output "function_url" {
  description = "Function trigger URL for GitHub webhook"
  value       = "https://${azurerm_linux_function_app.main.default_hostname}/api/merge_warden"
}

output "function_app_hostname" {
  description = "Function App hostname"
  value       = azurerm_linux_function_app.main.default_hostname
}

output "key_vault_name" {
  description = "Name of the Key Vault"
  value       = azurerm_key_vault.main.name
}

output "key_vault_uri" {
  description = "URI of the Key Vault"
  value       = azurerm_key_vault.main.vault_uri
}

output "app_config_endpoint" {
  description = "App Configuration endpoint"
  value       = azurerm_app_configuration.main.endpoint
}

output "application_insights_instrumentation_key" {
  description = "Application Insights instrumentation key"
  value       = azurerm_application_insights.main.instrumentation_key
  sensitive   = true
}

output "application_insights_connection_string" {
  description = "Application Insights connection string"
  value       = azurerm_application_insights.main.connection_string
  sensitive   = true
}

output "storage_account_name" {
  description = "Name of the storage account"
  value       = azurerm_storage_account.main.name
}

# Deployment information
output "deployment_info" {
  description = "Information needed for function deployment"
  value = {
    resource_group_name = azurerm_resource_group.main.name
    function_app_name   = azurerm_linux_function_app.main.name
    function_url        = "https://${azurerm_linux_function_app.main.default_hostname}/api/merge_warden"
  }
}
