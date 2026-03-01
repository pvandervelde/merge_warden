# Deployment Operations

Comprehensive deployment strategies, procedures, and operational guidelines for Merge Warden across different platforms and environments.

## Overview

This document defines the deployment architecture, procedures, and operational practices for managing Merge Warden deployments across Azure Functions, CLI installations, and future platforms. It covers deployment automation, environment management, rollback procedures, and operational monitoring.

## Deployment Targets

### Azure Functions (Primary Cloud Deployment)

**Architecture:**

- Consumption plan for cost-effective scaling
- Event-driven execution via GitHub webhooks
- Managed identity for secure resource access
- Integration with Azure services (App Configuration, Key Vault)

**Deployment Pipeline:**

1. Code compilation and testing
2. Infrastructure provisioning (Terraform)
3. Function app deployment
4. Configuration and secrets management
5. Integration testing
6. Production rollout

### CLI (Local and CI/CD Deployment)

**Distribution Methods:**

- GitHub Releases with pre-compiled binaries
- Cargo package registry
- Container images for CI/CD environments
- Package managers (future: Homebrew, apt, etc.)

**Installation Options:**

- Direct binary download
- Cargo install from source
- Docker container execution
- CI/CD pipeline integration

## Infrastructure as Code

### Terraform Configuration

**Resource Management:**

```hcl
# Core Azure Functions infrastructure
resource "azurerm_function_app" "merge_warden" {
  name                = var.function_app_name
  location            = var.location
  resource_group_name = var.resource_group_name

  storage_account_name = azurerm_storage_account.main.name
  app_service_plan_id  = azurerm_app_service_plan.main.id

  # Managed identity for secure access
  identity {
    type = "SystemAssigned"
  }

  # Application settings
  app_settings = {
    FUNCTIONS_WORKER_RUNTIME = "Custom"
    APP_CONFIG_ENDPOINT      = azurerm_app_configuration.main.endpoint
  }
}

# Azure App Configuration
resource "azurerm_app_configuration" "main" {
  name                = var.app_config_name
  resource_group_name = var.resource_group_name
  location            = var.location
  sku                 = "free"
}

# Key Vault for secrets
resource "azurerm_key_vault" "main" {
  name                = var.key_vault_name
  location            = var.location
  resource_group_name = var.resource_group_name

  tenant_id = data.azurerm_client_config.current.tenant_id
  sku_name  = "standard"
}
```

**Environment-Specific Configurations:**

- **Development**: Single resource group, basic monitoring
- **Staging**: Production-like setup with reduced capacity
- **Production**: High availability, comprehensive monitoring, backup strategies

### GitOps Workflow

**Branch Strategy:**

- `main`: Production-ready code
- `develop`: Integration branch for new features
- `feature/*`: Feature development branches
- `hotfix/*`: Critical production fixes

**Deployment Triggers:**

- **Development**: Push to `develop` branch
- **Staging**: Tagged pre-releases
- **Production**: Tagged releases

## Deployment Procedures

### Azure Functions Deployment

#### Automated Deployment (Recommended)

**GitHub Actions Workflow:**

```yaml
name: Deploy Azure Functions

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build for Azure Functions
        run: cargo build --release --target x86_64-unknown-linux-musl

      - name: Deploy infrastructure
        run: |
          terraform init
          terraform plan -out=tfplan
          terraform apply tfplan

      - name: Deploy function code
        uses: Azure/functions-action@v1
        with:
          app-name: ${{ vars.AZURE_FUNCTION_APP_NAME }}
          package: target/azure-functions
```

#### Manual Deployment

**Prerequisites:**

- Azure CLI installed and authenticated
- Terraform installed
- Function app infrastructure deployed

**Steps:**

1. Build the application: `cargo build --release --target x86_64-unknown-linux-musl`
2. Package for Azure Functions: `./scripts/package-azure-functions.sh`
3. Deploy function code: `func azure functionapp publish $FUNCTION_APP_NAME`
4. Verify deployment: `./scripts/test-deployment.sh`

### CLI Deployment

#### Release Process

**Automated Release:**

```yaml
name: Release CLI

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-gnu
          - x86_64-apple-darwin
          - x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}
    steps:
      - name: Build binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Create release package
        run: ./scripts/package-release.sh ${{ matrix.target }}

      - name: Upload to GitHub Releases
        uses: softprops/action-gh-release@v1
        with:
          files: releases/*
```

#### Distribution Channels

**GitHub Releases:**

- Pre-compiled binaries for major platforms
- SHA256 checksums for verification
- Release notes with changelog
- Installation instructions

**Cargo Registry:**

- Source distribution for Rust users
- Dependency management through Cargo
- Version compatibility information

## Environment Management

### Development Environment

**Purpose:** Local development and testing
**Configuration:**

- Local configuration files
- Mock GitHub API responses
- Debug logging enabled
- Fast iteration cycles

**Setup:**

```bash
# Clone repository
git clone https://github.com/pvandervelde/merge_warden.git
cd merge_warden

# Setup development environment
./scripts/setup-dev-env.sh

# Run local tests
cargo test

# Start local development server
cargo run --bin merge-warden-cli
```

### Staging Environment

**Purpose:** Pre-production testing and validation
**Configuration:**

- Production-like Azure infrastructure
- Real GitHub integration with test repositories
- Performance monitoring
- Automated testing

**Deployment:**

- Triggered by pre-release tags
- Full integration testing
- Performance benchmarking
- Security scanning

### Production Environment

**Purpose:** Live system serving real repositories
**Configuration:**

- High-availability Azure infrastructure
- Comprehensive monitoring and alerting
- Backup and disaster recovery
- Security hardening

**Deployment:**

- Triggered by release tags
- Blue-green deployment strategy
- Automated rollback capabilities
- Post-deployment verification

## Rollback Procedures

### Azure Functions Rollback

**Automatic Rollback:**

- Health check failures trigger automatic rollback
- Previous deployment slot activation
- Traffic routing back to stable version

**Manual Rollback:**

```bash
# List available deployment slots
az functionapp deployment slot list --name $FUNCTION_APP_NAME --resource-group $RESOURCE_GROUP

# Swap back to previous stable slot
az functionapp deployment slot swap --name $FUNCTION_APP_NAME --resource-group $RESOURCE_GROUP --slot staging --target-slot production

# Verify rollback success
./scripts/verify-deployment.sh
```

### CLI Rollback

**GitHub Releases:**

- Previous versions remain available
- Users can downgrade manually
- Clear versioning and compatibility information

**Emergency Procedures:**

- Yank problematic versions from cargo registry
- Publish hotfix releases
- Communicate issues through GitHub and documentation

## Monitoring and Observability

### Health Checks

**Azure Functions:**

- Function execution success rates
- Response time monitoring
- Error rate tracking
- Resource utilization

**CLI:**

- Installation success metrics
- Usage analytics (opt-in)
- Error reporting
- Performance benchmarks

### Alerting

**Critical Alerts:**

- Function execution failures
- Authentication failures
- High error rates
- Resource exhaustion

**Warning Alerts:**

- Performance degradation
- Configuration issues
- Dependency failures
- Capacity concerns

### Dashboards

**Operational Dashboard:**

- Real-time system health
- Request volume and latency
- Error rates and types
- Resource utilization

**Business Dashboard:**

- Repository usage statistics
- Policy enforcement metrics
- User adoption trends
- Feature usage analytics

## Security Operations

### Access Management

**Production Access:**

- Role-based access control
- Multi-factor authentication required
- Audit logging for all access
- Regular access reviews

**Deployment Permissions:**

- Automated deployments use service principals
- Human deployments require approval
- Separation of duties for critical operations

### Secret Management

**Azure Key Vault:**

- GitHub App credentials
- Webhook secrets
- API keys and tokens
- Certificate management

**Rotation Procedures:**

- Automated secret rotation where possible
- Manual rotation procedures documented
- Emergency rotation capabilities
- Impact assessment for rotation

### Security Monitoring

**Threat Detection:**

- Unusual access patterns
- Failed authentication attempts
- Unauthorized configuration changes
- Suspicious API usage

**Vulnerability Management:**

- Regular dependency scanning
- Security patch management
- Penetration testing schedule
- Incident response procedures

## Disaster Recovery

### Backup Strategy

**Configuration Backup:**

- Azure App Configuration export
- Infrastructure as Code repository
- Documentation and runbooks
- Access control settings

**Recovery Procedures:**

- Infrastructure recreation from Terraform
- Configuration restoration
- Service reconnection
- Validation and testing

### Business Continuity

**Service Availability:**

- Multi-region deployment capability
- Graceful degradation strategies
- Fallback mechanisms
- Communication plans

**Recovery Time Objectives:**

- Critical functions: 1 hour RTO
- Full service restoration: 4 hours RTO
- Data recovery: 24 hours RPO

## Performance Optimization

### Scaling Strategies

**Azure Functions:**

- Consumption plan auto-scaling
- Performance monitoring and tuning
- Cold start optimization
- Resource allocation adjustment

**CLI:**

- Binary size optimization
- Startup time optimization
- Memory usage efficiency
- Parallel processing capabilities

### Capacity Planning

**Traffic Analysis:**

- Peak usage patterns
- Growth projections
- Resource utilization trends
- Cost optimization opportunities

**Scaling Policies:**

- Automatic scaling triggers
- Manual scaling procedures
- Resource limit management
- Performance SLA maintenance

## Related Documents

- **[Deployment Architectures](../architecture/deployment-architectures.md)**: Technical deployment architecture
- **[Configuration Management](./configuration-management.md)**: Configuration deployment procedures
- **[Monitoring](./monitoring.md)**: Detailed monitoring specifications
- **[Security](../security/README.md)**: Security deployment considerations
