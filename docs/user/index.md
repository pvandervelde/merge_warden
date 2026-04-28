---
title: "Merge Warden Documentation"
description: "Complete documentation for Merge Warden — a GitHub pull request policy enforcement server."
---

# Merge Warden

Merge Warden is a self-hosted webhook server that automatically enforces pull request policies
on GitHub repositories. Install it once as a GitHub App, point your repositories at it, and
every pull request is checked for the rules you define — title format, work item references,
size limits, and more.

## How to navigate this documentation

| Section | Good for |
| :--- | :--- |
| [Tutorials](tutorials/01-getting-started.md) | New to Merge Warden — step-by-step walkthroughs to a working setup |
| [How-to guides](how-to/deploy-on-azure.md) | Know what you want to do — task-focused recipes |
| [Reference](reference/per-repo-config.md) | Looking up a specific setting, variable, or command |
| [Explanation](explanation/how-merge-warden-works.md) | Want to understand why something works the way it does |

## Tutorials

- [Your first Merge Warden deployment](tutorials/01-getting-started.md) — create a GitHub App,
  run the container, receive your first webhook
- [Enforce your first PR policy](tutorials/02-add-first-policy.md) — add `.github/merge-warden.toml`
  and see the title check in action

## How-to guides

**Deployment**

- [Deploy on Azure Container Apps](how-to/deploy-on-azure.md)
- [Deploy on AWS ECS / Fargate](how-to/deploy-on-aws.md)
- [Run the server locally](how-to/run-locally.md)
- [Test webhooks with smee relay](how-to/test-with-smee.md)

**Policy configuration**

- [Configure PR title validation](how-to/configure-pr-title-validation.md)
- [Configure work item validation](how-to/configure-work-item-validation.md)
- [Configure PR size labels](how-to/configure-pr-size-labels.md)
- [Configure WIP detection](how-to/configure-wip-detection.md)
- [Configure PR state lifecycle labels](how-to/configure-pr-state-labels.md)
- [Configure issue metadata propagation](how-to/configure-issue-propagation.md)
- [Configure change-type labels](how-to/configure-change-type-labels.md)
- [Configure bypass rules](how-to/configure-bypass-rules.md)
- [Set application-level policy defaults](how-to/set-app-level-defaults.md)

## Reference

- [Per-repository configuration schema](reference/per-repo-config.md) — `.github/merge-warden.toml`
- [Application configuration schema](reference/app-config.md) — `MERGE_WARDEN_CONFIG_FILE`
- [Environment variables](reference/environment-variables.md)
- [HTTP endpoints](reference/http-endpoints.md)
- [CLI reference](reference/cli.md)
- [GitHub App permissions](reference/github-app-permissions.md)

## Explanation

- [How Merge Warden works](explanation/how-merge-warden-works.md)
- [Configuration precedence](explanation/config-precedence.md)
- [Why there are two configuration files](explanation/two-config-files.md)
- [Webhook vs queue receiver modes](explanation/receiver-modes.md)
- [Bypass rules in depth](explanation/bypass-rules.md)

## Links

- [GitHub repository](https://github.com/pvandervelde/merge_warden)
- [Releases](https://github.com/pvandervelde/merge_warden/releases)
- [Issue tracker](https://github.com/pvandervelde/merge_warden/issues)
