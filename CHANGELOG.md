# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-07-12

### <!-- 0 -->⛰️  Features

- Complete terraform migration to separate repository infrastructure from source code ([#161](https://github.com/pvandervelde/merge_warden/issues/161))
- Implement smart label detection for conventional commit types ([#157](https://github.com/pvandervelde/merge_warden/issues/157))
- Add Azure App Configuration support for smart label detection
- Implement robust non-blocking smart label detection
- Integrate smart label detection with MergeWarden core processing
- Implement label detection algorithm with three-tier search strategy
- Add labeling for PR size ([#150](https://github.com/pvandervelde/merge_warden/issues/150))
- Implement PR size check bypasses and fix compilation errors
- Add PR size configuration support to Terraform and Azure Function
- Implement smart label discovery for PR size labeling
- Extend configuration schema for PR size checking
- Implement PR size analysis foundation
- Implement comprehensive bypass capabilities with audit trails ([#146](https://github.com/pvandervelde/merge_warden/issues/146))
- Integrate Azure App Configuration for centralized config management
- Add bypass indication in check status text
- Add bypass rule management commands
- Implement bypass rules for validation checks
- Implement data models for PR author bypass rules
- Support repository-specific PR rule configuration via .github/merge-warden.toml ([#141](https://github.com/pvandervelde/merge_warden/issues/141))
- Integrate TOML config loading for merge-warden validation rules
- Add TOML-based config schema, loader, and docs for merge-warden pull request rules
- GitHub checks for merge blocking ([#137](https://github.com/pvandervelde/merge_warden/issues/137))
- Switch to status checks for merge blocking

### <!-- 1 -->🐛 Bug Fixes

- Add synchronize event to webhook processing ([#154](https://github.com/pvandervelde/merge_warden/issues/154))
- Make PR size label discovery case-insensitive
- Fix description-based size label discovery
- Add PR size configuration to ApplicationDefaults
- Add tempfile dev dependency for bypass tests
- Configuration unit tests
- Enable smart label detection in CLI mode
- Azure function unable to start and connect to Key Vault ([#130](https://github.com/pvandervelde/merge_warden/issues/130))
- Remove the reference to the local function config file
- Write debug logs in Azure
- Don't initialize the logs twice

### <!-- 2 -->🚜 Refactor

- Remove tf-test job from CI workflow
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
- Update the config file example in the readme
- Add a section about the configuration to the README
- Minor changes to the readme
- Improving the README.md

### <!-- 6 -->🧪 Testing

- Add #[cfg(test)] test module imports and basic test scaffolding
- Add tests for the config changes
- Update the configuration tests

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Update rust crate toml to 0.9 ([#158](https://github.com/pvandervelde/merge_warden/issues/158))
- Pin actions/upload-release-asset action to e8f9f06 ([#163](https://github.com/pvandervelde/merge_warden/issues/163))
- Update gittools/actions action to v4 ([#151](https://github.com/pvandervelde/merge_warden/issues/151))
- Update swatinem/rust-cache digest to 98c8021 ([#145](https://github.com/pvandervelde/merge_warden/issues/145))
- Update embarkstudios/cargo-deny-action digest to 30f817c ([#142](https://github.com/pvandervelde/merge_warden/issues/142))
- Update taiki-e/upload-rust-binary-action digest to 3962470 ([#139](https://github.com/pvandervelde/merge_warden/issues/139))
- Pin dependencies ([#147](https://github.com/pvandervelde/merge_warden/issues/147))
- Update rust crate proptest to v1.7.0 ([#132](https://github.com/pvandervelde/merge_warden/issues/132))
- Update azure/functions-action digest to 0bd707f ([#120](https://github.com/pvandervelde/merge_warden/issues/120))
- Update actions/create-github-app-token digest to df432ce ([#122](https://github.com/pvandervelde/merge_warden/issues/122))
- Update taiki-e/install-action digest to 92f69c1 ([#121](https://github.com/pvandervelde/merge_warden/issues/121))
- Update taiki-e/upload-rust-binary-action digest to db10148 ([#123](https://github.com/pvandervelde/merge_warden/issues/123))
- Update codecov/codecov-action digest to 18283e0 ([#126](https://github.com/pvandervelde/merge_warden/issues/126))
- Remove copying files that no longer exist
- Trying to get better error messages
- Ignore the terraform state files
- Allow manual deploys for testing
- Cleaning up compiler warnings
- Remove the MCP file because it's now in VsCode profiles
- Clean up the git provider
- Remove the LLM instruction files

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
