---
title: "CLI reference"
description: "Complete reference for the merge-warden command-line interface."
---

# CLI reference

The `merge-warden` CLI starts a local webhook server that receives and processes GitHub pull
request events. It uses the same core logic as the production container but stores
credentials in the system keyring.

**Supported platform:** GitHub only.

---

## Global flags

| Flag | Description |
| :--- | :--- |
| `-v`, `--verbose` | Enable verbose output |
| `-h`, `--help` | Show help |
| `--version` | Show version |

---

## `checkpr` — Start the local webhook server

Loads credentials from the system keyring, creates a GitHub App client, and starts an HTTP
server that receives webhook events from GitHub and processes pull requests.

```text
USAGE:
    merge-warden checkpr [OPTIONS]

OPTIONS:
    -p, --provider <PROVIDER>    Git provider to use. Only "github" is supported.
    -c, --config <FILE>          Path to a CLI config file [default: .merge-warden.toml]
    -v, --verbose                Enable verbose output
    -h, --help                   Show help
```

**Before running `checkpr`**, authenticate with `merge-warden auth github` so that
credentials are stored in the keyring.

**Example:**

```bash
# Use default config file (.merge-warden.toml in current directory)
merge-warden checkpr --provider github

# Use a custom config file
merge-warden checkpr --provider github --config /path/to/config.toml
```

The server listens on `http://localhost:3000` by default. Configure your GitHub App webhook
URL (or smee relay target) to `http://localhost:3000/api/merge_warden`.

**Log level:** Controlled by the `MERGE_WARDEN_LOG` environment variable (not `RUST_LOG`).

---

## `config` — Manage the CLI configuration file

### `config init`

Creates a new CLI configuration file with default values.

```text
USAGE:
    merge-warden config init [OPTIONS]

OPTIONS:
    -p, --path <PATH>    Path to save the configuration file [default: .merge-warden.toml]
```

Fails if the file already exists.

**Example:**

```bash
merge-warden config init
merge-warden config init --path /etc/merge-warden/.merge-warden.toml
```

### `config validate`

Checks that an existing configuration file is valid TOML and matches the expected schema.

```text
USAGE:
    merge-warden config validate [OPTIONS]

OPTIONS:
    -p, --path <PATH>    Path to the configuration file [default: .merge-warden.toml]
```

Exits with code `0` on success, non-zero on error. Prints a diagnostic message to stdout.

**Example:**

```bash
merge-warden config validate
merge-warden config validate --path /path/to/config.toml
```

---

## `auth` — Authenticate with GitHub

Stores GitHub credentials in the system keyring. Run this before `checkpr`.

### `auth github`

```text
USAGE:
    merge-warden auth github [METHOD]

ARGUMENTS:
    METHOD    Authentication method: "app" or "token" [default: token]
```

#### `auth github app` — GitHub App authentication

Prompts interactively for:

1. **App ID** — the numeric App ID from your GitHub App settings page
2. **Path to private key** — the filesystem path to the downloaded `.pem` file

Credentials are stored in the system keyring under the `merge_warden_cli` service.

```bash
merge-warden auth github app
# GitHub App Authentication
# ------------------------
# Please provide the following information:
# App ID:
# 123456
# Path to private key file:
# /home/user/.config/merge-warden/private-key.pem
```

#### `auth github token` — Personal access token authentication

Prompts interactively for a GitHub personal access token.

```bash
merge-warden auth github token
# or equivalently:
merge-warden auth github
```

---

## Exit codes

| Code | Meaning |
| :---: | :--- |
| `0` | Success |
| `1` | General error (check stderr for details) |
| `2` | Configuration error (invalid or missing config file) |
| `3` | Authentication error (missing or invalid credentials) |
| `4` | Network or API error |
| `5` | Invalid arguments |

---

## CLI configuration file format

The CLI configuration file (`.merge-warden.toml` by default) uses the
`ApplicationDefaults` schema for policies. See
[Application configuration schema](app-config.md) for the full field reference.

```toml
[default]
# Default Git provider
provider = "github"

[authentication]
# Authentication method: "app" or "token"
auth_method = "app"

[webhooks]
# Webhook server port
port = 3000

[policies]
# Application-level policy defaults — see app-config reference
enforceTitleValidation = true
```

---

## Related

- [Application configuration schema](app-config.md)
- [Run the server locally](../how-to/run-locally.md)
- [Tutorial: Your first deployment](../tutorials/01-getting-started.md)
