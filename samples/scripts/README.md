# Helper Scripts for Merge Warden Deployment

This directory contains helpful scripts for managing Merge Warden deployments.

## Scripts

- [`deploy.sh`](deploy.sh) - Complete deployment script for new environments
- [`update-function.sh`](update-function.sh) - Update function code from releases
- [`configure-webhook.sh`](configure-webhook.sh) - Configure GitHub repository webhooks
- [`validate-deployment.sh`](validate-deployment.sh) - Validate deployment health

## Usage

All scripts support help output with the `-h` or `--help` flag.

### Deploy New Environment

```bash
./deploy.sh -e prod -l australiaeast -r myorg-merge-warden
```

### Update Function Code

```bash
./update-function.sh -e prod -v v1.2.0
```

### Configure Repository Webhook

```bash
./configure-webhook.sh -o myorg -r myrepo -u https://myfunction.azurewebsites.net/api/merge_warden
```

## Prerequisites

- Azure CLI installed and logged in
- Terraform installed (>= 1.5)
- GitHub CLI installed (for webhook configuration)
- `jq` for JSON processing
- `curl` for API calls

## Configuration

Scripts look for configuration in the following order:

1. Command line arguments
2. Environment variables
3. Configuration files (`.env`, `config.sh`)
4. Interactive prompts
