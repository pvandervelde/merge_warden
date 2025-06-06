# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2025-06-06

### <!-- 1 -->🐛 Bug Fixes

- Azure function unable to start and connect to Key Vault ([#130](https://github.com/pvandervelde/merge_warden/issues/130))
- Remove the reference to the local function config file
- Write debug logs in Azure
- Don't initialize the logs twice

### <!-- 7 -->⚙️ Miscellaneous Tasks

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
