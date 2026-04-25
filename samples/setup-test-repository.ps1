<#
.SYNOPSIS
    Creates a GitHub repository in an organisation and pushes test branches that
    cover every merge-warden validation scenario.

.DESCRIPTION
    This script provisions a clean test repository in a GitHub organisation,
    creates branches whose names describe the merge-warden scenario they are
    designed to exercise, and opens a draft pull request for each one so
    merge-warden can process it immediately as a real webhook event.

    It also creates:
      - Repository labels (size:, type:, status:, pr-issue: groups)
      - Two milestones: v1.0 and v2.0
      - Two issues: a generic work-item reference and a propagation test issue
      - A Projects v2 board with the propagation issue pre-added

    Scenarios created
    -----------------
    Title validation
      test/title-valid-feat          — valid conventional commit title
      test/title-valid-fix           — valid fix title with scope
      test/title-invalid-no-type     — missing commit type prefix
      test/title-invalid-no-desc     — type present but no description
      test/title-breaking-change     — valid breaking change (type!: desc)

    Work-item validation
      test/workitem-fixes-hash       — "fixes #1" reference
      test/workitem-closes-url       — full GitHub issue URL reference
      test/workitem-missing          — no work-item reference at all

    PR size
      test/size-xs                   — 1–10 lines changed (1 file, 2 lines)
      test/size-s                    — 11–50 lines changed
      test/size-m                    — 51–100 lines changed
      test/size-l                    — 101–250 lines changed
      test/size-xl                   — 251–500 lines changed
      test/size-xxl                  — 500+ lines changed

    WIP detection
      test/wip-title-prefix          — title starts with "WIP:"
      test/wip-title-bracket         — title contains "[WIP]"
      test/wip-ready                 — title has no WIP markers

    Change-type labels (conventional commit types)
      test/change-type-feat          — feat commit type → enhancement label
      test/change-type-fix           — fix commit type → bug label
      test/change-type-docs          — docs commit type → documentation label
      test/change-type-chore         — chore commit type → maintenance label
      test/change-type-refactor      — refactor commit type → refactoring label
      test/change-type-ci            — ci commit type → CI label
      test/change-type-perf          — perf commit type → performance label
      test/change-type-test          — test commit type → testing label
      test/change-type-build         — build commit type → dependencies label
      test/change-type-style         — style commit type → formatting label
      test/change-type-revert        — revert commit type → revert label

    PR state / lifecycle labels
      test/state-draft               — open as a draft PR → "status: draft"
      test/state-ready-for-review    — open as ready → "status: in-review"

    Issue-propagation (milestone + project)
      test/issue-propagation         — body references an issue that has a
                                       milestone and is in a Projects v2 board

    Bypass rules
      test/bypass-title-user         — invalid title but author is in bypass list
      test/bypass-workitem-user      — missing work item but author is in bypass list

.PARAMETER Org
    GitHub organisation name where the repository will be created.

.PARAMETER RepoName
    Name of the new repository. Defaults to "merge-warden-test".

.PARAMETER Description
    Repository description. Defaults to a sensible test description.

.PARAMETER Private
    When present, creates a private repository. Defaults to public.

.PARAMETER SkipRepoCreation
    Skip creating the repository (use when it already exists).

.PARAMETER CleanupOnExit
    When present, deleted the local clone directory on exit.

.EXAMPLE
    .\samples\setup-test-repository.ps1 -Org myorg

.EXAMPLE
    .\samples\setup-test-repository.ps1 -Org myorg -RepoName mw-test -Private

.EXAMPLE
    .\samples\setup-test-repository.ps1 -Org myorg -SkipRepoCreation
#>
[CmdletBinding()]
param (
    [Parameter(Mandatory = $true)]
    [string] $Org,

    [Parameter(Mandatory = $false)]
    [string] $RepoName = 'merge-warden-test',

    [Parameter(Mandatory = $false)]
    [string] $Description = 'Test repository for merge-warden integration testing',

    [switch] $Private,

    [switch] $SkipRepoCreation,

    [switch] $CleanupOnExit
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

function Write-Step([string]$Message)
{
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Write-Ok([string]$Message)
{
    Write-Host "    OK  $Message" -ForegroundColor Green
}

function Write-Info([string]$Message)
{
    Write-Host "    ... $Message" -ForegroundColor Gray
}

function Invoke-Git
{
    param([string[]]$Arguments)
    & git @Arguments
    if ($LASTEXITCODE -ne 0)
    {
        throw "git $($Arguments -join ' ') exited with code $LASTEXITCODE"
    }
}

# Generates a file large enough to accumulate roughly $LineCount lines.
function New-FileWithLines
{
    param(
        [string] $Path,
        [int]    $LineCount,
        [string] $Prefix = 'line'
    )
    $lines = 1..$LineCount | ForEach-Object { "$($Prefix)-$(${_}): $(New-Guid)" }
    Set-Content -Path $Path -Value $lines -Encoding UTF8
}

# Creates a GitHub milestone via the REST API and returns its number.
function New-GitHubMilestone
{
    param(
        [string] $Title,
        [string] $MilestoneDescription = ''
    )
    Write-Info "Milestone: $Title"
    $number = gh api "repos/$repoFullName/milestones" `
        --method POST `
        -f title="$Title" `
        -f description="$MilestoneDescription" `
        --jq '.number'
    if ($LASTEXITCODE -ne 0)
    {
        Write-Warning "Failed to create milestone '$Title' (exit $LASTEXITCODE) — continuing"
        return 0
    }
    return [int]$number
}

# Creates a GitHub issue via the REST API and returns its number.
function New-GitHubIssue
{
    param(
        [string] $Title,
        [string] $IssueBody = '',
        [int]    $MilestoneNumber = 0
    )
    Write-Info "Issue: $Title"
    $apiArgs = @('api', "repos/$repoFullName/issues", '--method', 'POST',
        '-f', "title=$Title", '-f', "body=$IssueBody", '--jq', '.number')
    if ($MilestoneNumber -gt 0)
    {
        $apiArgs += @('-F', "milestone=$MilestoneNumber")
    }
    $number = gh @apiArgs
    if ($LASTEXITCODE -ne 0)
    {
        Write-Warning "Failed to create issue '$Title' (exit $LASTEXITCODE) — continuing"
        return 0
    }
    return [int]$number
}

# Creates a GitHub Projects v2 board in the organisation and returns its node ID.
function New-GitHubProject
{
    param([string] $Title)
    Write-Info "Project: $Title"
    $orgId = gh api graphql `
        -f query="{ organization(login: `"$Org`") { id } }" `
        --jq '.data.organization.id'
    if ($LASTEXITCODE -ne 0 -or -not $orgId)
    {
        Write-Warning "Failed to resolve organisation node ID — skipping project creation"
        return $null
    }
    $projectId = gh api graphql `
        -f query='mutation($ownerId: ID!, $title: String!) { createProjectV2(input: { ownerId: $ownerId, title: $title }) { projectV2 { id } } }' `
        -f ownerId="$orgId" `
        -f title="$Title" `
        --jq '.data.createProjectV2.projectV2.id'
    if ($LASTEXITCODE -ne 0 -or -not $projectId)
    {
        Write-Warning "Failed to create project '$Title' — skipping"
        return $null
    }
    return $projectId
}

# Adds a repository issue to a Projects v2 board by issue number.
function Add-IssueToProject
{
    param(
        [string] $ProjectId,
        [int]    $IssueNumber
    )
    Write-Info "Adding issue #$IssueNumber to project"
    $issueNodeId = gh api graphql `
        -f query="{ repository(owner: `"$Org`", name: `"$RepoName`") { issue(number: $IssueNumber) { id } } }" `
        --jq '.data.repository.issue.id'
    if ($LASTEXITCODE -ne 0 -or -not $issueNodeId)
    {
        Write-Warning "Failed to resolve node ID for issue #$IssueNumber — skipping"
        return
    }
    $null = gh api graphql `
        -f query='mutation($projectId: ID!, $contentId: ID!) { addProjectV2ItemById(input: { projectId: $projectId, contentId: $contentId }) { item { id } } }' `
        -f projectId="$ProjectId" `
        -f contentId="$issueNodeId"
    if ($LASTEXITCODE -ne 0)
    {
        Write-Warning "Failed to add issue #$IssueNumber to project — continuing"
    }
}

# Creates or updates a GitHub label; uses --force so re-runs are idempotent.
function New-GitHubLabel
{
    param(
        [string] $Name,
        [string] $Color,
        [string] $LabelDescription = ''
    )
    Write-Info "Label: $Name"
    $labelArgs = @('label', 'create', $Name, '--color', $Color, '--force', '--repo', $repoFullName)
    if ($LabelDescription)
    {
        $labelArgs += @('--description', $LabelDescription)
    }
    gh @labelArgs
    if ($LASTEXITCODE -ne 0)
    {
        Write-Warning "Failed to create/update label '$Name' (exit $LASTEXITCODE) — continuing"
    }
}

# ---------------------------------------------------------------------------
# Pre-flight checks
# ---------------------------------------------------------------------------

Write-Step 'Checking prerequisites'

foreach ($cmd in 'git', 'gh')
{
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue))
    {
        throw "'$cmd' is not on PATH. Install it and re-run."
    }
}

# Verify gh is authenticated
$null = gh auth status 2>&1
if ($LASTEXITCODE -ne 0)
{
    throw "'gh' is not authenticated. Run 'gh auth login' first."
}

Write-Ok 'git and gh are available and authenticated'

# ---------------------------------------------------------------------------
# Create the repository (unless skipped)
# ---------------------------------------------------------------------------

$repoFullName = "$Org/$RepoName"

if (-not $SkipRepoCreation)
{
    Write-Step "Creating repository $repoFullName"

    $visibilityFlag = if ($Private)
    {
        '--private'
    }
    else
    {
        '--public'
    }

    gh repo create $repoFullName `
        $visibilityFlag `
        --description $Description `
        --add-readme

    if ($LASTEXITCODE -ne 0)
    {
        throw "Failed to create repository $repoFullName"
    }
    Write-Ok "Repository $repoFullName created"
}
else
{
    Write-Info "Skipping repository creation — assuming $repoFullName already exists"
}

# ---------------------------------------------------------------------------
# Clone locally
# ---------------------------------------------------------------------------

Write-Step 'Cloning repository'

$cloneDir = Join-Path ([System.IO.Path]::GetTempPath()) "mw-test-setup-$RepoName"

if (Test-Path $cloneDir)
{
    Remove-Item $cloneDir -Recurse -Force
}

gh repo clone $repoFullName $cloneDir
if ($LASTEXITCODE -ne 0)
{
    throw "Failed to clone $repoFullName"
}

Push-Location $cloneDir

try
{
    # Set a local git identity so commits are attributed cleanly
    Invoke-Git @('config', 'user.email', 'merge-warden-test-setup@local')
    Invoke-Git @('config', 'user.name', 'merge-warden test setup')

    # Determine the default branch name (usually "main" or "master")
    $defaultBranch = git symbolic-ref --short HEAD 2>$null
    if (-not $defaultBranch)
    {
        $defaultBranch = 'main'
    }
    Write-Info "Default branch: $defaultBranch"

    # Make sure there is at least one commit on default (gh --add-readme covers
    # this, but guard for --SkipRepoCreation cases where the repo may be empty).
    $commitCount = (git rev-list --count HEAD 2>$null)
    if (-not $commitCount -or $commitCount -eq '0')
    {
        Write-Info 'No commits found — creating initial commit'
        Set-Content -Path 'README.md' -Value "# $RepoName`n`nMerge-warden integration test repository." -Encoding UTF8
        Invoke-Git @('add', 'README.md')
        Invoke-Git @('commit', '-m', 'chore: initial commit')
        Invoke-Git @('push', 'origin', $defaultBranch)
    }

    # Ensure the merge-warden config exists on the default branch so every PR
    # picks it up automatically.
    Write-Step 'Adding .github/merge-warden.toml on default branch'
    $githubDir = Join-Path $cloneDir '.github'
    $null = New-Item -ItemType Directory -Path $githubDir -Force

    # A permissive config that enables every feature so all scenarios are visible.
    $mergeWardenConfig = @'
schemaVersion = 1

[policies.pullRequests.prTitle]
required = true
label_if_missing = "pr-issue: invalid-title-format"

[policies.pullRequests.workItem]
required = true
label_if_missing = "pr-issue: missing-work-item"

[policies.pullRequests.prSize]
enabled = true
fail_on_oversized = false
excluded_file_patterns = ["*.md", "*.txt", "docs/*"]
label_prefix = "size: "
add_comment = true

[policies.pullRequests.wip]
enforce_wip_blocking = true
wip_label = "status: work-in-progress"
wip_title_patterns = ["WIP", "wip:", "[wip]", "draft:", "Draft:"]
wip_description_patterns = []

[policies.pullRequests.issuePropagation]
sync_milestone_from_issue = true
sync_project_from_issue = true

[policies.pullRequests.prState]
enabled = true
draft_label    = "status: draft"
review_label   = "status: in-review"
approved_label = "status: approved"

[policies.pullRequests.bypassRules.title_convention]
enabled = true
users = ["merge-warden-bypass-user"]

[policies.pullRequests.bypassRules.work_items]
enabled = true
users = ["merge-warden-bypass-user"]

[change_type_labels]
enabled = true

[change_type_labels.conventional_commit_mappings]
feat     = ["type: enhancement", "type: feature"]
fix      = ["type: bug", "type: bugfix", "type: fix"]
docs     = ["type: documentation", "type: docs"]
style    = ["type: style", "type: formatting"]
refactor = ["type: refactor", "type: refactoring", "type: code quality"]
perf     = ["type: performance", "type: optimization"]
test     = ["type: test", "type: tests", "type: testing"]
chore    = ["type: chore", "type: maintenance", "type: housekeeping"]
ci       = ["type: ci", "type: continuous integration", "type: build"]
build    = ["type: build", "type: dependencies"]
revert   = ["type: revert"]

[change_type_labels.fallback_label_settings]
name_format       = "type: {change_type}"
create_if_missing = true

[change_type_labels.fallback_label_settings.color_scheme]
feat     = "#e4e669"
fix      = "#e4e669"
docs     = "#e4e669"
style    = "#e4e669"
refactor = "#e4e669"
perf     = "#e4e669"
test     = "#e4e669"
chore    = "#e4e669"
ci       = "#e4e669"
build    = "#e4e669"
revert   = "#e4e669"

[change_type_labels.detection_strategy]
exact_match       = true
prefix_match      = true
description_match = true
common_prefixes   = ["type:", "kind:", "category:"]
'@

    Set-Content -Path (Join-Path $githubDir 'merge-warden.toml') -Value $mergeWardenConfig -Encoding UTF8
    Invoke-Git @('add', '.github/merge-warden.toml')
    Invoke-Git @('commit', '-m', 'chore: add merge-warden configuration for integration tests')
    Invoke-Git @('push', 'origin', $defaultBranch)
    Write-Ok '.github/merge-warden.toml pushed'

    # ------------------------------------------------------------------
    # Create repository labels up front so all scenarios have labels to
    # match against (and merge-warden can find them by name).
    # ------------------------------------------------------------------
    Write-Step 'Creating repository labels'

    # PR size labels — must match label_prefix = "size: " in the config.
    # All share the same light-blue colour so they are visually grouped.
    New-GitHubLabel 'size: XS'  'c5def5' 'PR size: 1–10 lines changed'
    New-GitHubLabel 'size: S'   'c5def5' 'PR size: 11–50 lines changed'
    New-GitHubLabel 'size: M'   'c5def5' 'PR size: 51–100 lines changed'
    New-GitHubLabel 'size: L'   'c5def5' 'PR size: 101–250 lines changed'
    New-GitHubLabel 'size: XL'  'c5def5' 'PR size: 251–500 lines changed'
    New-GitHubLabel 'size: XXL' 'c5def5' 'PR size: 500+ lines changed'

    # Change-type labels — all share yellow so they form one visual group.
    # Colour must match fallback_label_settings.color_scheme in the TOML config.
    New-GitHubLabel 'type: feat'     'e4e669' 'Change type: new feature'
    New-GitHubLabel 'type: fix'      'e4e669' 'Change type: bug fix'
    New-GitHubLabel 'type: docs'     'e4e669' 'Change type: documentation'
    New-GitHubLabel 'type: style'    'e4e669' 'Change type: formatting / style'
    New-GitHubLabel 'type: refactor' 'e4e669' 'Change type: code refactoring'
    New-GitHubLabel 'type: perf'     'e4e669' 'Change type: performance improvement'
    New-GitHubLabel 'type: test'     'e4e669' 'Change type: tests'
    New-GitHubLabel 'type: chore'    'e4e669' 'Change type: chore / maintenance'
    New-GitHubLabel 'type: ci'       'e4e669' 'Change type: CI/CD pipeline'
    New-GitHubLabel 'type: build'    'e4e669' 'Change type: build system / dependencies'
    New-GitHubLabel 'type: revert'   'e4e669' 'Change type: revert a previous commit'

    # Validation-failure labels — all red so problems are immediately visible.
    New-GitHubLabel 'pr-issue: invalid-title-format' 'd73a4a' 'PR title does not follow the conventional commit format'
    New-GitHubLabel 'pr-issue: missing-work-item'    'd73a4a' 'PR description contains no work-item reference'

    # PR state lifecycle labels — all lavender so state is visually grouped.
    New-GitHubLabel 'status: draft'            'd4c5f9' 'PR is a draft'
    New-GitHubLabel 'status: in-review'        'd4c5f9' 'PR is ready and awaiting review'
    New-GitHubLabel 'status: approved'         'd4c5f9' 'PR has been approved'
    New-GitHubLabel 'status: work-in-progress' 'd4c5f9' 'PR is a work in progress'

    Write-Ok 'Labels created'

    # ------------------------------------------------------------------
    # Create milestones, issues and a Projects v2 board so the
    # issue-propagation scenario has real data to exercise.
    # ------------------------------------------------------------------
    Write-Step 'Creating milestones'

    $milestoneV1 = New-GitHubMilestone `
        -Title                'v1.0 — Initial Release' `
        -MilestoneDescription 'First production release'
    $milestoneV2 = New-GitHubMilestone `
        -Title                'v2.0 — Next Release' `
        -MilestoneDescription 'Planned next major release'
    Write-Ok "Milestones created: v1.0 (#$milestoneV1), v2.0 (#$milestoneV2)"

    Write-Step 'Creating issues'

    # Issue used by most test PR bodies (fixes/closes #N work-item references).
    $workItemIssue = New-GitHubIssue `
        -Title     'Add new feature: title validation' `
        -IssueBody 'This issue exists so that test pull requests can satisfy the work-item reference requirement using closes/fixes keywords.'

    # Issue with a milestone — merge-warden copies it onto the PR when
    # sync_milestone_from_issue = true.
    $propagationIssue = New-GitHubIssue `
        -Title           'Add new feature: issue propagation support' `
        -IssueBody       'This issue carries a milestone and will be added to a Projects v2 board. The test/issue-propagation PR references this issue to verify that the milestone and project are propagated onto the PR.' `
        -MilestoneNumber $milestoneV1

    Write-Ok "Issues created: #$workItemIssue (work-item), #$propagationIssue (propagation)"

    Write-Step 'Creating Projects v2 board'

    $projectId = New-GitHubProject -Title 'merge-warden test board'
    if ($projectId)
    {
        Add-IssueToProject -ProjectId $projectId -IssueNumber $propagationIssue
        Write-Ok "Project created and issue #$propagationIssue added"
    }

    # ------------------------------------------------------------------
    # Helper: create a branch, add unique content, commit, push, and
    # open a draft pull request.  Returns to the default branch afterwards.
    # ------------------------------------------------------------------
    function New-TestBranch
    {
        param(
            [string] $BranchName,
            [string] $CommitMessage,
            [string] $FileName,
            [string] $FileContent,
            [int]    $ExtraLines = 0,
            [string] $PrTitle,
            [string] $PrBody = 'closes #1'
        )

        Write-Info "Branch: $BranchName"

        Invoke-Git @('checkout', $defaultBranch)
        Invoke-Git @('checkout', '-b', $BranchName)

        $filePath = Join-Path $cloneDir $FileName
        $dir = Split-Path $filePath -Parent
        if (-not (Test-Path $dir))
        {
            $null = New-Item -ItemType Directory -Path $dir -Force
        }

        Set-Content -Path $filePath -Value $FileContent -Encoding UTF8

        if ($ExtraLines -gt 0)
        {
            # Append padding lines so merge-warden size buckets are exercised.
            1..$ExtraLines | ForEach-Object {
                Add-Content -Path $filePath -Value "padding-line-$($_): $(New-Guid)" -Encoding UTF8
            }
        }

        Invoke-Git @('add', $FileName)
        Invoke-Git @('commit', '-m', $CommitMessage)
        Invoke-Git @('push', 'origin', $BranchName)

        # Open a draft pull request so merge-warden processes the event immediately.
        gh pr create `
            --draft `
            --title $PrTitle `
            --body  $PrBody `
            --base  $defaultBranch `
            --head  $BranchName `
            --repo  $repoFullName
        if ($LASTEXITCODE -ne 0)
        {
            Write-Warning "Failed to create draft PR for '$BranchName' — continuing"
        }

        Invoke-Git @('checkout', $defaultBranch)
    }

    # ======================================================================
    # TITLE VALIDATION scenarios
    # ======================================================================
    Write-Step 'Creating title-validation branches'

    New-TestBranch `
        -BranchName    'test/title-valid-feat' `
        -CommitMessage 'feat: add valid feature title test file' `
        -FileName      'test/title/valid-feat.txt' `
        -FileContent   @"
This branch is for testing a PR with a valid conventional commit title.
PR title to use: feat(test): add new feature for integration testing
PR body to use:  closes #1
"@ `
        -PrTitle 'feat(test): add new feature for integration testing' `
        -PrBody  'closes #1'

    New-TestBranch `
        -BranchName    'test/title-valid-fix' `
        -CommitMessage 'fix(scope): add valid fix title with scope test file' `
        -FileName      'test/title/valid-fix.txt' `
        -FileContent   @"
This branch is for testing a PR with a valid fix title that includes a scope.
PR title to use: fix(auth): correct token expiry calculation
PR body to use:  fixes #1
"@ `
        -PrTitle 'fix(auth): correct token expiry calculation' `
        -PrBody  'fixes #1'

    New-TestBranch `
        -BranchName    'test/title-invalid-no-type' `
        -CommitMessage 'chore: add invalid-no-type title test file' `
        -FileName      'test/title/invalid-no-type.txt' `
        -FileContent   @"
This branch is for testing a PR whose title has NO conventional commit type.
PR title to use: add a thing without a type prefix
PR body to use:  closes #1
Expected result: pr-issue: invalid-title-format label applied, check fails.
"@ `
        -PrTitle 'add a thing without a type prefix' `
        -PrBody  'closes #1'

    New-TestBranch `
        -BranchName    'test/title-invalid-no-desc' `
        -CommitMessage 'chore: add invalid-no-desc title test file' `
        -FileName      'test/title/invalid-no-desc.txt' `
        -FileContent   @"
This branch is for testing a PR whose title has a type but no description.
PR title to use: feat:
PR body to use:  closes #1
Expected result: pr-issue: invalid-title-format label applied, check fails.
"@ `
        -PrTitle 'feat:' `
        -PrBody  'closes #1'

    New-TestBranch `
        -BranchName    'test/title-breaking-change' `
        -CommitMessage 'feat!: add breaking change title test file' `
        -FileName      'test/title/breaking-change.txt' `
        -FileContent   @"
This branch is for testing a PR with a BREAKING CHANGE marker in the title.
PR title to use: feat!: remove deprecated configuration option
PR body to use:  closes #1
Expected result: valid title, no title label.
"@ `
        -PrTitle 'feat!: remove deprecated configuration option' `
        -PrBody  'closes #1'

    # ======================================================================
    # WORK-ITEM VALIDATION scenarios
    # ======================================================================
    Write-Step 'Creating work-item-validation branches'

    New-TestBranch `
        -BranchName    'test/workitem-fixes-hash' `
        -CommitMessage 'fix: add workitem-fixes-hash test file' `
        -FileName      'test/workitem/fixes-hash.txt' `
        -FileContent   @"
This branch is for testing a PR whose body contains a "fixes #N" reference.
PR title to use: fix: correct null pointer in validator
PR body to use:  fixes #1
Expected result: work-item validation passes, no missing-work-item label.
"@ `
        -PrTitle 'fix: correct null pointer in validator' `
        -PrBody  'fixes #1'

    New-TestBranch `
        -BranchName    'test/workitem-closes-url' `
        -CommitMessage 'fix: add workitem-closes-url test file' `
        -FileName      'test/workitem/closes-url.txt' `
        -FileContent   @"
This branch is for testing a PR whose body contains a full GitHub issue URL.
PR title to use: fix: address issue via full URL reference
PR body to use:  closes https://github.com/$Org/$RepoName/issues/1
Expected result: work-item validation passes, no missing-work-item label.
"@ `
        -PrTitle 'fix: address issue via full URL reference' `
        -PrBody  "closes https://github.com/$Org/$RepoName/issues/1"

    New-TestBranch `
        -BranchName    'test/workitem-missing' `
        -CommitMessage 'fix: add workitem-missing test file' `
        -FileName      'test/workitem/missing.txt' `
        -FileContent   @"
This branch is for testing a PR whose body contains NO work-item reference.
PR title to use: fix: a fix with no issue reference
PR body to use:  This PR fixes a bug but mentions no issue.
Expected result: pr-issue: missing-work-item label applied, check fails.
"@ `
        -PrTitle 'fix: a fix with no issue reference' `
        -PrBody  'This PR fixes a bug but mentions no issue.'

    # ======================================================================
    # PR SIZE scenarios
    # ======================================================================
    Write-Step 'Creating PR-size branches'

    # XS: 2 lines (1–10)
    New-TestBranch `
        -BranchName    'test/size-xs' `
        -CommitMessage 'chore: add size-xs test file' `
        -FileName      'test/size/xs.txt' `
        -FileContent   "line-1: xs size test`nline-2: xs size test" `
        -ExtraLines    0 `
        -PrTitle       'chore: size XS test (1-10 lines)' `
        -PrBody        'closes #1'

    # S: 20 lines (11–50)
    New-TestBranch `
        -BranchName    'test/size-s' `
        -CommitMessage 'chore: add size-s test file' `
        -FileName      'test/size/s.txt' `
        -FileContent   'size-s test file' `
        -ExtraLines    19 `
        -PrTitle       'chore: size S test (11-50 lines)' `
        -PrBody        'closes #1'

    # M: 75 lines (51–100)
    New-TestBranch `
        -BranchName    'test/size-m' `
        -CommitMessage 'chore: add size-m test file' `
        -FileName      'test/size/m.txt' `
        -FileContent   'size-m test file' `
        -ExtraLines    74 `
        -PrTitle       'chore: size M test (51-100 lines)' `
        -PrBody        'closes #1'

    # L: 150 lines (101–250)
    New-TestBranch `
        -BranchName    'test/size-l' `
        -CommitMessage 'chore: add size-l test file' `
        -FileName      'test/size/l.txt' `
        -FileContent   'size-l test file' `
        -ExtraLines    149 `
        -PrTitle       'chore: size L test (101-250 lines)' `
        -PrBody        'closes #1'

    # XL: 300 lines (251–500)
    New-TestBranch `
        -BranchName    'test/size-xl' `
        -CommitMessage 'chore: add size-xl test file' `
        -FileName      'test/size/xl.txt' `
        -FileContent   'size-xl test file' `
        -ExtraLines    299 `
        -PrTitle       'chore: size XL test (251-500 lines)' `
        -PrBody        'closes #1'

    # XXL: 600 lines (500+)
    New-TestBranch `
        -BranchName    'test/size-xxl' `
        -CommitMessage 'chore: add size-xxl test file' `
        -FileName      'test/size/xxl.txt' `
        -FileContent   'size-xxl test file' `
        -ExtraLines    599 `
        -PrTitle       'chore: size XXL test (500+ lines)' `
        -PrBody        'closes #1'

    # ======================================================================
    # WIP DETECTION scenarios
    # ======================================================================
    Write-Step 'Creating WIP-detection branches'

    New-TestBranch `
        -BranchName    'test/wip-title-prefix' `
        -CommitMessage 'chore: add wip-title-prefix test file' `
        -FileName      'test/wip/title-prefix.txt' `
        -FileContent   @"
This branch is for testing WIP detection via title prefix.
PR title to use: WIP: work in progress on something
PR body to use:  fixes #1
Expected result: WIP label applied, check set to failure (blocking).
"@ `
        -PrTitle 'WIP: work in progress on something' `
        -PrBody  'fixes #1'

    New-TestBranch `
        -BranchName    'test/wip-title-bracket' `
        -CommitMessage 'chore: add wip-title-bracket test file' `
        -FileName      'test/wip/title-bracket.txt' `
        -FileContent   @"
This branch is for testing WIP detection via [WIP] in title.
PR title to use: [WIP] something not yet ready
PR body to use:  fixes #1
Expected result: WIP label applied, check set to failure (blocking).
"@ `
        -PrTitle '[WIP] something not yet ready' `
        -PrBody  'fixes #1'

    New-TestBranch `
        -BranchName    'test/wip-ready' `
        -CommitMessage 'feat: add wip-ready test file' `
        -FileName      'test/wip/ready.txt' `
        -FileContent   @"
This branch is for testing that a PR with no WIP markers is not blocked.
PR title to use: feat: completed and ready for review
PR body to use:  fixes #1
Expected result: no WIP label, check passes.
"@ `
        -PrTitle 'feat: completed and ready for review' `
        -PrBody  'fixes #1'

    # ======================================================================
    # CHANGE-TYPE LABEL scenarios (one branch per conventional commit type)
    # ======================================================================
    Write-Step 'Creating change-type-label branches'

    $changeTypes = @(
        @{ Type = 'feat'; File = 'feat.txt'; Desc = 'enhancement / feature labels' }
        @{ Type = 'fix'; File = 'fix.txt'; Desc = 'bug / bugfix labels' }
        @{ Type = 'docs'; File = 'docs.txt'; Desc = 'documentation labels' }
        @{ Type = 'chore'; File = 'chore.txt'; Desc = 'maintenance / housekeeping labels' }
        @{ Type = 'refactor'; File = 'refactor.txt'; Desc = 'refactoring / code quality labels' }
        @{ Type = 'ci'; File = 'ci.txt'; Desc = 'CI / continuous integration labels' }
        @{ Type = 'perf'; File = 'perf.txt'; Desc = 'performance / optimisation labels' }
        @{ Type = 'test'; File = 'tests.txt'; Desc = 'test / testing labels' }
        @{ Type = 'build'; File = 'build.txt'; Desc = 'build / dependencies labels' }
        @{ Type = 'style'; File = 'style.txt'; Desc = 'style / formatting labels' }
        @{ Type = 'revert'; File = 'revert.txt'; Desc = 'revert labels' }
    )

    foreach ($ct in $changeTypes)
    {
        New-TestBranch `
            -BranchName    "test/change-type-$($ct.Type)" `
            -CommitMessage "$($ct.Type): add change-type $($ct.Type) test file" `
            -FileName      "test/change-type/$($ct.File)" `
            -FileContent   @"
This branch tests change-type label detection for the '$($ct.Type)' commit type.
PR title to use: $($ct.Type): test change type label detection
PR body to use:  fixes #1
Expected result: $($ct.Desc) applied via smart label detection.
"@ `
            -PrTitle "$($ct.Type): test change type label detection" `
            -PrBody  'fixes #1'
    }

    # ======================================================================
    # PR STATE / LIFECYCLE LABEL scenarios
    # ======================================================================
    Write-Step 'Creating PR-state branches'

    New-TestBranch `
        -BranchName    'test/state-draft' `
        -CommitMessage 'feat: add state-draft test file' `
        -FileName      'test/state/draft.txt' `
        -FileContent   @"
This branch is for testing the draft lifecycle label.
Open the PR as a DRAFT.
Expected result: "status: draft" label applied.
"@ `
        -PrTitle 'feat: draft state lifecycle test' `
        -PrBody  'fixes #1'

    New-TestBranch `
        -BranchName    'test/state-ready-for-review' `
        -CommitMessage 'feat: add state-ready-for-review test file' `
        -FileName      'test/state/ready-for-review.txt' `
        -FileContent   @"
This branch is for testing the in-review lifecycle label.
Open the PR as READY FOR REVIEW (not draft).
Expected result: "status: in-review" label applied.
"@ `
        -PrTitle 'feat: ready for review state lifecycle test' `
        -PrBody  'fixes #1'

    # ======================================================================
    # ISSUE PROPAGATION scenario
    # ======================================================================
    Write-Step 'Creating issue-propagation branch'

    New-TestBranch `
        -BranchName    'test/issue-propagation' `
        -CommitMessage 'feat: add issue-propagation test file' `
        -FileName      'test/issue-propagation.txt' `
        -FileContent   @"
This branch tests milestone and project propagation from a linked issue.

The script pre-created issue #$propagationIssue with milestone v1.0 and
added it to the 'merge-warden test board' Projects v2 board.

PR title to use: feat: test issue metadata propagation
PR body to use:  closes #$propagationIssue
Expected result:
  - The PR receives the same milestone as issue #$propagationIssue.
  - The PR is added to the same Projects v2 project as issue #$propagationIssue.
"@ `
        -PrTitle 'feat: test issue metadata propagation' `
        -PrBody  "closes #$propagationIssue"

    # ======================================================================
    # BYPASS RULES scenarios
    # ======================================================================
    Write-Step 'Creating bypass-rules branches'

    New-TestBranch `
        -BranchName    'test/bypass-title-user' `
        -CommitMessage 'chore: add bypass-title-user test file' `
        -FileName      'test/bypass/title-user.txt' `
        -FileContent   @"
This branch tests the title-convention bypass rule.
Open the PR as the GitHub user listed in bypassRules.title_convention.users
(configured as "merge-warden-bypass-user" in .github/merge-warden.toml).
PR title to use: Not a conventional commit title at all
PR body to use:  fixes #1
Expected result: title validation is skipped for the bypass user; no
                 invalid-title-format label applied.
"@ `
        -PrTitle 'Not a conventional commit title at all' `
        -PrBody  'fixes #1'

    New-TestBranch `
        -BranchName    'test/bypass-workitem-user' `
        -CommitMessage 'chore: add bypass-workitem-user test file' `
        -FileName      'test/bypass/workitem-user.txt' `
        -FileContent   @"
This branch tests the work-item bypass rule.
Open the PR as the GitHub user listed in bypassRules.work_items.users
(configured as "merge-warden-bypass-user" in .github/merge-warden.toml).
PR title to use: chore: bypass work-item check test
PR body to use:  This PR has no issue reference but the author is bypassed.
Expected result: work-item validation is skipped; no missing-work-item label.
"@ `
        -PrTitle 'chore: bypass work-item check test' `
        -PrBody  'This PR has no issue reference but the author is bypassed.'

    # ======================================================================
    # Summary
    # ======================================================================
    Write-Step 'Done'
    Write-Host ''
    Write-Host "Repository  : https://github.com/$repoFullName" -ForegroundColor White
    Write-Host "Clone dir   : $cloneDir" -ForegroundColor White
    Write-Host ''
    Write-Host 'Branches and draft PRs created:' -ForegroundColor White

    $branches = git branch -r --list 'origin/test/*' 2>$null
    $branches | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }

    Write-Host ''
    Write-Host 'Next steps:' -ForegroundColor Yellow
    Write-Host '  1. Install your GitHub App on the new repository.' -ForegroundColor Yellow
    Write-Host '  2. Start merge-warden locally:' -ForegroundColor Yellow
    Write-Host "       .\samples\run-local.ps1 -SmeeUrl <url> -AppConfigFile .\samples\app-config.sample.toml" -ForegroundColor Yellow
    Write-Host '  3. Update or re-open the draft PRs to trigger merge-warden webhook events.' -ForegroundColor Yellow
    Write-Host '     For state-ready-for-review, mark the PR as ready for review manually.' -ForegroundColor Yellow
    Write-Host "  4. The issue-propagation PR (test/issue-propagation) already references" -ForegroundColor Yellow
    Write-Host "     issue #$propagationIssue which has milestone v1.0 and is in the project board." -ForegroundColor Yellow

}
finally
{
    Pop-Location

    if ($CleanupOnExit)
    {
        Write-Step 'Cleaning up local clone'
        Remove-Item $cloneDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Ok "Removed $cloneDir"
    }
    else
    {
        Write-Info "Local clone kept at: $cloneDir"
        Write-Info 'Pass -CleanupOnExit to remove it automatically.'
    }
}
