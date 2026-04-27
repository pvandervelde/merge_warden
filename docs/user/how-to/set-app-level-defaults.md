---
title: "How to set application-level policy defaults"
description: "Configure default policies that apply to all repositories processed by a Merge Warden server instance."
---

# How to set application-level policy defaults

Merge Warden supports two configuration files with different scopes:

| File | Scope | Who controls it |
| :--- | :--- | :--- |
| `MERGE_WARDEN_CONFIG_FILE` | All repositories on this server instance | Operator (platform team) |
| `.github/merge-warden.toml` | One specific repository | Repository maintainer |

This guide covers the application-level file. For the per-repository file, see
[Per-repository configuration schema](../reference/per-repo-config.md).

---

## Creating the application config file

Copy `samples/app-config.sample.toml` as a starting point:

```bash
cp samples/app-config.sample.toml /etc/merge-warden/app-config.toml
```

Edit it to match your organisation's defaults. The full schema is documented in
[Application configuration schema](../reference/app-config.md).

---

## Supplying the file to the server

### Docker

Mount the file into the container and set `MERGE_WARDEN_CONFIG_FILE`:

```bash
docker run --rm \
  -e MERGE_WARDEN_GITHUB_APP_ID=12345 \
  -e MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY="$(cat private-key.pem)" \
  -e GITHUB_WEBHOOK_SECRET=supersecret \
  -e MERGE_WARDEN_CONFIG_FILE=/etc/merge-warden/app-config.toml \
  -v /etc/merge-warden:/etc/merge-warden:ro \
  -p 3000:3000 \
  ghcr.io/pvandervelde/merge-warden-server:latest
```

### Azure Container Apps

See the environment variable section in
[Deploy on Azure Container Apps](deploy-on-azure.md). Store the config file content as
a Key Vault secret and mount it as a volume, then set `MERGE_WARDEN_CONFIG_FILE`.

### AWS ECS

Mount the config file from S3 (via a init container) or EFS, then set
`MERGE_WARDEN_CONFIG_FILE` in the task definition's environment section.

---

## Field name differences

The application config uses `ApplicationDefaults` field names, which differ from the
per-repository config. The most common ones:

| Application config field | Per-repo equivalent | Purpose |
| :--- | :--- | :--- |
| `enforceTitleValidation` | `required` (under `prTitle`) | Enable title validation |
| `titlePattern` | `pattern` (under `prTitle`) | Title regex |
| `labelIfTitleInvalid` | `label_if_missing` (under `prTitle`) | Label when title invalid |
| `enforceWorkItemValidation` | `required` (under `workItem`) | Enable work item validation |
| `workItemPattern` | `pattern` (under `workItem`) | Work item reference regex |
| `labelIfWorkItemMissing` | `label_if_missing` (under `workItem`) | Label when work item missing |

For the complete field list, see [Application configuration schema](../reference/app-config.md).

---

## Precedence

When both files set the same policy, the per-repository file takes precedence:

```
per-repo .github/merge-warden.toml  (highest priority)
    ↓ overrides
MERGE_WARDEN_CONFIG_FILE             (application defaults)
    ↓ overrides
Compiled-in defaults                 (lowest priority, always present)
```

See [Configuration precedence](../explanation/config-precedence.md) for a detailed
explanation.

---

## If no config file is provided

When `MERGE_WARDEN_CONFIG_FILE` is unset, compiled-in defaults apply. With compiled-in
defaults, title and work item validation are both **disabled** — the server runs but
enforces no policies until repositories provide their own `.github/merge-warden.toml`.

---

## Related

- [Application configuration schema](../reference/app-config.md)
- [Configuration precedence](../explanation/config-precedence.md)
- [Why there are two configuration files](../explanation/two-config-files.md)
