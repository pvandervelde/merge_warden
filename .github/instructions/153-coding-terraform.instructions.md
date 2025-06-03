---
applyTo: "*"
---

# Copilot Instructions

## coding-terraform

**tf-documentation:** Add documentation comments for each resource, module, and variable.
Use the `#` symbol for comments. Use `##` for module-level documentation. Add examples to the
documentation comments when possible.

**tf-ci:** Run `terraform validate` and `terraform fmt` as part of the CI pipeline. This will help ensure
that the code is valid and follows the correct formatting. Use `terraform plan` to check for any
changes before applying them.

**tf-release-management:** Use tooling to manage the release
process. This includes creating release notes, tagging releases, and managing version numbers.


