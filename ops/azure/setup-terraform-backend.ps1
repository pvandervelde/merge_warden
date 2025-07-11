#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Creates Azure resources for Terraform backend state storage

.DESCRIPTION
    This script creates the necessary Azure resources for storing Terraform state:
    - Resource Group
    - Storage Account
    - Storage Container

    The resources are created based on the backend configuration in main.tf

.PARAMETER Location
    Azure region where resources will be created. Default is 'australiaeast'

.PARAMETER SubscriptionId
    Azure subscription ID. If not provided, uses the current subscription

.EXAMPLE
    .\setup-terraform-backend.ps1

.EXAMPLE
    .\setup-terraform-backend.ps1 -Location "eastus" -SubscriptionId "your-subscription-id"
#>

param(
    [string]$Location = "australiaeast",
    [string]$SubscriptionId = "c3420a9b-5638-4c5e-9f3a-b54263bd3662"
)

# Set error action preference to stop on errors
$ErrorActionPreference = "Stop"

# Backend configuration values from main.tf
$ResourceGroupName = "t-aue-tf-tfstate-rg"
$StorageAccountName = "taueterraformstate"
$ContainerName = "t-aue-tf-tfstate-sc"

Write-Host "Setting up Terraform backend infrastructure..." -ForegroundColor Green

try
{
    # Check if Azure CLI is installed
    Write-Host "Checking Azure CLI installation..." -ForegroundColor Yellow
    $azVersion = az version --output tsv --query '"azure-cli"' 2>$null
    if ($LASTEXITCODE -ne 0)
    {
        throw "Azure CLI is not installed or not in PATH. Please install Azure CLI first."
    }
    Write-Host "Azure CLI version: $azVersion" -ForegroundColor Green

    # Set subscription if provided
    if ($SubscriptionId)
    {
        Write-Host "Setting Azure subscription to: $SubscriptionId" -ForegroundColor Yellow
        az account set --subscription $SubscriptionId
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to set subscription. Please check the subscription ID and your access."
        }
    }

    # Get current subscription info
    $currentSub = az account show --query "{id:id,name:name}" --output json | ConvertFrom-Json
    Write-Host "Using subscription: $($currentSub.name) ($($currentSub.id))" -ForegroundColor Green

    # Create resource group
    Write-Host "Creating resource group: $ResourceGroupName" -ForegroundColor Yellow
    az group show --name $ResourceGroupName --output json 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0)
    {
        Write-Host "Resource group already exists" -ForegroundColor Green
    }
    else
    {
        az group create --name $ResourceGroupName --location $Location --output table
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to create resource group"
        }
        Write-Host "Resource group created successfully" -ForegroundColor Green
    }

    # Create storage account
    Write-Host "Creating storage account: $StorageAccountName" -ForegroundColor Yellow
    az storage account show --name $StorageAccountName --resource-group $ResourceGroupName --output json 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0)
    {
        Write-Host "Storage account already exists" -ForegroundColor Green
    }
    else
    {
        az storage account create `
            --name $StorageAccountName `
            --resource-group $ResourceGroupName `
            --location $Location `
            --sku Standard_LRS `
            --kind StorageV2 `
            --access-tier Hot `
            --https-only true `
            --min-tls-version TLS1_2 `
            --output table

        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to create storage account"
        }
        Write-Host "Storage account created successfully" -ForegroundColor Green
    }

    # Get storage account key
    Write-Host "Retrieving storage account key..." -ForegroundColor Yellow
    $storageKey = az storage account keys list `
        --resource-group $ResourceGroupName `
        --account-name $StorageAccountName `
        --query "[0].value" `
        --output tsv

    if ($LASTEXITCODE -ne 0)
    {
        throw "Failed to retrieve storage account key"
    }

    # Create storage container
    Write-Host "Creating storage container: $ContainerName" -ForegroundColor Yellow
    az storage container show `
        --name $ContainerName `
        --account-name $StorageAccountName `
        --account-key $storageKey `
        --output json 2>$null | Out-Null

    if ($LASTEXITCODE -eq 0)
    {
        Write-Host "Storage container already exists" -ForegroundColor Green
    }
    else
    {
        az storage container create `
            --name $ContainerName `
            --account-name $StorageAccountName `
            --account-key $storageKey `
            --public-access off `
            --output table

        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to create storage container"
        }
        Write-Host "Storage container created successfully" -ForegroundColor Green
    }

    # Enable versioning and soft delete for better state management
    Write-Host "Configuring storage account features..." -ForegroundColor Yellow
    az storage account blob-service-properties update `
        --account-name $StorageAccountName `
        --resource-group $ResourceGroupName `
        --enable-versioning true `
        --enable-delete-retention true `
        --delete-retention-days 30 `
        --output none

    if ($LASTEXITCODE -ne 0)
    {
        Write-Warning "Failed to configure blob service properties. This is not critical."
    }
    else
    {
        Write-Host "Storage account features configured successfully" -ForegroundColor Green
    }

    Write-Host "`n✅ Terraform backend setup completed successfully!" -ForegroundColor Green
    Write-Host "`nBackend configuration:" -ForegroundColor Cyan
    Write-Host "  Resource Group:   $ResourceGroupName" -ForegroundColor White
    Write-Host "  Storage Account:  $StorageAccountName" -ForegroundColor White
    Write-Host "  Container:        $ContainerName" -ForegroundColor White
    Write-Host "  Location:         $Location" -ForegroundColor White

    Write-Host "`nYou can now run 'terraform init' to initialize your Terraform configuration." -ForegroundColor Yellow
}
catch
{
    Write-Error "❌ Setup failed: $($_.Exception.Message)"
    exit 1
}
