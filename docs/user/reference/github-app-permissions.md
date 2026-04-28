---
title: "GitHub App permissions"
description: "Complete list of GitHub App permissions and webhook subscriptions required by Merge Warden."
---

# GitHub App permissions

This page is the single authoritative reference for all permissions that must be configured
on the GitHub App that Merge Warden uses. Create or update the app at
**GitHub → Settings → Developer settings → GitHub Apps**.

---

## Repository permissions

| Permission | Level | Why it is needed |
| :--- | :--- | :--- |
| Checks | Read & Write | Create and update check runs on pull requests |
| Contents | Read | Read `.github/merge-warden.toml` from the default branch |
| Issues | Read | Read issue metadata for milestone and project propagation (GitHub issues only) |
| Labels | Read & Write | Apply labels to pull requests; create fallback labels when none match |
| Metadata | Read | Required by GitHub for all GitHub Apps (cannot be removed) |
| Projects | Read & Write | Add pull requests to repository-level Projects v2 |
| Pull requests | Read & Write | Read PR details; apply labels and post comments |

> **Note on Labels:** Without Read & Write on Labels, Merge Warden can still read and
> apply *existing* repository labels. The Write level is needed only to *create* new labels
> (e.g. fallback change-type labels). If you prefer not to grant Write, set
> `create_if_missing = false` in `[change_type_labels.fallback_label_settings]`.

---

## Organisation permissions

| Permission | Level | Why it is needed |
| :--- | :--- | :--- |
| Projects | Read & Write | Add pull requests to organisation-level Projects v2 (`sync_project_from_issue`) |

> **Note:** The Organisation Projects permission is listed under *Organization permissions*
> in the GitHub App settings, not under *Repository permissions*. It is only required when
> `sync_project_from_issue = true`. Milestone and project propagation are available only
> when issues are tracked in GitHub Issues — external issue trackers are not supported.
> Repositories owned by individual users (not organisations) cannot use the Projects v2
> GraphQL API via GitHub App tokens, so `sync_project_from_issue` has no effect on
> personal repositories.

---

## Webhook events

Subscribe the GitHub App to the following events:

| Event | Why it is needed |
| :--- | :--- |
| Pull request | Trigger processing when a PR is opened, edited, synchronised, reopened, or converted from draft |
| Pull request review | Trigger state-label updates when a review is submitted |

---

## Webhook URL

Set the **Webhook URL** to:

```
https://<your-server-hostname>/api/merge_warden
```

Set **Content type** to `application/json`.

Set a **Webhook secret** and store the same value in your deployment as
`GITHUB_WEBHOOK_SECRET`. This secret is used to verify the HMAC-SHA256 signature on every
incoming webhook payload.

---

## Installing the App

After creating the GitHub App, install it on the repositories (or the whole organisation)
that should be managed by Merge Warden. The App must be installed for Merge Warden to
receive webhooks and make API calls for those repositories.

---

## Related

- [Tutorial: Your first deployment](../tutorials/01-getting-started.md)
- [Environment variables reference](environment-variables.md)
- [How Merge Warden works](../explanation/how-merge-warden-works.md)
