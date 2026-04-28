---
title: "Tutorial: Your first Merge Warden deployment"
description: "Create a GitHub App, run the Merge Warden container, and receive your first live webhook."
---

# Tutorial: Your first Merge Warden deployment

In this tutorial you will create a GitHub App, run the Merge Warden server as a Docker
container on your local machine, and verify that it receives a live pull request event from
GitHub.

**What you need before you start:**

- A GitHub account and a test repository (can be private)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/) installed and running
- A free account at [smee.io](https://smee.io) to relay webhooks to your local machine
- [Node.js](https://nodejs.org/) installed (provides `npx` to run smee-client)

By the end of this tutorial you will have:

- A GitHub App installed on your test repository
- A running Merge Warden server
- Confirmation that a pull request event reaches the server

---

## Step 1 — Create a GitHub App

GitHub App credentials are how Merge Warden authenticates to read pull requests and post
check results.

1. Go to **GitHub → Settings → Developer settings → GitHub Apps → New GitHub App**.
2. Fill in the form:
   - **GitHub App name**: choose any name (e.g. `merge-warden-test`)
   - **Homepage URL**: `https://github.com/pvandervelde/merge_warden` (or any URL)
   - **Webhook URL**: leave blank for now — you will fill it in step 3
   - **Webhook secret**: type a random secret string and note it (e.g. `dev-secret-123`)
3. Under **Repository permissions**, set:

   | Permission | Level |
   | :--- | :--- |
   | Checks | Read & Write |
   | Contents | Read |
   | Issues | Read |
   | Labels | Read & Write |
   | Metadata | Read (mandatory) |
   | Projects | Read & Write |
   | Pull requests | Read & Write |

   > **Note:** Labels Read & Write allows Merge Warden to create fallback labels when
   > no matching label exists in the repository. If this permission is absent, the app
   > still works — it simply skips label assignment when no match is found.

4. Under **Organisation permissions**, set:

   | Permission | Level |
   | :--- | :--- |
   | Projects | Read & Write |

   > **Note:** The Organisation Projects permission is only required if you use
   > `sync_project_from_issue`. It has no effect on personal (non-organisation)
   > repositories.

   *(In the GitHub App form this step is numbered 4 — the two permission tables are on
   the same page, just in separate sections.)*

5. Under **Subscribe to events**, tick **Pull request** and **Pull request review**.
6. Under **Where can this GitHub App be installed**, select **Only on this account**.
7. Click **Create GitHub App**.
8. Note the **App ID** shown at the top of the settings page.
9. Scroll to the bottom, click **Generate a private key**, save the downloaded `.pem` file.
10. On the left sidebar click **Install App**, then **Install** on your account, and choose
    the test repository.

---

## Step 2 — Create a smee channel

smee.io provides a public URL that forwards HTTP requests to your local machine. GitHub
cannot reach `localhost` directly, so you need a relay.

1. Open [smee.io](https://smee.io) and click **Start a new channel**.
2. Copy the channel URL shown on the page (e.g. `https://smee.io/AbCdEfGhIj123456`).

Keep this URL — you need it in steps 3 and 4.

---

## Step 3 — Configure the GitHub App webhook

1. Go back to your GitHub App settings (**Settings → Developer settings → GitHub Apps →
   your app**).
2. Set the **Webhook URL** to your smee channel URL from step 2.
3. Click **Save changes**.

---

## Step 4 — Start the smee relay

Open a terminal and run:

```bash
npx smee-client --url https://smee.io/AbCdEfGhIj123456 --target http://localhost:3000/api/merge_warden
```

Replace the smee URL with your own channel URL. Leave this terminal running.

---

## Step 5 — Run the Merge Warden server

Open a second terminal. Set the three required environment variables from your GitHub App:

```bash
export MERGE_WARDEN_GITHUB_APP_ID="123456"
export MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY="$(cat /path/to/your-app.private-key.pem)"
export GITHUB_WEBHOOK_SECRET="dev-secret-123"
```

Pull and run the container:

```bash
docker run --rm \
  -e MERGE_WARDEN_GITHUB_APP_ID \
  -e MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY \
  -e GITHUB_WEBHOOK_SECRET \
  -p 3000:3000 \
  ghcr.io/pvandervelde/merge-warden-server:latest
```

Verify the server is healthy:

```bash
curl http://localhost:3000/api/merge_warden
# Expected: HTTP 200 OK
```

---

## Step 6 — Trigger a pull request event

1. In your test repository, create a new branch:

   ```bash
   git checkout -b test-merge-warden
   git commit --allow-empty -m "test: trigger merge warden"
   git push origin test-merge-warden
   ```

2. Open a pull request from that branch on GitHub.
3. Watch the smee relay terminal — you should see a request forwarded.
4. Watch the Docker container logs — you should see a line like:

   ```
   Processing pull request repository=your-repo pull_request=1 action=opened
   ```

5. On the pull request page on GitHub, check the **Checks** tab. Merge Warden has run with
   its compiled-in defaults (title and work-item validation are disabled by default) so the
   check will pass without any configuration.

---

## What's next?

- Add your first policy: [Enforce your first PR policy](02-add-first-policy.md)
- Deploy the server permanently: [Deploy on Azure](../how-to/deploy-on-azure.md) or
  [Deploy on AWS](../how-to/deploy-on-aws.md)
- Read about how it all works: [How Merge Warden works](../explanation/how-merge-warden-works.md)
