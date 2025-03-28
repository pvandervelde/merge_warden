# Merge Warden CLI

The Merge Warden CLI is a command-line interface for validating pull requests against configured rules across multiple Git providers (GitHub, GitLab, Azure DevOps). It acts as the primary user interface for the Merge Warden validation system.

## Commands

### `check-pr`

Validates a pull request against configured rules.

```text
USAGE:
    merge-warden check-pr [OPTIONS] --provider <PROVIDER> --repo <REPO> --pr <PR_NUMBER>

OPTIONS:
    -p, --provider <PROVIDER>  Supported Git providers: github
    -r, --repo <REPO>          Repository in format: owner/repo
    -n, --pr <PR_NUMBER>       Pull request number
    -j, --json                 Machine-readable JSON output
    -c, --config <FILE>        Alternate config file [default: .merge-warden.toml]
    -v, --verbose              Show detailed validation results
```

### `config`

Manages configuration.

```text
SUBCOMMANDS:
    init      Create initial configuration file
    validate  Check configuration syntax
    get       Show current configuration
    set       Update configuration values
```

### `auth`

Authenticates with Git providers.

```text
SUBCOMMANDS:
    github [app|token]  Authenticate with GitHub (OAuth App or Personal Access Token)
```

## Configuration

The CLI uses a configuration file to store settings. The default location for the configuration file is
`.merge-warden.toml` in the root of the repository. You can specify an alternate config file using
the `-c` or `--config` option with the `check-pr` command.

The configuration file is in TOML format. Here is an example configuration file:

```toml
# Example .merge-warden.toml
[default]
provider = "github"  # Default Git provider

[rules]
require_work_items = true
enforce_title_convention = "conventional"
min_approvals = 1

[github]
app_id = 12345
private_key_path = "~/.merge-warden/github-key.pem"
```

## Error Handling

| Exit Code | Description               | Example Scenario                     |
|-----------|---------------------------|---------------------------------------|
| 0         | Success                   | Validation passed                    |
| 1         | Validation failed         | PR failed rule checks                |
| 2         | Configuration error       | Invalid config file syntax           |
| 3         | Authentication error      | Invalid or expired credentials       |
| 4         | Network error             | API connection failure               |
| 5         | Invalid arguments         | Missing required parameters          |

## Examples

Basic validation:

```bash
merge-warden check-pr -p github -r owner/repo -n 42
```

JSON output:

```bash
merge-warden check-pr -p github -r group/project -n 123 --json
```

Initialize configuration:

```bash
merge-warden config init
```

GitHub App authentication:

```bash
merge-warden auth github app
