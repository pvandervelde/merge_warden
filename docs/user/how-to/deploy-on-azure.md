---
title: "How to deploy on Azure Container Apps"
description: "Deploy the Merge Warden server container on Azure Container Apps with Key Vault secrets."
---

# How to deploy on Azure Container Apps

This guide deploys Merge Warden on [Azure Container Apps](https://learn.microsoft.com/en-us/azure/container-apps/)
using Azure Key Vault for secret management.

**Prerequisites:**

- Azure CLI installed and authenticated (`az login`)
- An Azure subscription with Contributor access to a resource group
- A GitHub App with private key — see [GitHub App permissions](../reference/github-app-permissions.md)

---

## 1 — Create infrastructure

```bash
# Resource group
az group create \
  --name rg-merge-warden \
  --location australiaeast

# Container Apps environment
az containerapp env create \
  --name cae-merge-warden \
  --resource-group rg-merge-warden \
  --location australiaeast

# Key Vault
az keyvault create \
  --name kv-merge-warden \
  --resource-group rg-merge-warden \
  --location australiaeast \
  --enable-rbac-authorization true
```

---

## 2 — Store secrets in Key Vault

```bash
az keyvault secret set \
  --vault-name kv-merge-warden \
  --name github-app-id \
  --value "123456"

az keyvault secret set \
  --vault-name kv-merge-warden \
  --name github-app-key \
  --value "$(cat /path/to/private-key.pem)"

az keyvault secret set \
  --vault-name kv-merge-warden \
  --name github-webhook-secret \
  --value "your-webhook-secret"
```

---

## 3 — Create a managed identity and grant Key Vault access

```bash
# Create a user-assigned managed identity
az identity create \
  --name id-merge-warden \
  --resource-group rg-merge-warden

IDENTITY_ID=$(az identity show \
  --name id-merge-warden \
  --resource-group rg-merge-warden \
  --query id -o tsv)

IDENTITY_PRINCIPAL=$(az identity show \
  --name id-merge-warden \
  --resource-group rg-merge-warden \
  --query principalId -o tsv)

KV_ID=$(az keyvault show \
  --name kv-merge-warden \
  --resource-group rg-merge-warden \
  --query id -o tsv)

# Grant the identity permission to read secrets
az role assignment create \
  --assignee "$IDENTITY_PRINCIPAL" \
  --role "Key Vault Secrets User" \
  --scope "$KV_ID"
```

---

## 4 — Deploy the container

Retrieve the Key Vault secret URIs, then deploy:

```bash
APP_ID_URI=$(az keyvault secret show \
  --vault-name kv-merge-warden --name github-app-id --query id -o tsv)

APP_KEY_URI=$(az keyvault secret show \
  --vault-name kv-merge-warden --name github-app-key --query id -o tsv)

WEBHOOK_SECRET_URI=$(az keyvault secret show \
  --vault-name kv-merge-warden --name github-webhook-secret --query id -o tsv)

az containerapp create \
  --name merge-warden \
  --resource-group rg-merge-warden \
  --environment cae-merge-warden \
  --image ghcr.io/pvandervelde/merge-warden-server:latest \
  --target-port 3000 \
  --ingress external \
  --user-assigned "$IDENTITY_ID" \
  --secrets \
    "app-id=keyvaultref:${APP_ID_URI},identityref:${IDENTITY_ID}" \
    "app-key=keyvaultref:${APP_KEY_URI},identityref:${IDENTITY_ID}" \
    "webhook-secret=keyvaultref:${WEBHOOK_SECRET_URI},identityref:${IDENTITY_ID}" \
  --env-vars \
    "MERGE_WARDEN_GITHUB_APP_ID=secretref:app-id" \
    "MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY=secretref:app-key" \
    "GITHUB_WEBHOOK_SECRET=secretref:webhook-secret"
```

---

## 5 — Configure the GitHub App webhook

1. Get the Container Apps ingress URL:

   ```bash
   az containerapp show \
     --name merge-warden \
     --resource-group rg-merge-warden \
     --query properties.configuration.ingress.fqdn -o tsv
   ```

2. In your GitHub App settings, set the **Webhook URL** to:

   ```
   https://<fqdn>/api/merge_warden
   ```

3. Set **Content type** to `application/json`.

---

## 6 — Verify the deployment

```bash
FQDN=$(az containerapp show \
  --name merge-warden \
  --resource-group rg-merge-warden \
  --query properties.configuration.ingress.fqdn -o tsv)

curl https://$FQDN/api/merge_warden
# Expected: HTTP 200 OK
```

---

## Optional — Configure health probes

Azure Container Apps supports HTTP liveness and readiness probes. Add them with
`--health-check` flags or by editing the container app's YAML. Point both probes to
`GET /api/merge_warden` on port `3000`.

---

## Optional — Enable OTLP telemetry

Set the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable to an OpenTelemetry collector
endpoint. With the [Azure Monitor OpenTelemetry Distro](https://learn.microsoft.com/en-us/azure/azure-monitor/app/opentelemetry-enable)
collector sidecar, traces are forwarded to Application Insights without any SDK in the
binary.

```bash
az containerapp update \
  --name merge-warden \
  --resource-group rg-merge-warden \
  --set-env-vars \
    "OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318" \
    "OTEL_SERVICE_NAME=merge-warden"
```

---

## Related

- [Environment variables reference](../reference/environment-variables.md)
- [GitHub App permissions](../reference/github-app-permissions.md)
- [Set application-level policy defaults](set-app-level-defaults.md)
