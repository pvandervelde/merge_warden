---
title: "How to suppress keyword-triggered labels"
description: "Allow PR authors and reviewers to prevent Merge Warden from re-applying keyword-triggered labels."
---

# How to suppress keyword-triggered labels

When Merge Warden detects a keyword in a PR title or body (such as `breaking change`,
`security`, `hotfix`, or `tech debt`), it automatically applies a label and posts an
explanatory comment. If the label was incorrectly triggered — for example, the word
"security" appears in a comment that does not indicate a security concern — a PR
participant can suppress it.

---

## Posting a suppression command

Any PR participant (author, reviewer, or commenter) can suppress a keyword-triggered
label by posting a PR comment that contains the following on its own line:

```
@merge-warden suppress: <label-name>
```

Replace `<label-name>` with the exact name of the label to suppress. The command can
appear anywhere in a comment body (it does not need to be the only content), but it must
start at the beginning of a line. Leading and trailing whitespace on that line is ignored.

**Example:**

If the `breaking-change` label was incorrectly applied, post:

```
@merge-warden suppress: breaking-change
```

Merge Warden processes suppression commands on the next PR event (for example, when
a new commit is pushed or when the PR title is edited). After a suppression command
is detected:

- The label is removed from the PR.
- Merge Warden will not re-apply that label for the lifetime of the PR, even if
  subsequent evaluations detect the keyword again.

---

## How the explanatory comment helps

When Merge Warden applies a keyword-triggered label, it posts a comment on the PR that
includes the exact suppression command to use. You do not need to remember the syntax —
look for the comment that mentions the label name.

---

## Which labels can be suppressed

Any keyword-triggered label can be suppressed. The four built-in keyword labels and
their default names are:

| Keyword trigger | Default label name |
| :--- | :--- |
| `!:` in title, or `breaking change` in body | `breaking-change` |
| `security` or `vulnerability` in body | `security` |
| `hotfix` in body | `hotfix` |
| `tech debt` or `technical debt` in body | `tech-debt` |

If you have customised label names using `[change_type_labels.keyword_labels]`, use the
customised names in the suppression command.

Suppression affects only keyword-triggered labels. Labels applied by other features
(PR size, WIP, state lifecycle, change-type) cannot be suppressed via this mechanism.

---

## Changing the bot mention prefix

By default the suppression command prefix is `@merge-warden`. If your GitHub App is
installed under a different name, set the `bot_mention` field in the application-level
configuration:

```toml
# In the application-level config file (MERGE_WARDEN_CONFIG_FILE)
[policies]
bot_mention = "@acme-merge-warden[bot]"
```

Users must then use the new prefix in their suppression commands:

```
@acme-merge-warden[bot] suppress: breaking-change
```

---

## Related

- [Configure change-type labels](configure-change-type-labels.md)
- [Application configuration schema — bot_mention field](../reference/app-config.md#bot_mention)
- [Per-repository configuration schema — keyword_labels](../reference/per-repo-config.md#change_type_labelskeyword_labels)
