# Merge Warden Azure Terraform Module

This directory contains a complete Terraform module for deploying Merge Warden to Azure.

## Features

- Azure Function App with Linux Consumption Plan
- Key Vault for secrets management
- App Configuration for centralized settings
- Application Insights for monitoring
- Proper RBAC and access policies
- Support for multiple environments

## Usage

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

## Quick Start

1. **Download artifacts**:

   ```bash
   wget https://github.com/pvandervelde/merge_warden/releases/latest/download/azure-function-package.zip
   ```

2. **Configure variables**:

   ```bash
   cp terraform.tfvars.example terraform.tfvars
   # Edit terraform.tfvars with your values
   ```

3. **Deploy infrastructure**:

   ```bash
   terraform init
   terraform plan
   terraform apply
   ```

4. **Deploy function code**:

   ```bash
   az functionapp deployment source config-zip \
     --resource-group $(terraform output -raw resource_group_name) \
     --name $(terraform output -raw function_app_name) \
     --src azure-function-package.zip
   ```

## Variables

See [variables.tf](variables.tf) for all available configuration options.

## Outputs

See [outputs.tf](outputs.tf) for all available outputs.

## Examples

- [Basic deployment](examples/basic/) - Minimal configuration
- [Multi-environment](examples/multi-environment/) - Dev/staging/prod setup
- [Custom configuration](examples/custom-configuration/) - Advanced settings
