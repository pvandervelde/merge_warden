# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-04-25

### <!-- 0 -->⛰️  Features

- Propagate issue metadata for all reference keywords

### <!-- 1 -->🐛 Bug Fixes

- Resolve five runtime bugs in validation, labels, and issue propagation ([#223](https://github.com/pvandervelde/merge_warden/issues/223))
- Add missing docs to serde defaults and fix pattern_matches escaping
- Resolve test failures, unused import, and security advisories
- Correct serde defaults, TOML config, and GitHub App permissions for issue propagation
- Migrate to github-bot-sdk sub-client API
- Address five bugs found during testing
- Restructure release workflow and rename container image ([#218](https://github.com/pvandervelde/merge_warden/issues/218))
- Replace ls with find in artifact count check
- Address release workflow security and reliability issues
- Restructure publish workflow and rename container image

### <!-- 6 -->🧪 Testing

- Add tests covering the five bug fixes

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Ignore the logs folder
- Upgrade to distroless/cc-debian13 and update samples



## [0.3.0] - 2026-04-12

### <!-- 0 -->⛰️  Features

- Add structured PR title diagnostics with actionable error messages ([#211](https://github.com/pvandervelde/merge_warden/issues/211))
- Thread TitleValidationResult through title validation
- Add PR title diagnostic types and unit tests
- Propagate milestone and project from linked issues to PRs ([#204](https://github.com/pvandervelde/merge_warden/issues/204))
- Implement project propagation via SDK GraphQL
- Call propagate_issue_metadata inside process_pull_request
- Wire issue provider into MergeWarden and add integration tests
- Implement propagate_issue_metadata with milestone sync
- Add IssuePropagationConfig, PullRequest.milestone_number and propagation tests
- Implement IssueMetadataProvider for GitHubProvider with tests
- Add IssueMetadata models and IssueMetadataProvider trait with tests
- Implement extract_closing_issue_reference parser
- Add IssueReference type and tests for issue reference parser
- Add interface design for issue metadata propagation
- Add state-based PR lifecycle label management ([#203](https://github.com/pvandervelde/merge_warden/issues/203))
- Implement PR state lifecycle label management (auto via agent)
- Add types, docs, and tests for PR state lifecycle labels (auto via agent)
- Add WIP detection and blocking for pull requests ([#201](https://github.com/pvandervelde/merge_warden/issues/201))
- Implement WIP detection and blocking
- Implement queue-based webhook processing ([#200](https://github.com/pvandervelde/merge_warden/issues/200))
- Implement queue-based webhook processing (task 3.0)
- Replace azure-functions crate with containerised server binary ([#199](https://github.com/pvandervelde/merge_warden/issues/199))
- Implement containerised webhook server binary
- Migrate GitHub API client from octocrab to github-bot-sdk ([#196](https://github.com/pvandervelde/merge_warden/issues/196))
- Migrate webhook handling to SDK WebhookReceiver
- Replace manual JWT signing with AppAuthProvider
- Migrate GitHubProvider and entry points to github-bot-sdk
- Add AppAuthProvider for GitHub App authentication
- Replace octocrab with github-bot-sdk dependency and add WireMock tests
- Add design specs and interface stubs for SDK migration, containerisation, and queue processing ([#195](https://github.com/pvandervelde/merge_warden/issues/195))
- Add missing webhook handler and route stubs
- Add interface design stubs for SDK migration, containerisation, and queue ingress
- Add embedded webhook server for self-contained testing
- Refactor integration tests into focused test modules
- Complete integration testing infrastructure ([#188](https://github.com/pvandervelde/merge_warden/issues/188))
- Complete integration testing infrastructure implementation
- Introducing the integration testing crate
- Remove Application Insights connection from Azure Function and replace with console logging ([#186](https://github.com/pvandervelde/merge_warden/issues/186))
- Exclude Azure config files from deployment package ([#183](https://github.com/pvandervelde/merge_warden/issues/183))
- Exclude Azure config files from deployment package
- Refactor specs folder into comprehensive living document system ([#174](https://github.com/pvandervelde/merge_warden/issues/174))
- Complete content migration and cleanup of old spec files
- Refactor specs folder into comprehensive living document
- Enforce documentation standards with clippy rules
- Refactor PullRequestProvider trait method ordering and naming ([#171](https://github.com/pvandervelde/merge_warden/issues/171))
- Refactor PullRequestProvider trait method ordering and naming

### <!-- 1 -->🐛 Bug Fixes

- Repair Docker image build and close gaps in release pipeline ([#215](https://github.com/pvandervelde/merge_warden/issues/215))
- Resolve static link deps, shell bug, and license check
- Commit Cargo.lock and persist security-patched dependency versions
- Resolve advisory failures and address PR review comments
- Install pkg-config and libssl-dev in builder stage
- Update Docker action SHAs to valid pinned commits ([#213](https://github.com/pvandervelde/merge_warden/issues/213))
- Update Docker action SHAs to valid pinned commits
- Address PR review comments — dead code, stale comment, Display, spelling
- Report MissingColon even when UppercaseType or UnrecognizedType are also present
- Reopen PR if auto-closed by force-push during branch update ([#209](https://github.com/pvandervelde/merge_warden/issues/209))
- Address PR review feedback on auto-close fix
- Reopen PR if auto-closed by force-push during branch update
- Update release branch to master HEAD on each workflow run/crea ([#208](https://github.com/pvandervelde/merge_warden/issues/208))
- Address PR review feedback on prepare-release script
- Update release branch to master HEAD on each workflow run
- Update Dockerfile base image and add version assertion to publish-release workflow ([#207](https://github.com/pvandervelde/merge_warden/issues/207))
- Harden version assertion step against script injection
- Update Dockerfile to rust:1.94-slim and add version assertion to publish-release workflow
- Remove dead PerformanceResult test infrastructure
- Address PR review comments
- Repair stale test repository cleanup workflow ([#205](https://github.com/pvandervelde/merge_warden/issues/205))
- Replace gh api --arg with awk -v for prefix filtering
- Pass REPO_PREFIX as jq --arg to prevent injection
- Retry content API calls after fresh repository creation
- Resolve actionlint errors across workflow files
- SHA-pin actionlint action and fix repo-count display
- Resolve stale-repo cleanup prefix mismatch and org/user API endpoint
- Address PR review comments
- Skip reviews with null user id; update spec
- Address PR review comments for state-lifecycle labels
- Resolve clippy lints in manage_pr_state_labels (auto via agent)
- Address second round of PR review feedback
- Address PR review feedback on WIP implementation
- Resolve clippy warnings in test helpers
- Add missing wip_check field to struct initializers
- Address third round of PR review feedback
- Address PR review feedback
- Queue mode is pure queue consumer, no webhook endpoint
- Address second round of PR review feedback
- Address PR review feedback
- Fix pre-existing clippy warnings
- Address PR review findings
- Remove todo! panics in GitHub mock module
- Return error on missing default_branch; percent-encode content URLs
- Suppress RUSTSEC-2023-0071; remove stale RUSTSEC-2025-0134 entry
- Install rustls crypto provider before test environment setup
- Allow pvandervelde git org source and new license exceptions
- Pin github-bot-sdk to commit SHA; suppress RUSTSEC-2023-0071 in audit
- Prevent thundering herd in installation token cache
- Use constant-time HMAC comparison in webhook signature verification
- Update tests for AppState migration from octocrab
- Fix pagination and deletion issues in test repo cleanup workflow
- Re-trigger webhook each poll iteration for config test
- Poll for success conclusion instead of check-run ID change
- Fix config-update timeout and find() scan bug
- Fix webhook payloads and remove unachievable assertions
- Add missing semicolons after ?-operator
- Fix check name mismatch and add_file upsert
- Fix configure_for_repository panic and checks permission
- Treat empty LOCAL_WEBHOOK_ENDPOINT as unset
- Upsert config file and add webhook endpoint secret
- Fix remaining warnings and implement webhook stubs
- Resolve 33 compiler warnings in test infrastructure
- Mark credential tests as ignored for coverage runner
- Restructure integration-tests workflow for correct credential handling
- Resolve unit, doc, and integration test failures
- Resolve security audit failures and add integration/e2e workflows
- Resolve all compilation errors in integration tests
- Force serial test execution to prevent environment variable race conditions
- Add environment variable cleanup to prevent test interference
- Add environment cleanup to missing github token test
- Add environment cleanup to timeout format test
- Resolve CI test failures with environment isolation
- Resolve unit test failures in integration test infrastructure
- Improve unit test environment isolation
- Resolve doctest compilation errors
- Update CI workflow to remove references to deleted config files
- Add missing function documentation and resolve doc test error
- Correct error message test expectation in developer_platforms crate
- Add comprehensive documentation and resolve clippy issues in CLI crate
- Add comprehensive documentation to Azure Functions crate
- Resolve all remaining clippy documentation and lint errors
- Resolve all missing documentation compilation errors
- Resolve remaining missing documentation compilation errors

### <!-- 2 -->🚜 Refactor

- Move private helpers before diagnose_pr_title in alphabetical order
- Rename test helper app credentials to repo-creation-app

### <!-- 3 -->📚 Documentation

- Document TitleIssue, TitleDiagnosis, TitleValidationResult types
- Update sync_projects docstring to reflect GraphQL implementation
- Add issuePropagation section to sample config
- Document containerised server deployment
- Mark github-bot-sdk migration spec as complete
- Add architecture design specs for SDK migration, containerisation, and queue processing
- Reorganise-docs-folder ([#194](https://github.com/pvandervelde/merge_warden/issues/194))
- Adding the AGENTS.md file
- Adding the catalog and constraint documents
- Adding the ADR folder
- Adding the standards folder
- Moving the spec files
- Update testing specs
- Update testing spec README
- Complete GitHubProvider functions documentation and organization
- Add initial GitHub provider documentation
- Complete comprehensive model struct documentation
- Add comprehensive error variant documentation

### <!-- 5 -->🎨 Styling

- Apply rustfmt formatting to WIP config and labels code
- Apply formatting fixes to integration test files

### <!-- 6 -->🧪 Testing

- Verify diagnosis-aware comment content per TitleIssue variant
- Add actionlint job to validate workflow YAML and embedded shell scripts
- Add IssuePropagationConfig TOML and milestone e2e tests
- Update deps and crate_structure_tests for SDK migration
- Add comprehensive integration test infrastructure ([#193](https://github.com/pvandervelde/merge_warden/issues/193))
- Remove configuration_change_detection test

### <!-- 7 -->⚙️ Miscellaneous Tasks

- 0.3.0 ([#214](https://github.com/pvandervelde/merge_warden/issues/214))
- 0.3.0 ([#210](https://github.com/pvandervelde/merge_warden/issues/210))
- Improve the renovate config for security purposes ([#212](https://github.com/pvandervelde/merge_warden/issues/212))
- Improve the renovate config for security purposes
- Remove azure-functions crate and orphaned dead code ([#206](https://github.com/pvandervelde/merge_warden/issues/206))
- Remove azure-functions crate and dead code
- Suppress unmaintained advisories for instant and paste
- Fix Rust formatting
- Fix rust formatting
- Add Claude PR review workflow ([#198](https://github.com/pvandervelde/merge_warden/issues/198))
- Add the Claude PR review workflow
- Add scheduled workflow to clean up stale test repositories ([#197](https://github.com/pvandervelde/merge_warden/issues/197))
- Add scheduled cleanup workflow for stale test repositories
- Fix rust formatting
- Fix Rust formatting
- Fixing Rust formatting
- Comprehensive documentation improvements ([#173](https://github.com/pvandervelde/merge_warden/issues/173))



## [0.3.0] - 2026-04-11

### <!-- 0 -->⛰️  Features

- Add structured PR title diagnostics with actionable error messages ([#211](https://github.com/pvandervelde/merge_warden/issues/211))
- Thread TitleValidationResult through title validation
- Add PR title diagnostic types and unit tests
- Propagate milestone and project from linked issues to PRs ([#204](https://github.com/pvandervelde/merge_warden/issues/204))
- Implement project propagation via SDK GraphQL
- Call propagate_issue_metadata inside process_pull_request
- Wire issue provider into MergeWarden and add integration tests
- Implement propagate_issue_metadata with milestone sync
- Add IssuePropagationConfig, PullRequest.milestone_number and propagation tests
- Implement IssueMetadataProvider for GitHubProvider with tests
- Add IssueMetadata models and IssueMetadataProvider trait with tests
- Implement extract_closing_issue_reference parser
- Add IssueReference type and tests for issue reference parser
- Add interface design for issue metadata propagation
- Add state-based PR lifecycle label management ([#203](https://github.com/pvandervelde/merge_warden/issues/203))
- Implement PR state lifecycle label management (auto via agent)
- Add types, docs, and tests for PR state lifecycle labels (auto via agent)
- Add WIP detection and blocking for pull requests ([#201](https://github.com/pvandervelde/merge_warden/issues/201))
- Implement WIP detection and blocking
- Implement queue-based webhook processing ([#200](https://github.com/pvandervelde/merge_warden/issues/200))
- Implement queue-based webhook processing (task 3.0)
- Replace azure-functions crate with containerised server binary ([#199](https://github.com/pvandervelde/merge_warden/issues/199))
- Implement containerised webhook server binary
- Migrate GitHub API client from octocrab to github-bot-sdk ([#196](https://github.com/pvandervelde/merge_warden/issues/196))
- Migrate webhook handling to SDK WebhookReceiver
- Replace manual JWT signing with AppAuthProvider
- Migrate GitHubProvider and entry points to github-bot-sdk
- Add AppAuthProvider for GitHub App authentication
- Replace octocrab with github-bot-sdk dependency and add WireMock tests
- Add design specs and interface stubs for SDK migration, containerisation, and queue processing ([#195](https://github.com/pvandervelde/merge_warden/issues/195))
- Add missing webhook handler and route stubs
- Add interface design stubs for SDK migration, containerisation, and queue ingress
- Add embedded webhook server for self-contained testing
- Refactor integration tests into focused test modules
- Complete integration testing infrastructure ([#188](https://github.com/pvandervelde/merge_warden/issues/188))
- Complete integration testing infrastructure implementation
- Introducing the integration testing crate
- Remove Application Insights connection from Azure Function and replace with console logging ([#186](https://github.com/pvandervelde/merge_warden/issues/186))
- Exclude Azure config files from deployment package ([#183](https://github.com/pvandervelde/merge_warden/issues/183))
- Exclude Azure config files from deployment package
- Refactor specs folder into comprehensive living document system ([#174](https://github.com/pvandervelde/merge_warden/issues/174))
- Complete content migration and cleanup of old spec files
- Refactor specs folder into comprehensive living document
- Enforce documentation standards with clippy rules
- Refactor PullRequestProvider trait method ordering and naming ([#171](https://github.com/pvandervelde/merge_warden/issues/171))
- Refactor PullRequestProvider trait method ordering and naming

### <!-- 1 -->🐛 Bug Fixes

- Update Docker action SHAs to valid pinned commits ([#213](https://github.com/pvandervelde/merge_warden/issues/213))
- Update Docker action SHAs to valid pinned commits
- Address PR review comments — dead code, stale comment, Display, spelling
- Report MissingColon even when UppercaseType or UnrecognizedType are also present
- Reopen PR if auto-closed by force-push during branch update ([#209](https://github.com/pvandervelde/merge_warden/issues/209))
- Address PR review feedback on auto-close fix
- Reopen PR if auto-closed by force-push during branch update
- Update release branch to master HEAD on each workflow run/crea ([#208](https://github.com/pvandervelde/merge_warden/issues/208))
- Address PR review feedback on prepare-release script
- Update release branch to master HEAD on each workflow run
- Update Dockerfile base image and add version assertion to publish-release workflow ([#207](https://github.com/pvandervelde/merge_warden/issues/207))
- Harden version assertion step against script injection
- Update Dockerfile to rust:1.94-slim and add version assertion to publish-release workflow
- Remove dead PerformanceResult test infrastructure
- Address PR review comments
- Repair stale test repository cleanup workflow ([#205](https://github.com/pvandervelde/merge_warden/issues/205))
- Replace gh api --arg with awk -v for prefix filtering
- Pass REPO_PREFIX as jq --arg to prevent injection
- Retry content API calls after fresh repository creation
- Resolve actionlint errors across workflow files
- SHA-pin actionlint action and fix repo-count display
- Resolve stale-repo cleanup prefix mismatch and org/user API endpoint
- Address PR review comments
- Skip reviews with null user id; update spec
- Address PR review comments for state-lifecycle labels
- Resolve clippy lints in manage_pr_state_labels (auto via agent)
- Address second round of PR review feedback
- Address PR review feedback on WIP implementation
- Resolve clippy warnings in test helpers
- Add missing wip_check field to struct initializers
- Address third round of PR review feedback
- Address PR review feedback
- Queue mode is pure queue consumer, no webhook endpoint
- Address second round of PR review feedback
- Address PR review feedback
- Fix pre-existing clippy warnings
- Address PR review findings
- Remove todo! panics in GitHub mock module
- Return error on missing default_branch; percent-encode content URLs
- Suppress RUSTSEC-2023-0071; remove stale RUSTSEC-2025-0134 entry
- Install rustls crypto provider before test environment setup
- Allow pvandervelde git org source and new license exceptions
- Pin github-bot-sdk to commit SHA; suppress RUSTSEC-2023-0071 in audit
- Prevent thundering herd in installation token cache
- Use constant-time HMAC comparison in webhook signature verification
- Update tests for AppState migration from octocrab
- Fix pagination and deletion issues in test repo cleanup workflow
- Re-trigger webhook each poll iteration for config test
- Poll for success conclusion instead of check-run ID change
- Fix config-update timeout and find() scan bug
- Fix webhook payloads and remove unachievable assertions
- Add missing semicolons after ?-operator
- Fix check name mismatch and add_file upsert
- Fix configure_for_repository panic and checks permission
- Treat empty LOCAL_WEBHOOK_ENDPOINT as unset
- Upsert config file and add webhook endpoint secret
- Fix remaining warnings and implement webhook stubs
- Resolve 33 compiler warnings in test infrastructure
- Mark credential tests as ignored for coverage runner
- Restructure integration-tests workflow for correct credential handling
- Resolve unit, doc, and integration test failures
- Resolve security audit failures and add integration/e2e workflows
- Resolve all compilation errors in integration tests
- Force serial test execution to prevent environment variable race conditions
- Add environment variable cleanup to prevent test interference
- Add environment cleanup to missing github token test
- Add environment cleanup to timeout format test
- Resolve CI test failures with environment isolation
- Resolve unit test failures in integration test infrastructure
- Improve unit test environment isolation
- Resolve doctest compilation errors
- Update CI workflow to remove references to deleted config files
- Add missing function documentation and resolve doc test error
- Correct error message test expectation in developer_platforms crate
- Add comprehensive documentation and resolve clippy issues in CLI crate
- Add comprehensive documentation to Azure Functions crate
- Resolve all remaining clippy documentation and lint errors
- Resolve all missing documentation compilation errors
- Resolve remaining missing documentation compilation errors

### <!-- 2 -->🚜 Refactor

- Move private helpers before diagnose_pr_title in alphabetical order
- Rename test helper app credentials to repo-creation-app

### <!-- 3 -->📚 Documentation

- Document TitleIssue, TitleDiagnosis, TitleValidationResult types
- Update sync_projects docstring to reflect GraphQL implementation
- Add issuePropagation section to sample config
- Document containerised server deployment
- Mark github-bot-sdk migration spec as complete
- Add architecture design specs for SDK migration, containerisation, and queue processing
- Reorganise-docs-folder ([#194](https://github.com/pvandervelde/merge_warden/issues/194))
- Adding the AGENTS.md file
- Adding the catalog and constraint documents
- Adding the ADR folder
- Adding the standards folder
- Moving the spec files
- Update testing specs
- Update testing spec README
- Complete GitHubProvider functions documentation and organization
- Add initial GitHub provider documentation
- Complete comprehensive model struct documentation
- Add comprehensive error variant documentation

### <!-- 5 -->🎨 Styling

- Apply rustfmt formatting to WIP config and labels code
- Apply formatting fixes to integration test files

### <!-- 6 -->🧪 Testing

- Verify diagnosis-aware comment content per TitleIssue variant
- Add actionlint job to validate workflow YAML and embedded shell scripts
- Add IssuePropagationConfig TOML and milestone e2e tests
- Update deps and crate_structure_tests for SDK migration
- Add comprehensive integration test infrastructure ([#193](https://github.com/pvandervelde/merge_warden/issues/193))
- Remove configuration_change_detection test

### <!-- 7 -->⚙️ Miscellaneous Tasks

- 0.3.0 ([#210](https://github.com/pvandervelde/merge_warden/issues/210))
- Improve the renovate config for security purposes ([#212](https://github.com/pvandervelde/merge_warden/issues/212))
- Improve the renovate config for security purposes
- Remove azure-functions crate and orphaned dead code ([#206](https://github.com/pvandervelde/merge_warden/issues/206))
- Remove azure-functions crate and dead code
- Suppress unmaintained advisories for instant and paste
- Fix Rust formatting
- Fix rust formatting
- Add Claude PR review workflow ([#198](https://github.com/pvandervelde/merge_warden/issues/198))
- Add the Claude PR review workflow
- Add scheduled workflow to clean up stale test repositories ([#197](https://github.com/pvandervelde/merge_warden/issues/197))
- Add scheduled cleanup workflow for stale test repositories
- Fix rust formatting
- Fix Rust formatting
- Fixing Rust formatting
- Comprehensive documentation improvements ([#173](https://github.com/pvandervelde/merge_warden/issues/173))



## [0.3.0] - 2026-04-10

### <!-- 0 -->⛰️  Features

- Add structured PR title diagnostics with actionable error messages ([#211](https://github.com/pvandervelde/merge_warden/issues/211))
- Thread TitleValidationResult through title validation
- Add PR title diagnostic types and unit tests
- Propagate milestone and project from linked issues to PRs ([#204](https://github.com/pvandervelde/merge_warden/issues/204))
- Implement project propagation via SDK GraphQL
- Call propagate_issue_metadata inside process_pull_request
- Wire issue provider into MergeWarden and add integration tests
- Implement propagate_issue_metadata with milestone sync
- Add IssuePropagationConfig, PullRequest.milestone_number and propagation tests
- Implement IssueMetadataProvider for GitHubProvider with tests
- Add IssueMetadata models and IssueMetadataProvider trait with tests
- Implement extract_closing_issue_reference parser
- Add IssueReference type and tests for issue reference parser
- Add interface design for issue metadata propagation
- Add state-based PR lifecycle label management ([#203](https://github.com/pvandervelde/merge_warden/issues/203))
- Implement PR state lifecycle label management (auto via agent)
- Add types, docs, and tests for PR state lifecycle labels (auto via agent)
- Add WIP detection and blocking for pull requests ([#201](https://github.com/pvandervelde/merge_warden/issues/201))
- Implement WIP detection and blocking
- Implement queue-based webhook processing ([#200](https://github.com/pvandervelde/merge_warden/issues/200))
- Implement queue-based webhook processing (task 3.0)
- Replace azure-functions crate with containerised server binary ([#199](https://github.com/pvandervelde/merge_warden/issues/199))
- Implement containerised webhook server binary
- Migrate GitHub API client from octocrab to github-bot-sdk ([#196](https://github.com/pvandervelde/merge_warden/issues/196))
- Migrate webhook handling to SDK WebhookReceiver
- Replace manual JWT signing with AppAuthProvider
- Migrate GitHubProvider and entry points to github-bot-sdk
- Add AppAuthProvider for GitHub App authentication
- Replace octocrab with github-bot-sdk dependency and add WireMock tests
- Add design specs and interface stubs for SDK migration, containerisation, and queue processing ([#195](https://github.com/pvandervelde/merge_warden/issues/195))
- Add missing webhook handler and route stubs
- Add interface design stubs for SDK migration, containerisation, and queue ingress
- Add embedded webhook server for self-contained testing
- Refactor integration tests into focused test modules
- Complete integration testing infrastructure ([#188](https://github.com/pvandervelde/merge_warden/issues/188))
- Complete integration testing infrastructure implementation
- Introducing the integration testing crate
- Remove Application Insights connection from Azure Function and replace with console logging ([#186](https://github.com/pvandervelde/merge_warden/issues/186))
- Exclude Azure config files from deployment package ([#183](https://github.com/pvandervelde/merge_warden/issues/183))
- Exclude Azure config files from deployment package
- Refactor specs folder into comprehensive living document system ([#174](https://github.com/pvandervelde/merge_warden/issues/174))
- Complete content migration and cleanup of old spec files
- Refactor specs folder into comprehensive living document
- Enforce documentation standards with clippy rules
- Refactor PullRequestProvider trait method ordering and naming ([#171](https://github.com/pvandervelde/merge_warden/issues/171))
- Refactor PullRequestProvider trait method ordering and naming

### <!-- 1 -->🐛 Bug Fixes

- Address PR review comments — dead code, stale comment, Display, spelling
- Report MissingColon even when UppercaseType or UnrecognizedType are also present
- Reopen PR if auto-closed by force-push during branch update ([#209](https://github.com/pvandervelde/merge_warden/issues/209))
- Address PR review feedback on auto-close fix
- Reopen PR if auto-closed by force-push during branch update
- Update release branch to master HEAD on each workflow run/crea ([#208](https://github.com/pvandervelde/merge_warden/issues/208))
- Address PR review feedback on prepare-release script
- Update release branch to master HEAD on each workflow run
- Update Dockerfile base image and add version assertion to publish-release workflow ([#207](https://github.com/pvandervelde/merge_warden/issues/207))
- Harden version assertion step against script injection
- Update Dockerfile to rust:1.94-slim and add version assertion to publish-release workflow
- Remove dead PerformanceResult test infrastructure
- Address PR review comments
- Repair stale test repository cleanup workflow ([#205](https://github.com/pvandervelde/merge_warden/issues/205))
- Replace gh api --arg with awk -v for prefix filtering
- Pass REPO_PREFIX as jq --arg to prevent injection
- Retry content API calls after fresh repository creation
- Resolve actionlint errors across workflow files
- SHA-pin actionlint action and fix repo-count display
- Resolve stale-repo cleanup prefix mismatch and org/user API endpoint
- Address PR review comments
- Skip reviews with null user id; update spec
- Address PR review comments for state-lifecycle labels
- Resolve clippy lints in manage_pr_state_labels (auto via agent)
- Address second round of PR review feedback
- Address PR review feedback on WIP implementation
- Resolve clippy warnings in test helpers
- Add missing wip_check field to struct initializers
- Address third round of PR review feedback
- Address PR review feedback
- Queue mode is pure queue consumer, no webhook endpoint
- Address second round of PR review feedback
- Address PR review feedback
- Fix pre-existing clippy warnings
- Address PR review findings
- Remove todo! panics in GitHub mock module
- Return error on missing default_branch; percent-encode content URLs
- Suppress RUSTSEC-2023-0071; remove stale RUSTSEC-2025-0134 entry
- Install rustls crypto provider before test environment setup
- Allow pvandervelde git org source and new license exceptions
- Pin github-bot-sdk to commit SHA; suppress RUSTSEC-2023-0071 in audit
- Prevent thundering herd in installation token cache
- Use constant-time HMAC comparison in webhook signature verification
- Update tests for AppState migration from octocrab
- Fix pagination and deletion issues in test repo cleanup workflow
- Re-trigger webhook each poll iteration for config test
- Poll for success conclusion instead of check-run ID change
- Fix config-update timeout and find() scan bug
- Fix webhook payloads and remove unachievable assertions
- Add missing semicolons after ?-operator
- Fix check name mismatch and add_file upsert
- Fix configure_for_repository panic and checks permission
- Treat empty LOCAL_WEBHOOK_ENDPOINT as unset
- Upsert config file and add webhook endpoint secret
- Fix remaining warnings and implement webhook stubs
- Resolve 33 compiler warnings in test infrastructure
- Mark credential tests as ignored for coverage runner
- Restructure integration-tests workflow for correct credential handling
- Resolve unit, doc, and integration test failures
- Resolve security audit failures and add integration/e2e workflows
- Resolve all compilation errors in integration tests
- Force serial test execution to prevent environment variable race conditions
- Add environment variable cleanup to prevent test interference
- Add environment cleanup to missing github token test
- Add environment cleanup to timeout format test
- Resolve CI test failures with environment isolation
- Resolve unit test failures in integration test infrastructure
- Improve unit test environment isolation
- Resolve doctest compilation errors
- Update CI workflow to remove references to deleted config files
- Add missing function documentation and resolve doc test error
- Correct error message test expectation in developer_platforms crate
- Add comprehensive documentation and resolve clippy issues in CLI crate
- Add comprehensive documentation to Azure Functions crate
- Resolve all remaining clippy documentation and lint errors
- Resolve all missing documentation compilation errors
- Resolve remaining missing documentation compilation errors

### <!-- 2 -->🚜 Refactor

- Move private helpers before diagnose_pr_title in alphabetical order
- Rename test helper app credentials to repo-creation-app

### <!-- 3 -->📚 Documentation

- Document TitleIssue, TitleDiagnosis, TitleValidationResult types
- Update sync_projects docstring to reflect GraphQL implementation
- Add issuePropagation section to sample config
- Document containerised server deployment
- Mark github-bot-sdk migration spec as complete
- Add architecture design specs for SDK migration, containerisation, and queue processing
- Reorganise-docs-folder ([#194](https://github.com/pvandervelde/merge_warden/issues/194))
- Adding the AGENTS.md file
- Adding the catalog and constraint documents
- Adding the ADR folder
- Adding the standards folder
- Moving the spec files
- Update testing specs
- Update testing spec README
- Complete GitHubProvider functions documentation and organization
- Add initial GitHub provider documentation
- Complete comprehensive model struct documentation
- Add comprehensive error variant documentation

### <!-- 5 -->🎨 Styling

- Apply rustfmt formatting to WIP config and labels code
- Apply formatting fixes to integration test files

### <!-- 6 -->🧪 Testing

- Verify diagnosis-aware comment content per TitleIssue variant
- Add actionlint job to validate workflow YAML and embedded shell scripts
- Add IssuePropagationConfig TOML and milestone e2e tests
- Update deps and crate_structure_tests for SDK migration
- Add comprehensive integration test infrastructure ([#193](https://github.com/pvandervelde/merge_warden/issues/193))
- Remove configuration_change_detection test

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Improve the renovate config for security purposes ([#212](https://github.com/pvandervelde/merge_warden/issues/212))
- Improve the renovate config for security purposes
- Remove azure-functions crate and orphaned dead code ([#206](https://github.com/pvandervelde/merge_warden/issues/206))
- Remove azure-functions crate and dead code
- Suppress unmaintained advisories for instant and paste
- Fix Rust formatting
- Fix rust formatting
- Add Claude PR review workflow ([#198](https://github.com/pvandervelde/merge_warden/issues/198))
- Add the Claude PR review workflow
- Add scheduled workflow to clean up stale test repositories ([#197](https://github.com/pvandervelde/merge_warden/issues/197))
- Add scheduled cleanup workflow for stale test repositories
- Fix rust formatting
- Fix Rust formatting
- Fixing Rust formatting
- Comprehensive documentation improvements ([#173](https://github.com/pvandervelde/merge_warden/issues/173))



## [0.2.0] - 2025-07-12

### <!-- 0 -->⛰️  Features

- Complete terraform migration to separate repository infrastructure from source code ([#161](https://github.com/pvandervelde/merge_warden/issues/161))
- Remove tf-test job from CI workflow
- Complete Phase 4 - Remove terraform code and update documentation
- Complete phases 1-3 of terraform migration
- Implement smart label detection for conventional commit types ([#157](https://github.com/pvandervelde/merge_warden/issues/157))
- Add Azure App Configuration support for smart label detection
- Implement robust non-blocking smart label detection
- Integrate smart label detection with MergeWarden core processing
- Integrate smart label detection with core processing pipeline
- Implement label detection algorithm with three-tier search strategy
- Extend configuration system for change type label detection
- Add labeling for PR size ([#150](https://github.com/pvandervelde/merge_warden/issues/150))
- Implement PR size check bypasses and fix compilation errors
- Add PR size configuration support to Terraform and Azure Function
- Implement smart label discovery for PR size labeling
- Extend configuration schema for PR size checking
- Implement PR size analysis foundation
- Implement comprehensive bypass capabilities with audit trails ([#146](https://github.com/pvandervelde/merge_warden/issues/146))
- Integrate Azure App Configuration for centralized config management
- Add Azure App Configuration for centralized configuration
- Add bypass indication in check status text
- Add enhanced validation result types for bypass logging
- Add bypass rule management commands
- Implement bypass rules for validation checks
- Implement data models for PR author bypass rules
- Support repository-specific PR rule configuration via .github/merge-warden.toml ([#141](https://github.com/pvandervelde/merge_warden/issues/141))
- Updating the CLI with the new config approach
- Updating the azure function with the new config approach
- Updating the core library to match the new config approach
- Updating the way we read and combine the configurations
- Integrate TOML config loading for merge-warden validation rules
- Add TOML-based config schema, loader, and docs for merge-warden pull request rules
- GitHub checks for merge blocking ([#137](https://github.com/pvandervelde/merge_warden/issues/137))
- Switch to status checks for merge blocking

### <!-- 1 -->🐛 Bug Fixes

- Correct conventional_commits_next_version command arguments ([#164](https://github.com/pvandervelde/merge_warden/issues/164))
- Correct conventional_commits_next_version command arguments
- Enable smart label detection in CLI mode
- Broken refactor ([#155](https://github.com/pvandervelde/merge_warden/issues/155))
- Add synchronize event to webhook processing ([#154](https://github.com/pvandervelde/merge_warden/issues/154))
- Add synchronize event to webhook processing
- Complete clippy warning fixes to achieve zero warnings
- Make PR size label discovery case-insensitive
- Fix description-based size label discovery
- Add PR size configuration to ApplicationDefaults
- Add tempfile dev dependency for bypass tests
- Configuration unit tests
- Azure function unable to start and connect to Key Vault ([#130](https://github.com/pvandervelde/merge_warden/issues/130))
- Remove the reference to the local function config file
- Write debug logs in Azure
- Don't initialize the logs twice

### <!-- 2 -->🚜 Refactor

- Improve naming conventions by removing 'Smart' prefix
- Unify label detection structs and improve naming
- Move size integration tests to separate file
- Report failures in the PR processing in the logs but continue working
- Add more logging to the config read
- If the PR is a draft we want to report as 'skipped'
- Improve the github webhook signature verification

### <!-- 3 -->📚 Documentation

- Complete smart label detection documentation
- Create the spec for PR size labeling
- Updating the config file example in the readme
- Adding the configuration schema rfc
- Minor changes to the readme
- Adding a section about the configuration to the README
- Improving the README.md

### <!-- 5 -->🎨 Styling

- Fix rustfmt formatting issues

### <!-- 6 -->🧪 Testing

- Adding tests for the config changes
- Updating the configuration tests
- Add #[cfg(test)] test module imports and basic test scaffolding

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Update the terraform check
- Cleaning up compiler warnings
- Remove copying files that no longer exist
- Trying to get better error messages
- Ignore the terraform state files
- Allow manual deploys for testing

## [0.1.4] - 2025-04-30

### <!-- 1 -->🐛 Bug Fixes

- Bridge log crate events to tracing for Application Insights ([#117](https://github.com/pvandervelde/merge_warden/issues/117))
- Bridge log crate events to tracing for Application Insights

## [0.1.3] - 2025-04-29

### <!-- 1 -->🐛 Bug Fixes

- Use ManagedIdentityCredential for Key Vault access in Azure Functions ([#114](https://github.com/pvandervelde/merge_warden/issues/114))
- Use ManagedIdentityCredential for Key Vault access in Azure Functions

## [0.1.2] - 2025-04-28

### <!-- 1 -->🐛 Bug Fixes

- Set the workspace_id for the AppInsights workspace ([#109](https://github.com/pvandervelde/merge_warden/issues/109))

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Actually create the variable before using it ([#111](https://github.com/pvandervelde/merge_warden/issues/111))
- Update the release pr script to wait for the creation of the release branch ([#110](https://github.com/pvandervelde/merge_warden/issues/110))

## [0.1.1] - 2025-04-27

### <!-- 1 -->🐛 Bug Fixes

- Update azure function to use the appropriate ApplicationInsights connection string ([#104](https://github.com/pvandervelde/merge_warden/issues/104))

## [0.1.0] - 2025-04-26

### <!-- 0 -->⛰️  Features

- Migrate release process to release-please ([#73](https://github.com/pvandervelde/merge_warden/issues/73))
- Add the ability to deploy merge warden to an Azure function ([#43](https://github.com/pvandervelde/merge_warden/issues/43))
- Re-add the comments and make the PR message more generic ([#42](https://github.com/pvandervelde/merge_warden/issues/42))
- Enhance Azure Functions Specification ([#30](https://github.com/pvandervelde/merge_warden/issues/30))
- Add LLM prompting files ([#28](https://github.com/pvandervelde/merge_warden/issues/28))
- Add LLM prompting files
- Add cli executable ([#23](https://github.com/pvandervelde/merge_warden/issues/23))
- Add the developer platform crate ([#15](https://github.com/pvandervelde/merge_warden/issues/15))
- Add the core library ([#1](https://github.com/pvandervelde/merge_warden/issues/1))

### <!-- 1 -->🐛 Bug Fixes

- Read GitHub App key from file in Terraform apply ([#66](https://github.com/pvandervelde/merge_warden/issues/66))
- Read GitHub App key from file in Terraform apply
- Update rust crate dirs to v6 ([#25](https://github.com/pvandervelde/merge_warden/issues/25))
- Update rust crate dirs to v6
- Update the cargo deny configuration

### <!-- 3 -->📚 Documentation

- Update Azure Functions specification

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Fix publish release again1 ([#99](https://github.com/pvandervelde/merge_warden/issues/99))
- Fix the branch target for the publishing of the release ([#95](https://github.com/pvandervelde/merge_warden/issues/95))
- Create a script to create the release branch ([#89](https://github.com/pvandervelde/merge_warden/issues/89))
- Create an insert point in the release notes ([#87](https://github.com/pvandervelde/merge_warden/issues/87))
- Don't commit cargo.lock as we're not changing it ([#86](https://github.com/pvandervelde/merge_warden/issues/86))
- Update the way we set the Cargo version ([#85](https://github.com/pvandervelde/merge_warden/issues/85))
- Tweaking version calc ([#84](https://github.com/pvandervelde/merge_warden/issues/84))
- Switching from knope to conventional_commits_next_version ([#83](https://github.com/pvandervelde/merge_warden/issues/83))
- Change from release-please to our own set of algorithms ([#82](https://github.com/pvandervelde/merge_warden/issues/82))
- Release ([#80](https://github.com/pvandervelde/merge_warden/issues/80))
- Release
- Tweaking release please ([#79](https://github.com/pvandervelde/merge_warden/issues/79))
- More fixing the release ([#77](https://github.com/pvandervelde/merge_warden/issues/77))
- Fix release please config ([#76](https://github.com/pvandervelde/merge_warden/issues/76))
- Set up proper environments for the app deployment ([#75](https://github.com/pvandervelde/merge_warden/issues/75))
- When checking out, checkout on an actual ref ([#71](https://github.com/pvandervelde/merge_warden/issues/71))
- Release 0.1.0 ([#70](https://github.com/pvandervelde/merge_warden/issues/70))
- Release 0.1.0
- Give permissions to upload to a GitHub release ([#69](https://github.com/pvandervelde/merge_warden/issues/69))
- Chasing tf issues ([#67](https://github.com/pvandervelde/merge_warden/issues/67))
- Add the config files for roo-code ([#65](https://github.com/pvandervelde/merge_warden/issues/65))
- Read the GitHub app key from a file in terraform ([#64](https://github.com/pvandervelde/merge_warden/issues/64))
- Fix deployment one more time ([#61](https://github.com/pvandervelde/merge_warden/issues/61))
- Trying to fix the build ([#60](https://github.com/pvandervelde/merge_warden/issues/60))
- Fixing the release even better ([#58](https://github.com/pvandervelde/merge_warden/issues/58))
- Fixing the release even better
- More deploy and release-plz updates ([#57](https://github.com/pvandervelde/merge_warden/issues/57))
- Updating the settings for the release notes. ([#56](https://github.com/pvandervelde/merge_warden/issues/56))
- Only release the changes when we are ready ([#54](https://github.com/pvandervelde/merge_warden/issues/54))
- Around and around we go ([#53](https://github.com/pvandervelde/merge_warden/issues/53))
- Final fix, here's hoping ([#52](https://github.com/pvandervelde/merge_warden/issues/52))
- Update release plz config ([#49](https://github.com/pvandervelde/merge_warden/issues/49))
- Fixing the release-plz and terraform deployments ([#46](https://github.com/pvandervelde/merge_warden/issues/46))
- Add the PR and issue templates ([#26](https://github.com/pvandervelde/merge_warden/issues/26))
- Give the cargo clippy check more permissions

## 0.0.0

- Created project
