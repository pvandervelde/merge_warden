# Merge Warden CLI Specification

## Overview

Command-line interface for validating pull requests against configured rules across multiple Git
providers (GitHub, GitLab, Azure DevOps). Acts as the primary user interface for the Merge Warden
validation system.

## Commands

### `check-pr`

```text
Validate a pull request against configured rules

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

```text
Manage configuration
SUBCOMMANDS:
    init      Create initial configuration file
    validate  Check configuration syntax
    get       Show current configuration
    set       Update configuration values
```

### `auth`

```text
Authenticate with Git providers
SUBCOMMANDS:
    github [app|token]  Authenticate with GitHub (OAuth App or Personal Access Token)
```

## Configuration

```toml
# Example .merge-warden.toml
[default]
provider = "github"  # Default Git provider

[rules]
require_work_items = true
enforce_title_convention = "conventional"
min_approvals = 1

[authentication]
auth_method = "token"
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

## Testing Strategy

1. **Unit Tests**
   - Command line argument parsing
   - Configuration file loading
   - Output formatting (human vs JSON)

2. **Integration Tests**
   - Mocked provider API responses
   - End-to-end command execution with test credentials

3. **Validation Scenarios**
   - Success case with passing PR
   - Failure case with rule violations
   - Error cases for invalid inputs

4. **Authentication Flow Tests**
   - Token storage/retrieval
   - Error handling for invalid credentials

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
