# Azure Function Configuration Samples

This directory contains sample configuration files for deploying the Merge Warden Azure Function. These files are **not included** in the deployment package and must be configured by users according to their specific requirements.

## Required Configuration Files

When deploying the Merge Warden Azure Function, you need to create the following configuration files in your Azure Function App:

### 1. host.json (Required)

**Sample file**: `host.json`

The `host.json` file configures the Azure Functions runtime. Place this file in the root directory of your Azure Function App.

**Key configuration areas:**

- **Logging levels**: Configure what level of detail you want in logs
- **Application Insights**: Set up telemetry and monitoring (optional)
- **Extension bundles**: Specify which Azure Functions extensions to use
- **Custom handler**: Configuration for the merge_warden binary (required)

**Customization options:**

- Set `logging.logLevel.default` to control verbosity (`Trace`, `Debug`, `Information`, `Warning`, `Error`)
- Configure Application Insights sampling and excluded request types
- Adjust extension bundle versions based on your Azure environment

### 2. function.json (Required)

**Sample file**: `function.json`

The `function.json` file defines the HTTP trigger configuration. Place this file in a subdirectory named after your function app, e.g. `your-function-app/`, within your Function App directory.

**Directory structure in your Function App:**

```
/
├── host.json
├── az_handler (binary, included in deployment package)
└── your-function-app/
    └── function.json
```

**Key configuration areas:**

- **Authentication level**: Currently set to `anonymous` but you may want to configure function-level security
- **HTTP methods**: Configured for `GET` and `POST` to handle GitHub webhooks
- **Trigger bindings**: HTTP input and output bindings

**Customization options:**

- Change `authLevel` from `anonymous` to `function`, `admin`, or use Azure AD authentication
- Modify allowed HTTP methods if needed
- Add additional bindings for integration with other Azure services

## Deployment Instructions

1. **Download the deployment binary** from the [GitHub releases](https://github.com/pvandervelde/merge_warden/releases)

2. **Create your configuration files** based on the samples in this directory:

   ```bash
   # Copy and customize the host.json
   cp samples/host.json ./host.json

   # Create the function directory and copy function.json
   mkdir merge_warden
   cp samples/function.json ./merge_warden/function.json
   ```

3. **Customize the configuration** according to your requirements:
   - Edit logging levels in `host.json`
   - Configure Application Insights connection if desired
   - Set authentication level in `function.json`

4. **Deploy to Azure Functions**:

   ```bash
   # Create deployment directory
   mkdir function-app

   # Copy the binary (make it executable)
   cp az_handler function-app/
   chmod +x function-app/az_handler

   # Copy your configuration files
   cp host.json function-app/
   cp -r merge_warden/ function-app/

   # Create deployment package
   cd function-app
   zip -r ../function-app.zip .
   cd ..

   # Deploy using Azure CLI
   az functionapp deployment source config-zip \
     --resource-group your-rg \
     --name your-function-app \
     --src function-app.zip
   ```

## Security Considerations

- **Authentication**: Consider using function-level or Azure AD authentication instead of anonymous access
- **CORS**: Configure CORS settings if your function will be called from web applications
- **IP restrictions**: Use Azure Function App access restrictions to limit access to specific IP ranges
- **Secrets**: Store sensitive configuration in Azure Key Vault and reference via Azure App Configuration

## Monitoring and Troubleshooting

- **Application Insights**: Enable Application Insights for comprehensive monitoring and logging
- **Log streaming**: Use `az functionapp log tail` to view real-time logs during development
- **Diagnostic settings**: Configure diagnostic settings to send logs to Log Analytics or Storage

## Support

For configuration questions and deployment issues:

1. Check the [deployment documentation](../docs/deployment/azure/README.md)
2. Review the Azure Functions documentation for host.json and function.json configuration options
3. Open an issue in the [merge_warden repository](https://github.com/pvandervelde/merge_warden/issues)
