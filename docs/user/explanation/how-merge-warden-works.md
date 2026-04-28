---
title: "How Merge Warden works"
description: "An explanation of the event lifecycle, configuration loading, and why a GitHub App is required."
---

# How Merge Warden works

This page explains the concepts behind Merge Warden so you can predict its behaviour,
diagnose unexpected results, and make informed configuration decisions.

---

## The event lifecycle

Every piece of work Merge Warden does starts from a GitHub webhook event. The full lifecycle
looks like this:

```
GitHub repository
    ↓  PR opened / edited / synchronised / ready_for_review / reopened / reviewed
GitHub App delivers webhook POST to /api/merge_warden
    ↓
HMAC-SHA256 signature verification
    ↓  (rejected with 401 if signature is invalid)
202 Accepted returned to GitHub  ← response sent here, before processing
    ↓
Event routing — only pull_request and pull_request_review events proceed
    ↓
Per-repository config loaded from .github/merge-warden.toml via GitHub API
    ↓  (falls back to application defaults if file is absent or malformed)
Policy evaluation
    ↓
GitHub API calls — labels applied, check run updated, comments posted
```

The `202 Accepted` response is returned to GitHub immediately after the HMAC signature is
verified, before any policy evaluation or GitHub API calls. This ensures Merge Warden
responds within GitHub's 10-second webhook delivery timeout regardless of how long
downstream processing takes. Processing happens after the response is sent.

> **In `queue` mode**, the payload is additionally persisted to a queue before the response
> is returned, and a separate consumer task performs the policy evaluation. See
> [Webhook vs queue receiver modes](receiver-modes.md) for details.

---

## What happens on each PR event

| Action | What Merge Warden does |
| :--- | :--- |
| `opened` | Full policy evaluation |
| `synchronise` (new commits pushed) | Full policy evaluation |
| `edited` (title or description changed) | Full policy evaluation |
| `ready_for_review` (draft converted) | Full policy evaluation |
| `reopened` | Full policy evaluation |
| `unlocked` | Full policy evaluation |
| `pull_request_review` submitted | State labels updated (draft/in-review/approved) |

All other PR actions (e.g. `assigned`, `labeled`, `milestoned`) are acknowledged and
discarded — no policy evaluation occurs.

---

## How per-repository configuration is loaded

Merge Warden fetches `.github/merge-warden.toml` directly from the repository's default
branch via the GitHub Contents API on every webhook event. There is no local caching across
events.

Consequences:

- You can update the configuration file on the default branch and the next PR event will
  use the new configuration immediately — no server restart required.
- The server must have `Contents: Read` permission on the repository to fetch the file.
- If the file does not exist, is not valid TOML, or has an unsupported `schemaVersion`, the
  server falls back to application-level defaults and logs a warning.

---

## Why a GitHub App is required

Merge Warden uses GitHub App authentication rather than a personal access token (PAT) for
several reasons:

- **Installation-scoped tokens.** A GitHub App issues short-lived tokens scoped to
  specific repositories. A PAT is tied to a user account and may have broader access
  than intended.
- **Fine-grained permissions.** GitHub Apps declare exactly which permissions they require
  (checks write, pull requests read/write, etc.). A PAT's scope is coarser.
- **Organisational control.** An organisation administrator controls which repositories the
  App is installed on. A PAT is controlled by the individual user.
- **Webhook identity.** GitHub App webhooks include an `installation` object that Merge
  Warden uses to create an installation-scoped API client. This is not available with
  PAT-triggered webhooks.
- **Issue metadata propagation.** Milestone and project copying from issues to PRs is only
  available when issues are tracked in GitHub Issues. External issue trackers (Jira, Linear,
  Azure Boards, etc.) are not supported.

---

## Webhook signature verification

Every incoming webhook is verified against the `GITHUB_WEBHOOK_SECRET` using HMAC-SHA256
before any processing begins. This prevents processing of forged or replayed requests.
The verification uses a constant-time comparison to prevent timing attacks.

Requests with a missing or invalid `X-Hub-Signature-256` header are rejected with
`401 Unauthorized`.

---

## What Merge Warden does not do

- It does not merge or close pull requests.
- It does not leave comments on code (only informational PR-level comments).
- It does not modify any files in the repository.
- It does not store any pull request data persistently.

---

## Related

- [Configuration precedence](config-precedence.md)
- [Webhook vs queue receiver modes](receiver-modes.md)
- [HTTP endpoints reference](../reference/http-endpoints.md)
- [GitHub App permissions](../reference/github-app-permissions.md)
