---
title: "Tutorial: Enforce your first PR policy"
description: "Add a per-repository configuration file and enforce pull request title format."
---

# Tutorial: Enforce your first PR policy

This tutorial assumes you have completed
[Your first Merge Warden deployment](01-getting-started.md) and have a running server that
receives webhook events from your test repository.

**What you will do:**

1. Create `.github/merge-warden.toml` in your test repository
2. Enable PR title validation
3. Open a pull request with an invalid title and observe the failing check
4. Fix the title and observe the check pass

---

## Step 1 — Create the configuration file

In your test repository, create the file `.github/merge-warden.toml` on the default branch
(usually `main` or `master`):

```toml
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
label_if_missing = "invalid-title-format"
```

This enables title validation using the built-in conventional commits pattern. Any PR whose
title does not match the pattern will fail the check and receive the label
`invalid-title-format`.

Commit and push directly to the default branch:

```bash
mkdir -p .github
cat > .github/merge-warden.toml << 'EOF'
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
label_if_missing = "invalid-title-format"
EOF

git add .github/merge-warden.toml
git commit -m "chore: add merge-warden configuration"
git push origin main
```

> **Note:** Merge Warden reads the configuration file from the default branch on every
> webhook event. You do not need to restart the server.

---

## Step 2 — Open a pull request with an invalid title

Create a branch and open a pull request with a title that does not follow the conventional
commit format:

```bash
git checkout -b test-policy-enforcement
git commit --allow-empty -m "add feature"
git push origin test-policy-enforcement
```

Open a pull request on GitHub with the title **"add feature"** (no type prefix, no colon).

---

## Step 3 — Observe the failing check

On the pull request on GitHub:

1. Go to the **Checks** tab.
2. You should see a check named **merge-warden** with a failed status.
3. The check details will show a diagnostic message explaining what is wrong with the title,
   for example:

   > PR title "add feature" does not match the required format. Expected a conventional commit
   > prefix such as `feat:`, `fix:`, `docs:`, etc.

4. The label `invalid-title-format` will be applied to the pull request automatically.

---

## Step 4 — Fix the title and observe the check pass

Edit the pull request title on GitHub to a valid conventional commit format, for example:

```
feat: add new feature
```

Merge Warden receives the `edited` webhook event and re-evaluates the title. Within a few
seconds:

1. The **Checks** tab shows the check passing.
2. The `invalid-title-format` label is removed automatically.

---

## What the conventional commit pattern requires

The default pattern enforces this format:

```
<type>(<optional scope>): <description>
```

Valid types: `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`,
`style`, `test`.

Examples of valid titles:

```
feat: add user authentication
fix(api): handle null response from GitHub
docs(readme): update installation instructions
chore!: drop support for Node 14
```

To use a different pattern, see
[Configure PR title validation](../how-to/configure-pr-title-validation.md).

---

## What's next?

- Require work item references: [Configure work item validation](../how-to/configure-work-item-validation.md)
- Control PR size: [Configure PR size labels](../how-to/configure-pr-size-labels.md)
- Full schema reference: [Per-repository configuration](../reference/per-repo-config.md)
