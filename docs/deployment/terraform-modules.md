# Terraform Module Guide

This guide explains how to create and use Terraform modules for deploying Merge Warden infrastructure.

## Module Structure

A recommended Terraform module structure for Merge Warden deployment:

```
terraform-merge-warden/
├── README.md
├── main.tf                 # Main resource definitions
├── variables.tf           # Input variables
├── outputs.tf             # Output values
├── versions.tf            # Provider version constraints
├── modules/
│   ├── function-app/      # Function App module
│   ├── key-vault/         # Key Vault module
│   └── app-config/        # App Configuration module
└── examples/
    ├── basic/             # Basic deployment example
    ├── multi-env/         # Multi-environment example
    └── custom-config/     # Custom configuration example
```

## Core Module Variables

### Required Variables

```hcl
variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
}

variable "location" {
  description = "Azure region for resources"
  type        = string
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
```

### Optional Variables

```hcl
variable "tags" {
  description = "Tags to apply to all resources"
  type        = map(string)
  default     = {}
}

variable "function_app_name" {
  description = "Custom name for the Function App"
  type        = string
  default     = null
}

variable "sku_name" {
  description = "App Service Plan SKU"
  type        = string
  default     = "Y1"  # Consumption plan
}

# Configuration variables
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

variable "pr_size_enabled" {
  description = "Enable PR size labeling"
  type        = bool
  default     = true
}
```

## Module Outputs

Essential outputs for consumers:

```hcl
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

output "key_vault_name" {
  description = "Name of the Key Vault"
  value       = azurerm_key_vault.main.name
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
```

## Provider Configuration

```hcl
# versions.tf
terraform {
  required_version = ">= 1.5"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 4.0"
    }
  }
}

provider "azurerm" {
  features {
    key_vault {
      purge_soft_delete_on_destroy = true
    }
  }
}
```

## Resource Naming Convention

Use consistent naming patterns:

```hcl
locals {
  # Generate short location code
  location_codes = {
    "australiaeast"      = "aue"
    "australiasoutheast" = "ause"
    "eastus"            = "eus"
    "eastus2"           = "eus2"
    "westus2"           = "wus2"
    "westeurope"        = "weu"
    "northeurope"       = "neu"
    # Add more as needed
  }

  location_short = lookup(local.location_codes, var.location, "unk")

  # Naming convention: {env}-{location}-{service}-{resource_type}
  name_prefix = "${substr(var.environment, 0, 1)}-${local.location_short}"

  # Common tags
  common_tags = {
    Environment   = var.environment
    Location      = var.location
    ManagedBy     = "terraform"
    Application   = "merge-warden"
  }
}

# Example resource naming
resource "azurerm_resource_group" "main" {
  name     = "${local.name_prefix}-merge-warden-rg"
  location = var.location
  tags     = merge(local.common_tags, var.tags)
}
```

## Configuration Management

### App Configuration Keys

Structure your configuration keys logically:

```hcl
# Application settings
resource "azurerm_app_configuration_key" "enforce_title_convention" {
  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "application:enforce_title_convention"
  value                  = var.enforce_title_convention ? "true" : "false"
  type                   = "kv"
}

# Bypass rules
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
```

### Complex Configuration

For complex configurations, use local values:

```hcl
locals {
  # PR size thresholds
  pr_size_thresholds = {
    xs           = var.pr_size_xs_threshold
    small        = var.pr_size_small_threshold
    medium       = var.pr_size_medium_threshold
    large        = var.pr_size_large_threshold
    extra_large  = var.pr_size_extra_large_threshold
  }
}

# Create threshold configurations dynamically
resource "azurerm_app_configuration_key" "pr_size_thresholds" {
  for_each = { for k, v in local.pr_size_thresholds : k => v if v != null }

  configuration_store_id = azurerm_app_configuration.main.id
  key                    = "pr_size:thresholds:${each.key}"
  value                  = tostring(each.value)
  type                   = "kv"
}
```

## Security Best Practices

### Key Vault Access

```hcl
# Function App managed identity
resource "azurerm_key_vault_access_policy" "function_app" {
  key_vault_id = azurerm_key_vault.main.id
  tenant_id    = data.azurerm_client_config.current.tenant_id
  object_id    = azurerm_linux_function_app.main.identity[0].principal_id

  secret_permissions = [
    "Get",
  ]
}

# App Configuration access
resource "azurerm_role_assignment" "function_app_appconfig_reader" {
  scope                = azurerm_app_configuration.main.id
  role_definition_name = "App Configuration Data Reader"
  principal_id         = azurerm_linux_function_app.main.identity[0].principal_id
}
```

### Network Security

```hcl
# Optional: IP restrictions for Function App
resource "azurerm_function_app_slot" "main" {
  # ... other configuration

  site_config {
    ip_restriction {
      ip_address = "140.82.112.0/20"  # GitHub webhook IPs
      action     = "Allow"
      priority   = 100
      name       = "Allow GitHub"
    }

    ip_restriction {
      ip_address = "0.0.0.0/0"
      action     = "Deny"
      priority   = 200
      name       = "Deny All"
    }
  }
}
```

## Module Usage Examples

### Basic Usage

```hcl
module "merge_warden" {
  source = "./modules/merge-warden"

  environment               = "prod"
  location                 = "australiaeast"
  github_app_id            = var.github_app_id
  github_app_private_key   = var.github_app_private_key
  github_webhook_secret    = var.github_webhook_secret

  tags = {
    Team    = "platform"
    Project = "merge-warden"
  }
}
```

### Advanced Usage

```hcl
module "merge_warden" {
  source = "./modules/merge-warden"

  environment               = "prod"
  location                 = "australiaeast"
  github_app_id            = var.github_app_id
  github_app_private_key   = var.github_app_private_key
  github_webhook_secret    = var.github_webhook_secret

  # Custom configuration
  enforce_title_convention = true
  require_work_items      = true
  pr_size_enabled         = true
  pr_size_fail_on_oversized = true

  # Custom size thresholds
  pr_size_xs_threshold    = 10
  pr_size_small_threshold = 50
  pr_size_medium_threshold = 100
  pr_size_large_threshold = 250
  pr_size_extra_large_threshold = 500

  # Bypass rules
  bypass_title_enabled = true
  bypass_title_users   = ["dependabot[bot]", "renovate[bot]"]

  tags = {
    Team        = "platform"
    Project     = "merge-warden"
    Environment = "production"
  }
}
```

## Testing Modules

### Validation

```hcl
# versions.tf - Add validation rules
variable "environment" {
  description = "Environment name"
  type        = string

  validation {
    condition = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be one of: dev, staging, prod."
  }
}

variable "location" {
  description = "Azure region"
  type        = string

  validation {
    condition = can(regex("^[a-z]+[a-z0-9]*$", var.location))
    error_message = "Location must be a valid Azure region name."
  }
}
```

### Local Testing

```bash
# Validate configuration
terraform validate

# Plan without applying
terraform plan -var-file="test.tfvars"

# Format code
terraform fmt -recursive

# Security scanning
tfsec .
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Deploy Infrastructure

on:
  push:
    branches: [main]
    paths: ['terraform/**']

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4

    - name: Setup Terraform
      uses: hashicorp/setup-terraform@v3

    - name: Terraform Init
      run: terraform init
      working-directory: terraform

    - name: Terraform Plan
      run: terraform plan -var-file="prod.tfvars"
      working-directory: terraform

    - name: Terraform Apply
      if: github.ref == 'refs/heads/main'
      run: terraform apply -auto-approve -var-file="prod.tfvars"
      working-directory: terraform
```

## Best Practices

1. **Version Pinning**: Always pin provider versions
2. **State Management**: Use remote state storage (Azure Storage Account)
3. **Secrets Management**: Never store secrets in code; use Azure Key Vault
4. **Documentation**: Document all variables and outputs
5. **Validation**: Add input validation for critical variables
6. **Testing**: Test modules in isolation before integration
7. **Tagging**: Implement consistent resource tagging
8. **Naming**: Use consistent naming conventions
9. **Modularity**: Keep modules focused and reusable
10. **Security**: Follow principle of least privilege for access policies
