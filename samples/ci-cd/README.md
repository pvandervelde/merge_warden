# Sample CI/CD Workflows for Infrastructure Repository

This directory contains sample GitHub Actions workflows for managing Merge Warden infrastructure deployment.

## Workflows

- [`terraform-plan.yml`](terraform-plan.yml) - Runs terraform plan on pull requests
- [`terraform-apply.yml`](terraform-apply.yml) - Deploys infrastructure on merge to main
- [`function-deploy.yml`](function-deploy.yml) - Deploys function code from releases

## Setup

1. **Repository Secrets**: Configure the following secrets in your repository:
   - `ARM_CLIENT_ID` - Azure service principal client ID
   - `ARM_CLIENT_SECRET` - Azure service principal client secret
   - `ARM_TENANT_ID` - Azure tenant ID
   - `ARM_SUBSCRIPTION_ID` - Azure subscription ID
   - `GITHUB_APP_ID` - Your GitHub App ID
   - `GITHUB_APP_PRIVATE_KEY` - Your GitHub App private key
   - `GITHUB_WEBHOOK_SECRET` - Your GitHub webhook secret

2. **Terraform Backend**: Configure remote state storage in `backend.tf`:

   ```hcl
   terraform {
     backend "azurerm" {
       resource_group_name  = "terraform-state-rg"
       storage_account_name = "terraformstate123"
       container_name       = "tfstate"
       key                  = "merge-warden.tfstate"
     }
   }
   ```

3. **Environment Files**: Create environment-specific `.tfvars` files:
   - `environments/dev.tfvars`
   - `environments/staging.tfvars`
   - `environments/prod.tfvars`

## Usage

1. **Development Workflow**:
   - Create feature branch
   - Modify terraform configuration
   - Open pull request → triggers `terraform-plan.yml`
   - Review plan output in PR comments
   - Merge to main → triggers `terraform-apply.yml`

2. **Function Deployment**:
   - New release created in merge_warden repository
   - Manually trigger `function-deploy.yml` workflow
   - Or set up automatic deployment on release

## Security Considerations

- Use Azure service principal with minimal required permissions
- Store all secrets in GitHub repository secrets
- Review terraform plans before applying
- Use branch protection rules for main branch
