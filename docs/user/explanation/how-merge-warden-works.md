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
GitHub App delivers webhook POST to /api/github/webhook
    ↓
HMAC-SHA256 signature verification
    ↓  (rejected with 401 if signature is invalid)
202 Accepted returned to GitHub  ← response sent here, before processing
    ↓
Repository scope check (if [policies.repository_scope] is configured)
    ↓  (out-of-scope repositories are acknowledged and dropped here — no
    ↓   further steps run, no GitHub API call is made)
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

The repository scope check is the **first** thing evaluated once processing begins —
before Merge Warden even looks at the event type, and before any repository-specific data
(config file, org policy, topics, custom properties) is fetched. A webhook payload with a
missing or unparseable `repository.name` is also treated as out of scope. See
[How to configure repository scope filtering](../how-to/configure-repository-scope.md).

> **In `queue` mode**, Merge Warden itself does not perform the steps above the repository
> scope check. It exposes no webhook POST endpoint at all — a wholly separate receiver
> service performs webhook receipt, HMAC verification, and enqueueing, and Merge Warden
> consumes the resulting message from the queue. The repository scope check and everything
> below it runs the same way regardless of receiver mode. See
> [Webhook vs queue receiver modes](receiver-modes.md) for the full architecture.

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

## Automatic configuration file validation

When a pull request modifies `.github/merge-warden.toml`, Merge Warden automatically
fetches the version of that file at the PR's head commit and validates it.

- If the file is **invalid** (bad TOML or an unsupported `schemaVersion`), Merge Warden
  posts a warning comment on the PR. The message is self-contained: it states what failed
  (invalid TOML syntax, or an unsupported `schemaVersion`), the impact (application defaults
  are used until the file is fixed), and the next step (fix the file on the default branch),
  with the underlying parse detail included as supporting context rather than as the
  headline message. This comment is updated on every subsequent push to the PR branch.
- If the file is **valid**, any existing warning comment is automatically deleted.

This validation is purely informational — it does not affect the Merge Warden check result
and cannot block merging. Its purpose is to give contributors early feedback before a
broken config lands on the default branch.

Operators can monitor configuration-load failures across all repositories, independent of
whether any pull request currently touches the config file, via the
`config_load_failures_total` metric — see
[HTTP endpoints reference](../reference/http-endpoints.md#get-metrics--prometheus-metrics).

---

## Broken default-branch configuration policy

The validation above only runs when a PR's diff touches `.github/merge-warden.toml`. It
says nothing about the config that is already committed to the default branch and is
actually governing every PR — including ones that don't touch the config file at all.

If that default-branch config becomes malformed or specifies an unsupported
`schemaVersion`, Merge Warden falls back to application defaults for that repository, on
every PR, until it is fixed. By default this fallback is silent. Operators can change that
with the application-level `broken_repo_config_behaviour` setting:

- **`silent_fallback`** (default) — unchanged: no PR comment, check conclusion unaffected.
- **`notify`** — posts a PR comment explaining that the default-branch config could not be
  loaded and that application defaults are in effect. Does not block.
- **`block`** — posts the same comment **and** forces the check conclusion to `failure`,
  hard-blocking the PR until the default-branch config is fixed.

This notice uses a comment marker distinct from the PR-diff config check above, so the two
are managed independently — fixing one does not affect the other's comment. A transient
fetch failure or a simply-absent config file is never treated as "broken" and never
triggers this notice, regardless of the setting.

This setting is application-level only: it cannot be set or overridden by a repository's
own `.github/merge-warden.toml` or an org policy file. See
[`broken_repo_config_behaviour`](../reference/app-config.md#broken_repo_config_behaviour)
for the full reference and
[Choosing a broken default-branch config policy](../how-to/set-app-level-defaults.md#choosing-a-broken-default-branch-config-policy)
for guidance on picking a value.

---

## Regex pattern validation

`title_pattern` and `work_item_reference_pattern` (configurable as `pattern` under
`[policies.pullRequests.prTitle]`/`[policies.pullRequests.workItem]` in a repository's
`.github/merge-warden.toml`, in an org policy file's `[enforced]`/`[defaults]` sections, or
as `default_title_pattern`/`default_work_item_pattern` at the application level) are
validated wherever they enter the system, with two fixed limits that cannot be raised at
any configuration tier:

- **Length cap** — a pattern longer than **256 bytes** is rejected outright, without
  attempting to compile it.
- **Compile-size budget** — a pattern within the length cap is compiled with a fixed
  **1 MiB** compiled-program size budget. A pattern that is syntactically valid but
  pathologically expensive to compile (for example, a large bounded repetition such as
  `(#\d+){1,10000}`) is rejected deterministically rather than risking unbounded
  compile-time memory or CPU.

**What happens when a pattern fails validation depends on where it was set:**

| Where the pattern was set | What happens on an invalid pattern |
| :--- | :--- |
| Application default (`default_title_pattern`/`default_work_item_pattern`) | The server logs the error and **refuses to start**. See [Set application-level defaults — startup pattern validation](../how-to/set-app-level-defaults.md#startup-validation-of-default-patterns). |
| Org policy (`[enforced]`/`[defaults]`) | Treated exactly like any other org-policy load failure: governed by `fail_if_unreachable` (warn-and-fall-back by default, or abort PR processing if `true`). See [Configure an organisation-level policy](../how-to/configure-org-policy.md#behaviour-when-the-org-policy-file-is-unreachable). |
| Repository's own `.github/merge-warden.toml` | The rest of that file's settings remain fully in effect. Only the specific check that depends on the broken pattern (title or work item) fails, with a distinct comment — see below. |

**At check time**, if the effective (merged) `title_pattern` or `work_item_reference_pattern`
governing a pull request is invalid, Merge Warden does **not** report the ordinary "title
doesn't match the pattern" or "no work item reference found" failure, and does **not**
silently skip the check or fall back to the built-in default pattern. It posts a distinct
comment naming the problem:

> The configured `title_pattern` in `.github/merge-warden.toml` is invalid and could not be
> evaluated: *(reason)*. A repository maintainer must correct this pattern before this
> check can run.

(The equivalent message for the work-item check names `work_item_reference_pattern`
instead.) A maintainer resolves this by finding which configuration tier actually set the
broken pattern — check the repository's own `.github/merge-warden.toml` first, then any
org policy, then the operator's application defaults — and fixing or removing it there.

A broken pattern **never** escalates to the `broken_repo_config_behaviour` policy described
above: that policy governs whether the whole repository config tier is used at all (a
structural failure, like malformed TOML), whereas a bad pattern is a narrower, local
failure handled entirely within the one check that depends on it. Every other policy in
the same repository config — labels, bypass rules, size thresholds, the *other* pattern —
keeps working normally.

Validated, compiled patterns are reused across pull-request events via a process-wide
cache, so identical patterns (most repositories share the built-in defaults) are not
recompiled on every webhook event. This caching is purely a performance optimisation and
has no user-visible effect on behaviour.

See [Per-repository configuration schema](../reference/per-repo-config.md#policiespullrequestsprtitle)
and [Application configuration schema](../reference/app-config.md#default_title_pattern-and-default_work_item_pattern)
for the field-level reference.

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
