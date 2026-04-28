---
title: "How to deploy on AWS ECS / Fargate"
description: "Deploy the Merge Warden server container on AWS ECS Fargate with Secrets Manager."
---

# How to deploy on AWS ECS / Fargate

This guide deploys Merge Warden on [AWS ECS Fargate](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/what-is-fargate.html)
using AWS Secrets Manager for secret injection.

**Prerequisites:**

- AWS CLI configured with appropriate permissions
- An ECS cluster and a VPC with at least one subnet
- An Application Load Balancer (ALB) for HTTPS termination
- A GitHub App with private key — see [GitHub App permissions](../reference/github-app-permissions.md)

---

## 1 — Store secrets in AWS Secrets Manager

```bash
aws secretsmanager create-secret \
  --name merge-warden/github-app-id \
  --secret-string "123456"

aws secretsmanager create-secret \
  --name merge-warden/github-app-key \
  --secret-string "$(cat /path/to/private-key.pem)"

aws secretsmanager create-secret \
  --name merge-warden/github-webhook-secret \
  --secret-string "your-webhook-secret"
```

Note the ARN of each secret — you need them in the task definition.

---

## 2 — Create an IAM task execution role

The task execution role allows ECS to pull secrets from Secrets Manager at container startup.

```bash
aws iam create-role \
  --role-name ecsTaskExecutionRole-merge-warden \
  --assume-role-policy-document '{
    "Version": "2012-10-17",
    "Statement": [{
      "Effect": "Allow",
      "Principal": { "Service": "ecs-tasks.amazonaws.com" },
      "Action": "sts:AssumeRole"
    }]
  }'

aws iam attach-role-policy \
  --role-name ecsTaskExecutionRole-merge-warden \
  --policy-arn arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy

# Allow reading the three secrets
aws iam put-role-policy \
  --role-name ecsTaskExecutionRole-merge-warden \
  --policy-name SecretsAccess \
  --policy-document '{
    "Version": "2012-10-17",
    "Statement": [{
      "Effect": "Allow",
      "Action": "secretsmanager:GetSecretValue",
      "Resource": [
        "arn:aws:secretsmanager:<region>:<account>:secret:merge-warden/github-app-id*",
        "arn:aws:secretsmanager:<region>:<account>:secret:merge-warden/github-app-key*",
        "arn:aws:secretsmanager:<region>:<account>:secret:merge-warden/github-webhook-secret*"
      ]
    }]
  }'
```

---

## 3 — Register the ECS task definition

Save the following as `task-definition.json`, replacing `<region>` and `<account>` with
your values:

```json
{
  "family": "merge-warden",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512",
  "executionRoleArn": "arn:aws:iam::<account>:role/ecsTaskExecutionRole-merge-warden",
  "containerDefinitions": [
    {
      "name": "merge-warden",
      "image": "ghcr.io/pvandervelde/merge-warden-server:latest",
      "portMappings": [{ "containerPort": 3000, "protocol": "tcp" }],
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
        { "name": "RUST_LOG", "value": "info" }
      ],
      "healthCheck": {
        "command": ["CMD-SHELL", "curl -f http://localhost:3000/api/merge_warden || exit 1"],
        "interval": 30,
        "timeout": 5,
        "retries": 3,
        "startPeriod": 10
      },
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/merge-warden",
          "awslogs-region": "<region>",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

Register it:

```bash
aws ecs register-task-definition --cli-input-json file://task-definition.json
```

---

## 4 — Create the ECS service

```bash
aws ecs create-service \
  --cluster <your-cluster-name> \
  --service-name merge-warden \
  --task-definition merge-warden \
  --desired-count 1 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[<subnet-id>],securityGroups=[<sg-id>],assignPublicIp=ENABLED}" \
  --load-balancers "targetGroupArn=<target-group-arn>,containerName=merge-warden,containerPort=3000"
```

---

## 5 — Configure the ALB target group health check

In the AWS console or CLI, configure the ALB target group to check:

- **Protocol**: HTTP
- **Path**: `/api/merge_warden`
- **Success codes**: `200`

---

## 6 — Configure the GitHub App webhook

1. Get the ALB DNS name from the console or:

   ```bash
   aws elbv2 describe-load-balancers \
     --query "LoadBalancers[?LoadBalancerName=='<your-alb-name>'].DNSName" \
     --output text
   ```

2. In your GitHub App settings, set the **Webhook URL** to:

   ```
   https://<alb-dns-name>/api/merge_warden
   ```

3. Set **Content type** to `application/json`.

---

## Optional — Enable OTLP telemetry

Run the [AWS Distro for OpenTelemetry (ADOT)](https://aws-otel.github.io/) collector as a
sidecar container. Add the following environment variable to the merge-warden container
definition:

```json
{ "name": "OTEL_EXPORTER_OTLP_ENDPOINT", "value": "http://localhost:4318" }
```

The ADOT collector forwards traces and metrics to CloudWatch or X-Ray.

---

## Optional — Supply a policy config file

Mount a TOML policy file from S3 or EFS and set `MERGE_WARDEN_CONFIG_FILE` to its path
inside the container. See [Set application-level policy defaults](set-app-level-defaults.md).

---

## Related

- [Environment variables reference](../reference/environment-variables.md)
- [GitHub App permissions](../reference/github-app-permissions.md)
- [Set application-level policy defaults](set-app-level-defaults.md)
