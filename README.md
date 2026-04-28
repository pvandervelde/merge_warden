# merge_warden

Merge Warden is a GitHub webhook server that enforces pull request policies and automates
PR workflows. Run it as a container, point your GitHub App's webhook at it, and it
automatically checks every pull request against rules you configure per repository.

📖 **[Full documentation](https://pvandervelde.github.io/merge_warden/)**

## Features

- **PR title validation** — enforces conventional commit format (or your own regex)
- **Work-item references** — requires a linked issue in the PR description
- **PR size labels** — automatically labels XS / S / M / L / XL / XXL by lines changed
- **WIP detection** — blocks merge when the PR title or description contains WIP markers
- **PR state labels** — applies a single label reflecting draft / in-review / approved state
- **Issue propagation** — copies milestone and Projects v2 membership from the linked issue
- **Change-type labels** — maps conventional commit types to repository labels
- **Bypass rules** — per-policy lists of users who can skip validation

All policies are configured with a TOML file committed to each repository at
`.github/merge-warden.toml`. A server-level config file sets the defaults that apply when
no per-repo file exists.

## Quick start

```bash
docker run --rm \
  -e MERGE_WARDEN_GITHUB_APP_ID=<your-app-id> \
  -e MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY="$(cat private-key.pem)" \
  -e GITHUB_WEBHOOK_SECRET=<your-webhook-secret> \
  -p 3000:3000 \
  ghcr.io/pvandervelde/merge-warden-server:latest
```

Then verify:

```bash
curl http://localhost:3000/api/merge_warden
# HTTP 200 OK
```

See the [getting-started tutorial](https://pvandervelde.github.io/merge_warden/tutorials/01-getting-started/)
for the complete walkthrough including GitHub App setup.

## Deployment

Merge Warden runs as an OCI container on any container host:

- [Azure Container Apps](https://pvandervelde.github.io/merge_warden/how-to/deploy-on-azure/)
- [AWS ECS / Fargate](https://pvandervelde.github.io/merge_warden/how-to/deploy-on-aws/)
- [Local development with Docker](https://pvandervelde.github.io/merge_warden/how-to/run-locally/)

## Configuration

Add `.github/merge-warden.toml` to the default branch of any repository you want Merge
Warden to monitor. The server reads this file via the GitHub API on every webhook event —
no restart needed when you change it.

```toml
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
label_if_missing = "invalid-title-format"

[policies.pullRequests.workItem]
required = true
label_if_missing = "missing-work-item"

[policies.pullRequests.prSize]
enabled = true
fail_on_oversized = false
```

Full schema reference: [per-repo config](https://pvandervelde.github.io/merge_warden/reference/per-repo-config/)

## Contributing

Issues and pull requests are welcome. Please open an issue before starting work on a
significant change.

## License

[MIT](LICENSE)
