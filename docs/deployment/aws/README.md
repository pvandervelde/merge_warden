# AWS Deployment Guide

Merge Warden is deployed as an OCI container image. On AWS the recommended host is
**ECS Fargate** — it runs the container without needing to manage EC2 instances and
integrates natively with AWS Secrets Manager for secret injection.

See the [container deployment guide](../README.md) for the full environment variable
reference before continuing.

---

## AWS ECS / Fargate

### Prerequisites

- AWS CLI configured with appropriate permissions
- An ECS cluster and VPC with at least one public subnet
- An Application Load Balancer (ALB) pointing to the ECS service (for HTTPS termination)

### Store secrets in AWS Secrets Manager

```bash
aws secretsmanager create-secret \
  --name merge-warden/github-app-id \
  --secret-string "12345"

aws secretsmanager create-secret \
  --name merge-warden/github-app-key \
  --secret-string "$(cat private-key.pem)"

aws secretsmanager create-secret \
  --name merge-warden/github-webhook-secret \
  --secret-string "supersecret"
```

### ECS Task Definition (excerpt)

```json
{
  "containerDefinitions": [
    {
      "name": "merge-warden",
      "image": "ghcr.io/pvandervelde/merge-warden-server:latest",
      "portMappings": [{ "containerPort": 3000 }],
      "secrets": [
        {
          "name": "MERGE_WARDEN_GITHUB_APP_ID",
          "valueFrom": "arn:aws:secretsmanager:<region>:<account>:secret:merge-warden/github-app-id"
        },
        {
          "name": "MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY",
          "valueFrom": "arn:aws:secretsmanager:<region>:<account>:secret:merge-warden/github-app-key"
        },
        {
          "name": "GITHUB_WEBHOOK_SECRET",
          "valueFrom": "arn:aws:secretsmanager:<region>:<account>:secret:merge-warden/github-webhook-secret"
        }
      ],
      "environment": [
        { "name": "RUST_LOG", "value": "info" },
        { "name": "OTEL_EXPORTER_OTLP_ENDPOINT", "value": "http://adot-collector:4318" }
      ],
      "healthCheck": {
        "command": ["CMD-SHELL", "curl -f http://localhost:3000/api/merge_warden || exit 1"],
        "interval": 30,
        "timeout": 5,
        "retries": 3,
        "startPeriod": 10
      }
    }
  ]
}
```

ECS injects each `secrets` entry as the named environment variable at runtime, so
the binary receives `MERGE_WARDEN_GITHUB_APP_ID`, `MERGE_WARDEN_GITHUB_APP_PRIVATE_KEY`, and
`GITHUB_WEBHOOK_SECRET` directly.

### OTLP telemetry on AWS

Run the [AWS Distro for OpenTelemetry (ADOT)](https://aws-otel.github.io/) collector
as a sidecar container. Set `OTEL_EXPORTER_OTLP_ENDPOINT` to the ADOT sidecar's OTLP
HTTP endpoint (default `http://localhost:4318`). The collector forwards traces and
metrics to CloudWatch or X-Ray.

---

## ALB health check

Configure the ALB target group health check to:

- **Protocol**: HTTP
- **Path**: `/api/merge_warden`
- **Success codes**: `200`

---

## GitHub webhook URL

After the ALB is provisioned, configure the GitHub App webhook to:

```
https://<alb-dns-name>/api/merge_warden
```

Set **Content type** to `application/json` and use the value from
`merge-warden/github-webhook-secret` as the webhook secret.

---

## Policy configuration

Mount a TOML policy file using an ECS volume (e.g., from S3 via a sidecar or EFS)
and set `MERGE_WARDEN_CONFIG_FILE` to its container path. If no file is provided,
compiled-in defaults apply.

---

## Contributing

If you build Terraform modules or CDK constructs for Merge Warden on AWS, contributions
are welcome. Review the [deployment README](../README.md) for the full environment
variable contract and open an issue or PR in the
[merge_warden repository](https://github.com/pvandervelde/merge_warden).
