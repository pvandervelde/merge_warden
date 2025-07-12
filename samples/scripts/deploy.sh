#!/bin/bash

# deploy.sh - Complete deployment script for Merge Warden
# Usage: ./deploy.sh -e <environment> -l <location> [-r <resource-prefix>] [-t <tag>]

set -euo pipefail

# Default values
ENVIRONMENT=""
LOCATION="australiaeast"
RESOURCE_PREFIX=""
RELEASE_TAG="latest"
TERRAFORM_DIR="terraform"
DRY_RUN=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

usage() {
    cat << EOF
Usage: $0 -e <environment> [-l <location>] [-r <resource-prefix>] [-t <release-tag>] [--dry-run]

Options:
    -e, --environment     Environment name (dev, staging, prod)
    -l, --location        Azure region (default: australiaeast)
    -r, --resource-prefix Custom resource prefix
    -t, --tag             Release tag to deploy (default: latest)
    --dry-run             Show what would be done without executing
    -h, --help            Show this help message

Examples:
    $0 -e prod -l australiaeast
    $0 -e dev -r myorg --dry-run
    $0 -e staging -t v1.2.0

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -l|--location)
            LOCATION="$2"
            shift 2
            ;;
        -r|--resource-prefix)
            RESOURCE_PREFIX="$2"
            shift 2
            ;;
        -t|--tag)
            RELEASE_TAG="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$ENVIRONMENT" ]]; then
    log_error "Environment is required (-e/--environment)"
    usage
    exit 1
fi

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(dev|staging|prod)$ ]]; then
    log_error "Environment must be one of: dev, staging, prod"
    exit 1
fi

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    local missing_tools=()

    if ! command -v az &> /dev/null; then
        missing_tools+=("azure-cli")
    fi

    if ! command -v terraform &> /dev/null; then
        missing_tools+=("terraform")
    fi

    if ! command -v jq &> /dev/null; then
        missing_tools+=("jq")
    fi

    if ! command -v curl &> /dev/null; then
        missing_tools+=("curl")
    fi

    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi

    # Check Azure login
    if ! az account show &> /dev/null; then
        log_error "Not logged into Azure. Run 'az login' first."
        exit 1
    fi

    log_success "All prerequisites met"
}

# Download release artifacts
download_artifacts() {
    log_info "Downloading release artifacts..."

    local download_url
    if [[ "$RELEASE_TAG" == "latest" ]]; then
        download_url="https://github.com/pvandervelde/merge_warden/releases/latest/download"
    else
        download_url="https://github.com/pvandervelde/merge_warden/releases/download/$RELEASE_TAG"
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would download from $download_url"
        return
    fi

    # Create temp directory
    local temp_dir
    temp_dir=$(mktemp -d)
    cd "$temp_dir"

    # Download function package
    if ! curl -L -o azure-function-package.zip "$download_url/azure-function-package.zip"; then
        log_error "Failed to download azure-function-package.zip"
        exit 1
    fi

    # Download and verify checksum
    if ! curl -L -o azure-function-package.zip.sha256 "$download_url/azure-function-package.zip.sha256"; then
        log_error "Failed to download checksum file"
        exit 1
    fi

    # Verify integrity
    if ! sha256sum -c azure-function-package.zip.sha256; then
        log_error "Checksum verification failed"
        exit 1
    fi

    # Move to working directory
    mv azure-function-package.zip "$OLDPWD/"
    cd "$OLDPWD"
    rm -rf "$temp_dir"

    log_success "Downloaded and verified release artifacts"
}

# Deploy infrastructure
deploy_infrastructure() {
    log_info "Deploying infrastructure..."

    if [[ ! -d "$TERRAFORM_DIR" ]]; then
        log_error "Terraform directory not found: $TERRAFORM_DIR"
        exit 1
    fi

    cd "$TERRAFORM_DIR"

    # Initialize terraform
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would run terraform init"
    else
        terraform init
    fi

    # Prepare terraform variables
    local tf_vars=()
    tf_vars+=("-var=environment=$ENVIRONMENT")
    tf_vars+=("-var=location=$LOCATION")

    if [[ -n "$RESOURCE_PREFIX" ]]; then
        tf_vars+=("-var=resource_prefix=$RESOURCE_PREFIX")
    fi

    # Add environment-specific vars file if it exists
    if [[ -f "environments/$ENVIRONMENT.tfvars" ]]; then
        tf_vars+=("-var-file=environments/$ENVIRONMENT.tfvars")
    fi

    # Plan
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would run terraform plan with: ${tf_vars[*]}"
    else
        terraform plan "${tf_vars[@]}" -out=deployment.tfplan
    fi

    # Apply
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would run terraform apply"
    else
        terraform apply -auto-approve deployment.tfplan

        # Get outputs
        RESOURCE_GROUP_NAME=$(terraform output -raw resource_group_name)
        FUNCTION_APP_NAME=$(terraform output -raw function_app_name)
        FUNCTION_URL=$(terraform output -raw function_url)

        log_success "Infrastructure deployed successfully"
        log_info "Resource Group: $RESOURCE_GROUP_NAME"
        log_info "Function App: $FUNCTION_APP_NAME"
        log_info "Function URL: $FUNCTION_URL"
    fi

    cd "$OLDPWD"
}

# Deploy function code
deploy_function() {
    log_info "Deploying function code..."

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: Would deploy azure-function-package.zip to $FUNCTION_APP_NAME"
        return
    fi

    if [[ ! -f "azure-function-package.zip" ]]; then
        log_error "Function package not found: azure-function-package.zip"
        exit 1
    fi

    # Deploy the package
    az functionapp deployment source config-zip \
        --resource-group "$RESOURCE_GROUP_NAME" \
        --name "$FUNCTION_APP_NAME" \
        --src azure-function-package.zip

    # Wait for deployment
    log_info "Waiting for deployment to complete..."
    sleep 30

    # Verify deployment
    local state
    state=$(az functionapp show \
        --resource-group "$RESOURCE_GROUP_NAME" \
        --name "$FUNCTION_APP_NAME" \
        --query "state" -o tsv)

    if [[ "$state" == "Running" ]]; then
        log_success "Function deployed and running"
    else
        log_warning "Function state: $state"
    fi
}

# Main execution
main() {
    log_info "Starting Merge Warden deployment"
    log_info "Environment: $ENVIRONMENT"
    log_info "Location: $LOCATION"
    log_info "Release: $RELEASE_TAG"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_warning "DRY RUN MODE - No changes will be made"
    fi

    check_prerequisites
    download_artifacts
    deploy_infrastructure

    if [[ "$DRY_RUN" != "true" ]]; then
        deploy_function

        echo
        log_success "Deployment completed successfully!"
        echo
        echo "Next steps:"
        echo "1. Configure GitHub webhook URL: $FUNCTION_URL"
        echo "2. Add repository configuration: .github/merge-warden.toml"
        echo "3. Test the webhook with a test PR"
    fi
}

# Run main function
main "$@"
