# Merge Warden

[![CI](https://github.com/pvandervelde/merge_warden/actions/workflows/ci.yml/badge.svg)](https://github.com/pvandervelde/merge_warden/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/pvandervelde/merge_warden)](https://github.com/pvandervelde/merge_warden/releases)
[![License](https://img.shields.io/github/license/pvandervelde/merge_warden)](LICENSE)

Merge Warden is an automated tool designed to enforce merge policies on GitHub pull requests. It ensures that PRs meet
predefined criteria before being merged, enhancing code quality and streamlining the development workflow.

## Features:
* **Automated PR Checks**: Validates pull requests against custom rules to ensure compliance.
* **Labeling System**: Automatically labels PRs based on size, affected areas, or other criteria.
* **Integration with GitHub Checks**: Utilizes GitHub Checks API for seamless integration and feedback.
* **Deployment Flexibility**: Can be deployed as an Azure Function or AWS Lambda for scalability.
* **Customizable Specifications**: Define rules and behaviors through YAML specification files.

---

## 🚀 Quickstart

### 1. Install the CLI

Download the latest binary from [Releases](https://github.com/pvandervelde/merge_warden/releases) or build from source:

```sh
cargo install --path .
```

### 2. Configure your rules

Create a YAML file in XXX (e.g., default.yaml):

```yaml

```

### 3. Deploy

See deployment options for Azure Functions or AWS Lambda.

## Telemetry & Logging

Merge Warden provides observability using standard output and structured logs.

### Logging Levels
Set the RUST_LOG environment variable:

```sh
Copy
Edit
export RUST_LOG=info
```

Use debug or trace for deeper insights during development.

### Azure Insights / AWS CloudWatch

When deployed as a cloud function:
* Azure logs are visible in Application Insights.
* AWS logs are visible in CloudWatch via the Lambda console.
