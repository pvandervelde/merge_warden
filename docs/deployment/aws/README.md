# AWS Deployment Guide

*This deployment method is planned for a future release.*

## Overview

AWS deployment support for Merge Warden is planned but not yet implemented. The AWS deployment will provide similar functionality to the Azure deployment using AWS Lambda and supporting services.

## Planned Architecture

The AWS deployment will use:

- **AWS Lambda** - Function runtime for processing GitHub webhooks
- **API Gateway** - HTTP endpoint for webhook reception
- **Systems Manager Parameter Store** - Configuration management
- **Secrets Manager** - Secure storage for GitHub App credentials
- **CloudWatch** - Logging and monitoring
- **S3** - Storage for Lambda deployment packages

## Timeline

AWS support is targeted for a future release. Track progress in the [GitHub issues](https://github.com/pvandervelde/merge_warden/issues) with the `aws` label.

## Contributing

If you're interested in contributing AWS deployment support:

1. Review the existing Azure implementation for patterns
2. Create an issue to discuss the implementation approach
3. Follow the contribution guidelines in the repository

## Alternative Solutions

Until native AWS support is available, consider:

1. **Containerized deployment** - Deploy the Azure Function binary in a container on AWS ECS or EKS
2. **Cross-cloud deployment** - Use the Azure deployment while running other infrastructure on AWS
3. **Custom implementation** - Use the `merge_warden_core` crate to build your own AWS Lambda function
