<#
.SYNOPSIS
    Builds and runs the merge-warden-server container locally, then relays live
    GitHub App webhook events to it via smee.io.

.DESCRIPTION
    This script automates the local development loop for merge-warden-server:

      1. Builds the Docker image from the repository root (unless -SkipBuild).
      2. Starts the container and exposes it on the chosen local port.
      3. Polls the health endpoint until the server is ready.
      4. Runs the smee client to relay live GitHub App webhook events from your
         smee channel to the local server.  Because the events originate from
         GitHub's App webhook delivery, the payload includes the 'installation'
         object and all headers — including the original HMAC signature — so the
         server's signature validation passes without any re-signing.
      5. Stops the container automatically when you press Ctrl+C.

    Per-repository configuration is read by the server at event-processing time
    from '.github/merge-warden.toml' in the target repository (fetched via the
    GitHub API). If that file is absent, compiled-in defaults apply — both title
    and work-item validation are DISABLED by default, so the bot will run but
    will not enforce any policies until a per-repo config file exists.

    To observe real enforcement behaviour, add a '.github/merge-warden.toml' to
    your test repository. Use 'samples/merge-warden.sample.toml' as a starting
    point.

.PARAMETER SmeeUrl
    The smee.io channel URL to listen on (e.g. https://smee.io/abc123).
    This must match the Webhook URL configured in your GitHub App settings.
    Create a free channel at https://smee.io.

.PARAMETER Port
    Local TCP port to bind on the host. The container always listens on 3000
    internally; this controls which host port is mapped to it. Default: 3000.

.PARAMETER ImageTag
    Docker image tag to build and/or run. Default: merge-warden-server:local.

.PARAMETER AppConfigFile
    Optional path to a TOML configuration file on the host (e.g.
    '.\samples\app-config.sample.toml'). When supplied, the file is
    bind-mounted into the container at '/etc/merge-warden/config.toml' and
    MERGE_WARDEN_CONFIG_FILE is set accordingly.

    This acts as the application-level policy default. It is used for any
    repository that has no '.github/merge-warden.toml' of its own, and as the
    fallback baseline for values not specified in a per-repo config.

    When omitted, the server uses compiled-in defaults (all validations
    disabled).

.PARAMETER SkipBuild
    When set, skips the 'docker build' step and uses the existing local image.
    Useful for rapid iteration when only environment variables have changed.

.EXAMPLE
    # 1. Set required credentials
    $env:GITHUB_APP_ID          = "123456"
    $env:GITHUB_APP_PRIVATE_KEY = Get-Content ".\my-app.private-key.pem" -Raw
    $env:GITHUB_WEBHOOK_SECRET  = "my-local-webhook-secret"

    # 2. Run and relay webhooks
    .\samples\run-local.ps1 -SmeeUrl "https://smee.io/abc123"

.EXAMPLE
    # Supply an app-level config so policies are enforced even on repos without
    # their own .github/merge-warden.toml
    # Use app-config.sample.toml — NOT merge-warden.sample.toml (that is the
    # per-repo format and uses different field names).
    .\samples\run-local.ps1 -SmeeUrl "https://smee.io/abc123" -AppConfigFile ".\samples\app-config.sample.toml"

.EXAMPLE
    # Rebuild is slow; skip it on subsequent runs
    .\samples\run-local.ps1 -SmeeUrl "https://smee.io/abc123" -SkipBuild

.NOTES
    Requirements
    ------------
    - Docker Desktop (running)
    - Node.js + npm:  https://nodejs.org  (for npx smee-client)
      Or install globally once: npm install --global smee-client

    Required environment variables
    --------------------------------
    GITHUB_APP_ID           Numeric GitHub App ID (from your App settings page).
    GITHUB_APP_PRIVATE_KEY  Full PEM content of the RSA private key downloaded
                            from your GitHub App settings. Newlines must be
                            preserved — use 'Get-Content ... -Raw'.
    GITHUB_WEBHOOK_SECRET   Webhook signing secret set in your GitHub App.
                            Must match exactly — used for HMAC-SHA256 validation.

    GitHub App setup
    ----------------
    1. Create a channel at https://smee.io and copy the URL.
    2. In your GitHub App settings set Webhook URL to that smee URL.
    3. Set a Webhook secret and note the value.
    4. Set permissions: Pull requests (R/W), Contents (R), Checks (W), Metadata (R).
    5. Subscribe to events: Pull request, Pull request review.
    6. Install the App on your test repository.

    How smee relay works
    --------------------
    smee-client subscribes to your smee channel using Server-Sent Events (SSE).
    When GitHub delivers a webhook to the smee URL it stores the request;
    smee-client then forwards it — headers and body verbatim — to the local
    server.  Because the original HMAC-SHA256 signature computed by GitHub is
    forwarded unchanged, the server's signature validation passes correctly.
    Unlike 'gh webhook forward', no re-signing occurs and the full payload
    including the 'installation' object is preserved.
#>

[CmdletBinding()]
param (
    [Parameter(Mandatory)]
    [ValidatePattern('^https://')]
    [string] $SmeeUrl,

    [Parameter()]
    [ValidateRange(1, 65535)]
    [int] $Port = 3000,

    [Parameter()]
    [string] $ImageTag = 'merge-warden-server:local',

    [Parameter()]
    [string] $AppConfigFile,

    [Parameter()]
    [switch] $SkipBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# Validate required environment variables
# ---------------------------------------------------------------------------

$appId = $env:GITHUB_APP_ID
$privateKey = $env:GITHUB_APP_PRIVATE_KEY
$webhookSecret = $env:GITHUB_WEBHOOK_SECRET

if (-not $appId)
{
    throw "GITHUB_APP_ID environment variable is not set.`n" +
    "Set it to the numeric App ID from your GitHub App settings page."
}

if (-not $privateKey)
{
    throw "GITHUB_APP_PRIVATE_KEY environment variable is not set.`n" +
    "Set it to the full PEM content of your GitHub App private key:`n" +
    '  $env:GITHUB_APP_PRIVATE_KEY = Get-Content "path\to\app.private-key.pem" -Raw'
}

if (-not $webhookSecret)
{
    throw "GITHUB_WEBHOOK_SECRET environment variable is not set.`n" +
    "Set it to the webhook secret configured in your GitHub App settings."
}

# Resolve the optional app config file to an absolute path now so relative
# paths work regardless of the working directory when docker run is called.
$resolvedAppConfigFile = $null
if ($AppConfigFile)
{
    $resolvedAppConfigFile = Resolve-Path -Path $AppConfigFile -ErrorAction SilentlyContinue
    if (-not $resolvedAppConfigFile)
    {
        throw "AppConfigFile not found: $AppConfigFile"
    }
    $resolvedAppConfigFile = $resolvedAppConfigFile.Path
}

# ---------------------------------------------------------------------------
# Locate the repository root (directory that contains the workspace Cargo.toml)
# ---------------------------------------------------------------------------

$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path (Join-Path $repoRoot 'Cargo.toml')))
{
    throw "Could not locate the repository root. Expected Cargo.toml at: $repoRoot"
}

$dockerfile = Join-Path $repoRoot 'crates' 'server' 'Dockerfile'
if (-not (Test-Path $dockerfile))
{
    throw "Dockerfile not found at: $dockerfile"
}

# ---------------------------------------------------------------------------
# Verify external tools are available
# ---------------------------------------------------------------------------

foreach ($tool in 'docker', 'npx')
{
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue))
    {
        throw "'$tool' was not found on PATH. Please install it and try again.`n" +
        "  docker: https://www.docker.com/products/docker-desktop/`n" +
        "  npx:    ships with Node.js — https://nodejs.org"
    }
}

# ---------------------------------------------------------------------------
# Build the container image
# ---------------------------------------------------------------------------

if (-not $SkipBuild)
{
    Write-Host ""
    Write-Host "Building Docker image '$ImageTag' ..."
    Write-Host "  Context:    $repoRoot"
    Write-Host "  Dockerfile: $dockerfile"
    Write-Host ""

    docker build -f $dockerfile -t $ImageTag $repoRoot

    if ($LASTEXITCODE -ne 0)
    {
        throw "docker build failed with exit code $LASTEXITCODE."
    }

    Write-Host ""
    Write-Host "Image '$ImageTag' built successfully."
}
else
{
    Write-Host "Skipping build — using existing image '$ImageTag'."
}

# ---------------------------------------------------------------------------
# Start the container
# ---------------------------------------------------------------------------
#
# The PEM private key may contain literal newlines; passing it via environment
# variable inheritance (-e VAR without =value) is the most reliable way to
# preserve them through Docker on Windows.
#
$env:GITHUB_APP_PRIVATE_KEY = $privateKey

Write-Host ""
Write-Host "Starting container on port $Port ..."

# Build the docker run argument list dynamically so the optional volume mount
# and config env var are only added when -AppConfigFile is supplied.
#
# Note: --rm is intentionally omitted. The container is removed explicitly in
# the finally block. Keeping the container around after an early exit allows
# 'docker logs' to retrieve startup error messages for diagnosis.
$dockerRunArgs = @(
    '-d',
    '-p', "${Port}:3000",
    '-e', 'GITHUB_APP_ID',
    '-e', 'GITHUB_APP_PRIVATE_KEY',
    '-e', 'GITHUB_WEBHOOK_SECRET',
    '-e', 'MERGE_WARDEN_RECEIVER_MODE=webhook'
)

if ($resolvedAppConfigFile)
{
    $containerConfigPath = '/etc/merge-warden/config.toml'
    $dockerRunArgs += '-v'
    $dockerRunArgs += "${resolvedAppConfigFile}:${containerConfigPath}:ro"
    $dockerRunArgs += '-e'
    $dockerRunArgs += "MERGE_WARDEN_CONFIG_FILE=${containerConfigPath}"
    Write-Host "  App config: $resolvedAppConfigFile"
    Write-Host "            → $containerConfigPath"
}
else
{
    Write-Host "  App config: none (compiled-in defaults apply)"
}

$containerId = docker run @dockerRunArgs $ImageTag

if ($LASTEXITCODE -ne 0 -or -not $containerId)
{
    throw "docker run failed. Check the output above for details."
}

$containerId = $containerId.Trim()
Write-Host "Container ID: $containerId"

# Give the server a moment to either start or crash before polling.
Start-Sleep -Seconds 2

# Detect an immediate crash before entering the health-check loop.
$containerState = docker inspect --format '{{.State.Status}}' $containerId 2>$null
if ($containerState -ne 'running')
{
    Write-Warning "Container exited immediately (state: $containerState)."
    Write-Host ""
    Write-Host "Container logs:"
    docker logs $containerId
    docker rm $containerId | Out-Null
    throw "Container crashed at startup — see logs above.`n" +
    "Common causes:`n" +
    "  - GITHUB_APP_ID / GITHUB_APP_PRIVATE_KEY / GITHUB_WEBHOOK_SECRET not set`n" +
    "  - GITHUB_APP_PRIVATE_KEY contains an invalid RSA PEM`n" +
    "  - MERGE_WARDEN_CONFIG_FILE points to a file the server cannot parse"
}

# ---------------------------------------------------------------------------
# Health check — wait until the server is ready
# ---------------------------------------------------------------------------

$healthUrl = "http://localhost:$Port/api/merge_warden"
$maxRetries = 20
$attempt = 0
$ready = $false

Write-Host ""
Write-Host "Waiting for server to be ready at $healthUrl ..."

while ($attempt -lt $maxRetries)
{
    Start-Sleep -Seconds 1
    try
    {
        $response = Invoke-WebRequest -Uri $healthUrl -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
        if ($response.StatusCode -eq 200)
        {
            $ready = $true
            break
        }
    }
    catch
    {
        # Server not ready yet — keep polling.
    }
    $attempt++
    Write-Host "  Attempt $attempt / $maxRetries ..."
}

if (-not $ready)
{
    Write-Warning "Server did not respond within $maxRetries seconds."
    Write-Host ""
    Write-Host "Container logs:"
    docker logs $containerId
    docker rm -f $containerId | Out-Null
    throw "Container failed to start. See logs above."
}

Write-Host "Server is ready."

# ---------------------------------------------------------------------------
# Relay webhooks via smee — blocks until Ctrl+C
# ---------------------------------------------------------------------------

Write-Host ""
Write-Host "--------------------------------------------------------"
Write-Host "Relaying webhooks from smee channel:"
Write-Host "  $SmeeUrl"
Write-Host "Local endpoint: $healthUrl"
Write-Host ""
Write-Host "To trigger events:"
Write-Host "  - Open a pull request in your test repository"
Write-Host "  - Edit the PR title or description"
Write-Host "  - Submit a pull request review"
Write-Host ""
if ($resolvedAppConfigFile)
{
    Write-Host "App config supplied — policies in that file apply to repos"
    Write-Host "that have no .github/merge-warden.toml of their own."
}
else
{
    Write-Host "No app config supplied. Compiled-in defaults apply to repos"
    Write-Host "without .github/merge-warden.toml (all validations disabled)."
    Write-Host "Re-run with -AppConfigFile samples\app-config.sample.toml"
    Write-Host "to enable enforcement without a per-repo config."
}
Write-Host ""
Write-Host "Press Ctrl+C to stop."
Write-Host "--------------------------------------------------------"
Write-Host ""

try
{
    # Use a globally installed smee client if available, otherwise fall back to
    # npx so no global install is required.
    $smeeCmd = if (Get-Command smee -ErrorAction SilentlyContinue)
    {
        'smee' 
    }
    else
    {
        $null 
    }

    if ($smeeCmd)
    {
        & $smeeCmd --url $SmeeUrl --target $healthUrl
    }
    else
    {
        npx smee-client --url $SmeeUrl --target $healthUrl
    }
}
finally
{
    Write-Host ""
    Write-Host "Stopping container $containerId ..."
    docker stop $containerId | Out-Null
    docker rm $containerId | Out-Null
    Write-Host "Done."
}
