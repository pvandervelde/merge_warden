---
title: "How to run Merge Warden in queue mode"
description: "Set up the separate receiver service and queue infrastructure that queue receiver mode requires."
---

# How to run Merge Warden in queue mode

`MERGE_WARDEN_RECEIVER_MODE=queue` turns the Merge Warden container into a **pure queue
consumer**. It does not expose a webhook POST endpoint — only `GET /health` is registered.
You must provide a **separate service** that receives the GitHub webhook, verifies its
HMAC signature, and enqueues the payload for Merge Warden to consume.

If you have not already read
[Webhook vs queue receiver modes](../explanation/receiver-modes.md), start there — this
guide assumes you understand why queue mode is a two-service architecture, not a single
container with a mode flag.

---

## What you need to build or provision

Queue mode requires a component that this repository does not ship a ready-made
implementation of: **the webhook receiver service**. At minimum it must:

1. Accept an HTTPS POST from GitHub at whatever URL you configure as the GitHub App's
   **Webhook URL**.
2. Verify the `X-Hub-Signature-256` header against your webhook secret using HMAC-SHA256
   (constant-time comparison). Reject invalid or missing signatures with `401`.
3. Publish the validated payload to the same queue (name, provider, and region/namespace)
   that the Merge Warden container is configured to consume from.
4. Return `200`/`202` to GitHub promptly, within GitHub's 10-second webhook delivery
   timeout.

Common ways to build this: a small serverless function (Azure Function, AWS Lambda) that
validates the signature and forwards to Azure Service Bus or SQS, or a lightweight sidecar
container. The receiver service's implementation is outside Merge Warden's scope — Merge
Warden only defines the consumer side.

---

## Choosing a queue provider

Set `MERGE_WARDEN_QUEUE_PROVIDER` to one of:

| Value | Backend | Use case |
| :--- | :--- | :--- |
| `azure` | Azure Service Bus | Production deployments on Azure |
| `aws` | AWS SQS | Production deployments on AWS |
| `memory` | In-process in-memory queue | Local development and testing only — **not durable**, not shared across processes, do not use in production |

Any other value is rejected at startup with a configuration error.

> **Planned, not yet available:** the underlying `queue-runtime` library Merge Warden
> depends on also has the ability to talk to NATS and RabbitMQ, but Merge Warden's own
> configuration loader does not yet expose them — there is no `nats` or `rabbitmq` value
> for `MERGE_WARDEN_QUEUE_PROVIDER` today. Setting either one currently fails startup with
> `Unknown queue provider '<value>'. Expected 'azure', 'aws', or 'memory'.` Use `azure` or
> `aws` until this is wired up.

---

## Configure the Merge Warden consumer — Azure Service Bus

```bash
docker run --rm \
  -e MERGE_WARDEN_GITHUB_APP_ID=12345 \
  -e MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY="$(cat private-key.pem)" \
  -e MERGE_WARDEN_RECEIVER_MODE=queue \
  -e MERGE_WARDEN_QUEUE_PROVIDER=azure \
  -e MERGE_WARDEN_QUEUE_NAME=merge-warden-events \
  -e MERGE_WARDEN_QUEUE_CONCURRENCY=4 \
  -e AZURE_SERVICEBUS_NAMESPACE=my-namespace.servicebus.windows.net \
  -p 3000:3000 \
  ghcr.io/pvandervelde/merge-warden-server:latest
```

Notes:

- `GITHUB_WEBHOOK_SECRET` is **not** set here — it is not read in queue mode. The receiver
  service holds and validates the webhook secret.
- Authentication to Azure Service Bus uses the default Azure credential chain (managed
  identity when running in Azure, `az login` locally, or environment-based credentials).
  There is no connection-string environment variable for the consumer side; grant the
  container's identity a Service Bus data-plane role (e.g. **Azure Service Bus Data
  Receiver**) on the namespace.
- Your receiver service needs its own credential to *send* to the same namespace/queue
  (e.g. **Azure Service Bus Data Sender**).

---

## Configure the Merge Warden consumer — AWS SQS

```bash
docker run --rm \
  -e MERGE_WARDEN_GITHUB_APP_ID=12345 \
  -e MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY="$(cat private-key.pem)" \
  -e MERGE_WARDEN_RECEIVER_MODE=queue \
  -e MERGE_WARDEN_QUEUE_PROVIDER=aws \
  -e MERGE_WARDEN_QUEUE_NAME=merge-warden-events \
  -e MERGE_WARDEN_QUEUE_CONCURRENCY=4 \
  -e AWS_REGION=ap-southeast-2 \
  -p 3000:3000 \
  ghcr.io/pvandervelde/merge-warden-server:latest
```

Notes:

- Prefer an IAM role (ECS task role, IRSA on EKS, EC2 instance profile) over
  `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`. Only set the static-key variables for local
  testing against a real SQS queue.
- Grant the container's role permission to receive and delete messages on the target SQS
  queue. Grant your receiver service permission to send messages to the same queue.

---

## Local testing — in-memory queue

```bash
docker run --rm \
  -e MERGE_WARDEN_GITHUB_APP_ID=12345 \
  -e MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY="$(cat private-key.pem)" \
  -e MERGE_WARDEN_RECEIVER_MODE=queue \
  -e MERGE_WARDEN_QUEUE_PROVIDER=memory \
  -p 3000:3000 \
  ghcr.io/pvandervelde/merge-warden-server:latest
```

The in-memory provider only exists within a single running process and is not durable
across restarts. It is useful for exercising the queue-consumer code path locally but
cannot receive messages from an external receiver service running in a different process.
Use `azure` or `aws` (even against a dev/test namespace or queue) to test the full
two-service flow end-to-end.

---

## Verifying the deployment

`GET /health` is the only route available in queue mode:

```bash
curl -i http://localhost:3000/health
# HTTP/1.1 200 OK
```

To confirm events are flowing, open or update a pull request in a repository the GitHub
App is installed on, and check:

1. Your receiver service's logs — did it accept the webhook and successfully enqueue?
2. The Merge Warden container's logs — did it dequeue and process the message (labels
   applied, check run updated)?
3. The pull request's **Checks** tab on GitHub, with a short delay relative to webhook mode.

---

## Related

- [Webhook vs queue receiver modes](../explanation/receiver-modes.md)
- [Environment variables reference](../reference/environment-variables.md)
- [HTTP endpoints reference](../reference/http-endpoints.md)
- [Deploy on Azure Container Apps](deploy-on-azure.md)
- [Deploy on AWS ECS / Fargate](deploy-on-aws.md)
