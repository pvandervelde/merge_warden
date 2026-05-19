# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-05-19

### Features

- **config**: Add tests for snake_case ApplicationDefaults TOML keys [b7faa708d3013ab3e449124179f147e061cafc8d]
- **config**: Rename ApplicationDefaults TOML keys from camelCase to snake_case [323aaf414b2bb6297b15ed991ba00000f991910d]
- **config**: align ApplicationDefaults TOML keys with snake_case convention (#242) [bcacf09aef977700b76bb77fa505645685dc18ed]
- **config-validation**: add types, docs, and tests for PR config validation (auto via agent) [32646d37dac3e07b0e024cc6b0f34f6c04a229df]
- **core**: Add types, stubs and tests for negation-aware keyword labels and suppression [cac710b4ac78811ac9fc61a9e0112b6e5924cddb]
- **core**: Implement negation-aware keyword detection, label suppression and explanation comments [845be6b31b2ad7276583160432626c52e170f097]
- **core**: add negation-aware keyword labels, comment suppression, and explanation comments (#251) [93398a5002faee1c625801125e2bec550a7db5af]
- **core**: validate config file changes in PRs and post informational comment (#256) [a3fafc98e654b00ad164448256903aea7450e750]
- **labels**: Add types, docs, and tests for configurable keyword labels [19e45e6d5565c60063e6588f35d5a6ed5c4269cb]
- **labels**: Implement configurable keyword labels [41a0c4379c32d2eb023fa0315f807a774007e03a]
- **labels**: make keyword-triggered auto-applied labels configurable (#249) [367358e59ed086c43eb2b4bd720df0003fde7299]
- **size**: add ignore_deletions field and tests for PR size calculation [8ca33e69c63c67e73360bfad5654b37099f3fa4f]
- **size**: ignore deletions when calculating PR size (#247) [828c9a0c9b71c3d613f111060eb51c352216ad5d]
- **size**: implement ignore_deletions for PR size calculation [80377ad95ee5df63e4754ce439ec1c9fe3e9b073]


### Bug Fixes

- **build**: add x86_64-unknown-linux-gnu openssl support for cross builds [8e2466e195597d2f056ca3b121ed5a2c4c180938]
- **config**: merge keyword_labels from repo config into app defaults [5222d4f815aa3eed5f8e781ff8aef55a884937ff]
- **core**: address PR review findings for config change validation [06fe6f67eeba34ec2bf921a7c44df1c7555a925f]
- **core**: address PR review findings on bypass rule precedence and spec docs [5001fbbc794f37b984c894ad46025475f6903469]
- **core**: address PR review findings on negation detection and comment lifecycle [4ae6134fdee4079562b75ec6205b0f2b5da4ba5a]
- **core**: address clippy warnings in labels.rs negation and comment functions [358d0a74b65a709afe38463eb2d33be33f3030e6]
- **core**: address second-round PR review findings [30f0e7fe91f6a36879ac2430f8fc56f1b5bdccce]
- **core**: per-repo bypass rules in merge-warden.toml were silently ignored [89a55ce562c1ba1622655a2aeb43601a5e935223]
- **core**: per-repo bypass rules in merge-warden.toml were silently ignored (#255) [cf3750709cfbc1a4802b661eec72798e41142eb1]
- **core**: replace unnecessary unwrap_err after is_ok with match [1b0f3a2787b75ce89b60225f45d11bd6a869dddb]
- **core**: wire label_prefix to manage_size_labels fallback (#253) [12da87162e1a7789fec8f1afe4d6f327e565a66b]
- **core**: wire label_prefix to manage_size_labels fallback (auto via agent) [55b5a00fa8704be16c10d226e00fbedb24d59a83]
- **labels**: make size label discovery case-insensitive [7cb541a9c7572c61625f40030697c113ae25ed80]
- **labels**: make size label discovery case-insensitive (#245) [b0814762d669e8d8159bcb95848d230e6d1a62c2]
- **release**: address PR review comments on release-regent config [de6d9ba1ca5c4e6605fb0562d8c5b314b4c852e8]
- **samples**: build Rust binary before docker build in run-local.ps1 [397589c7f76d2a76d18ae98754db1e002ca38de1]
- **samples**: point smee relay at webhook endpoint not health endpoint [31655a31619055d8377accf41f999b26b5a53665]
- cross build and smee config (#257) [bc8ccddfeb9588998060ed73dbef02288b26a52c]


### Documentation

- **catalog**: add FR-007 config validation abstractions to catalog [deabc6e69dedc8fe5cf531fa9cc661b2b657f139]
- **config**: add keyword_labels TOML examples to docstring and spec [e8693df4a379b5a88d61333e69b5b734aea22ff4]
- **size**: clarify total_lines_changed field reflects ignore_deletions mode [802dee327ae35e366c8cab2eebe52056a4a0fe22]
- **size**: document ignore_deletions field in reference docs and sample configs [f958a24b156a0cc8ce69de0c181df4b4305b0fa7]
- **spec**: add FR-007 config change validation spec [1e25a946577599300a8d682580460235bf63a2ee]
- **spec**: add interface specs for FR-007 config change validation [8186e58fef99a3b152b44db854168b278b85eebe]
- Remove obsolete reference to cliff.toml [8de860aea732334f3089615beac76c1cc2ec52a9]


### Tests

- **core**: Add adversarial tests for negation detection, suppression, and explanation comments [30d991ecac3dfccca37d8ea310b9d050caf51121]
- **core**: add idempotent config comment test; fix formatting [4d7e08e4d7671dacc4d014dff97e409e833d2b81]
- **core**: add tests for manage_size_labels label_prefix fallback wiring (auto via agent) [104e03728a9ea63a29d9c69e4125d54309daea36]
- **core**: add title-level negation integration test for breaking-change [d1d5afdccfea09b7cafd96b3b00af8f34aea7d93]
- **samples**: add config change validation scenarios to setup script [5a01a5a7196772d9343b0e78c36866fd94505dd3]


### Chores

- **release**: 0.6.0 (#259) [dacbc9b60a61fa5be9af9f90e5543065b07e37ff]
- **release**: switch to release-regent for release PR creation [3c2163ee1bc9ce0671525735923d52a85ea763b6]
- **release**: switch to release-regent for release PR creation (#239) [7ed2ec024c3314d117aaa5a1b242502e1726aea8]
- **release**: update release files for 0.6.0 [46329af62b154d694f6d6ea7772285ceb0a09898]
- Addressing PR comments [5815a086fe5a335747d2ae6c8f001b9214caa27e]
- Addressing PR comments [4467c22e3764cb2a65e17d077b762585043cbb8a]
- Fix rust formatting [50baee911135e129af553ce630980875e8ccde72]
- Update the catalog [944782867c9bfc03668d397dd96d0367fc30d20c]
- Update the renovate config [5802239dd6dfd88b68c6153ced391fe6b712d077]
- Update the renovate config (#258) [f5d72d478237a02e3462cb61fb790d4da6508b5f]
- Updating the catalog [ff70b5ffdf5c31bf9e7ca0392c0fc688f6d7e122]
- update the cargo dependencies [e916604d3b68d05497124a9cf238b1b90ce8e1e7]


### Config

- Correct the version detection for release-regent [c1a8c10d2c00eb043ea89958f5f57182940254c6]
- Disable the manifest_files option for release-regent [81084bb3850c57e70048361975d2bb5fb08c8408]
- Don't fail on large PRs [d69cf3656090feea41279799768cc6ca93eca367]

## [0.5.0] - 2026-05-07

### <!-- 0 -->⛰️  Features

- Config change validation: when `.github/merge-warden.toml` appears in a PR's
  changed files, fetch it at the PR's head SHA, validate TOML syntax and
  `schemaVersion`, and post/update/delete an informational comment on the PR.
  Validation is purely informational and never affects the check conclusion.

### <!-- 1 -->🐛 Bug Fixes

- Replace /api/merge_warden with /health and /api/github/webhook ([#235](https://github.com/pvandervelde/merge_warden/issues/235))
- Address PR review comments

### <!-- 2 -->🚜 Refactor

- Split health and webhook onto separate, conventional routes

### <!-- 3 -->⚠️ Breaking Changes

- `ConfigFetcher` trait (in `merge_warden_developer_platforms`) gains a new required
  method `fetch_config_at_ref(&self, owner, repo, path, git_ref)`.  Any external crate
  that implements `ConfigFetcher` must add this method.  The `GitHubProvider` bundled
  with this crate already implements it.

### <!-- 6 -->🧪 Testing

- Fix expected default webhook URL in config test



## [0.4.1] - 2026-05-02

### <!-- 1 -->🐛 Bug Fixes

- Pre-build server binaries natively to eliminate QEMU timeout ([#232](https://github.com/pvandervelde/merge_warden/issues/232))
- Move audit.toml to .cargo/audit.toml so cargo audit reads it
- Move cargo audit ignores to audit.toml for local and CI parity
- Stage pre-built binary in CI Docker validate, harden workflow dispatch inputs
- Pre-build server binaries natively to eliminate QEMU timeout
- Split Docker build into own job with 90min timeout and add recovery dispatch



## [0.4.0] - 2026-05-01

### <!-- 0 -->⛰️  Features

- Add deterministic codebase catalog generator ([#230](https://github.com/pvandervelde/merge_warden/issues/230))
- Add catalog generator script and per-domain documentation
- Harden installation ID resolution and clean up credential env var names ([#225](https://github.com/pvandervelde/merge_warden/issues/225))
- Rename GITHUB_APP_ID and GITHUB_APP_PRIVATE_KEY env vars to MERGE_WARDEN_ prefix
- Add resolve_installation_id to AppAuthProvider
- Propagate issue metadata for all reference keywords

### <!-- 1 -->🐛 Bug Fixes

- Address PR review issues in catalog generator
- Allow BSD-2-Clause and Zlib licenses from queue-runtime transitive deps
- Address PR review comments
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

### <!-- 2 -->🚜 Refactor

- Remove dead installation_id field from WebhookQueueMessage

### <!-- 3 -->📚 Documentation

- Add complete user documentation site with GitHub Pages deployment ([#229](https://github.com/pvandervelde/merge_warden/issues/229))
- Address PR review comments on documentation content
- Further changes to the repo README
- Rewrite README to reflect current architecture and link to docs site
- Add MkDocs config and GitHub Pages deployment workflow
- Add conceptual explanation pages
- Add reference documentation
- Add policy configuration how-to guides
- Add deployment and server setup how-to guides
- Add getting-started and first-policy tutorials
- Add user documentation landing page and structure plan
- Fix over-renamed Rust field names in server-config interface spec
- Fix stale Kubernetes env var names in deployment-architectures spec
- Update deployment docs and samples for MERGE_WARDEN_ env var prefix rename
- Remove installation_id from WebhookQueueMessage interface spec

### <!-- 6 -->🧪 Testing

- Update config tests for MERGE_WARDEN_ env var prefix rename
- Update ingress tests to assert installation_id absent from WebhookQueueMessage
- Add WireMock tests for resolve_installation_id
- Add tests covering the five bug fixes

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Adding the new icons for the app ([#231](https://github.com/pvandervelde/merge_warden/issues/231))
- Adding the new icons for the app
- Address PR review comments on workflow
- Trigger deployment on version tag, not every master push
- Fix Rust formatting
- Remove stale TDD scaffolding comments from ingress_tests
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


