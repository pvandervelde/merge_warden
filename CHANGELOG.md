# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.3] - 2026-07-02

### Bug Fixes

- **ci**: stage server release assets before upload [3682938010df2f5618edb348f41d07df683d5960]
- release asset upload by excluding server artifact directories (#326) [3fed6ffe7a330c5cb70ce333b0d1b6d6164a3a00]


### Chores

- Initial plan [39f4efb56a9fa55051f71299f22b61e069e43729]

## [0.7.2] - 2026-07-01

### Bug Fixes
- **ci**: prevent crates.io HTTP2 download failures and catch them in PR builds [8d0e0c897494b226b9249154c2b84e47796cdf80]
- **ci**: prevent crates.io HTTP2 download failures and catch them in PR builds (#322) [57d957125445f58a1365c3dff0cda70bff2e22e0]
- **ci**: promote cargo network env vars to workflow level [ee299cf0a09a407479fd94ec59bbeba383e63694]
### Chores
- **deps**: update actions/checkout action to v7 [8105b518524f4284e3a1e6f9fd17b633a7128742]
- **deps**: update actions/checkout action to v7 (#313) [6f0d81635c9c993cc7be3971f9ef4d932c082eb5]
- **deps**: update github artifact actions [f9287c4281b1f93b2da67b7169db8c81d7b57afb]
- **deps**: update github artifact actions (major) (#315) [e1b842aa09c6ad7cd52ba297b0e03f8829768552]
- **deps**: update rust crate anyhow to v1.0.103 [security] [edc9e73242ffedc5bf74e76547459f9fd198fecc]
- **deps**: update rust crate anyhow to v1.0.103 [security] (#320) [4cb1ee11a7989b595854cb5a3a3b7b79db72bed0]
- Adding a VSCode settings file to get a sensible window title [9df38d0f53ddddba6d157aadb6f6faf5440fbfa8]
- Adding a VSCode settings file to get a sensible window title (#321) [c19c013e5fb47329585c96552f277829f168f804]

## [0.7.1] - 2026-06-29

### Bug Fixes

- **release**: use predicate-path for SBOM attestation (#318) [4c6d1a5621d50e82bb3f9f3a8f4445e8768d30ae]
- **release**: use predicate-path instead of predicate for SBOM attestation [76103a9cbb20a8e60ff124c183d95cadb4490c5c]

## [0.7.0] - 2026-06-29

### Features
- **config**: Add types, docs, and tests for conditional org policies [747f09c10d6f044424fb0bb837e02deec341d92f]
- **config**: Implement conditional org policy evaluation [8f59b70eb1f72cea94f967e6216251fa1aae3260]
- **config**: add conditional org policies based on repository topics and custom properties (#270) [77d52b403ca6effe44c182edfd2c3a88a5f86729]
- **config**: add org-level policy configuration with four-tier merge chain (#268) [e899ad031968f29b2229cec909908d5f44c8b8cd]
- **config**: add types and tests for org-level policy configuration [a7d3d27df5b00365309b73ad2f31da4f798ea899]
- **config**: allow bypass rules in org-level policy sections [3f90787f265ef101a82a3cfb14291e227fd439bb]
- **config**: allow bypass rules in org-level policy sections (#311) [70e1e602afdd70bf49cbf9d73ac40f60933860ab]
- **config**: implement org-level policy four-tier config resolution [2cec2de18eb04cd2018f8790a81d768a2b90250a]
- **core**: add PolicySet struct and merge method stubs [c39ec15f7e38f5aba52dbdf04dbfa0e7b574de4a]
- **core**: implement PolicySet merge methods and constructors [b6581c499da16ffccf50030cdeba846cac40d74d]
- **developer-platforms**: add CommitStatus model [cadc87272a6ddc76f35938d28ee20795ddfbe1a8]
- **developer-platforms**: add CommitStatus model (#1.0) (#302) [5b4d2ebb7dd43b27a7f7b8d1fb71e0c43a6c9bb4]
- **release**: add SBOM generation and build attestations [e32ab371256fb671348d13a43977189450dce783]
- **release**: add SBOM generation and build attestations (#267) [2e424de573d364c37b8535a31bcd31856afda7a6]
- **stability**: add Renovate stability label management (FR008) (#303) [75f597b33af5a6725fc9866fba41b3213460f9d0]
- **stability**: implement Renovate stability label management [844878ebb128405318b69085ab4447dfcb1917fc]
### Bug Fixes
- **audit**: Address spec-feedback issues from task 6.0 review [ec74fd29692fcf60fe23ed994cebb5455a4bc296]
- **clippy**: replace cloned ref in slice with std::slice::from_ref [e628bda304f8b3a0547dc97f746c3b2834bc316c]
- **config**: add parse_repo_config to fix four-tier merge chain double-application bug [802870db41481202d67d0d5b333aaa04247507d0]
- **config**: address PR #270 reviewer comments [1bfffd7adf0b1913a6df9038a79f6f818a5160f2]
- **config**: address PR review comments on org bypass rules [f92d5361c0c9abfc899df5dbf76a2925f7bacf58]
- **config**: address all PR #268 review comments [485bcd664680c71167671b4828e13e72dba9faa4]
- **config**: warn when repo opt-out neutralises org-default bypass rules [22eee17d54722c8f0cd36afb083a4e289f98a348]
- **config**: write bypass_rules back in load_merge_warden_config [f22c464987299a5bc8ef7c99040b3c96c1b71e1c]
- **core**: address PR review comments on PolicySet implementation [aae16916ce7a14f19470b83af40844453fc2b7f7]
- **deps**: update rust crate mockall to 0.14 [f3c8eb31a78ef710098d140443be1537b1be293b]
- **deps**: update rust crate mockall to 0.14 (#278) [6c26a0a1301c2f629887c2c29f6e87dea0e3fc5f]
- **deps**: update rust crate octocrab to 0.53 [bd335f343670efc360c9f2e3de84e33e3544541f]
- **deps**: update rust crate octocrab to 0.53 (#279) [cc0e2071159729ec283c0821f03feffd494515b4]
- **deps**: update rust crate rand to 0.9 [security] [cb72fb71a8f384be52b0c5d20ac51c9f996a11f0]
- **deps**: update rust crate rand to 0.9 [security] (#273) [4f565b6bf76e80f33daad9bfa7d5fa17bf2a454a]
- **deps**: update rust crate reqwest to 0.13.0 [bd8386f0f8948ec154e5d3744f878b5204a0cc48]
- **deps**: update rust crate reqwest to 0.13.0 (#280) [0cddd83f5fdfb6ae0bd435fc8171efe81afe7484]
- **integration-tests**: retry create_branch; improve octocrab error messages [f811692903b4a76e9913b461b0cda5b8c7aa4970]
- **integration-tests**: serialize error_handling tests; retry create_repository [9288dfe076c3ce462eb797c02544a800bbf29f5f]
- **integration-tests**: use Pulls API for get_pr_labels; treat labels as best-effort [a5b2f2ffb12eefdfdf26acf4da64c7799242e078]
- **labels**: add missing status labels and fix duplicate description [82ef9528a4621ee9bbc9fe21e9943a8f6ee280c4]
- **release**: add contents:read to publish-container; dynamic artifact count check [b741c69db223de70128a676d857f41ea13f3cde0]
- **stability**: address PR review comments [a9f0c6abddc67c8cebcee355677e0552c15fd9b9]
- Renovate config contains invalid setting [de309072fdc95ed939e4b59462fb436256460c3d]
- Renovate config contains invalid setting (#272) [f9e8d8b3fa944db0174b9884e5d404c8a24b2bb4]
### Documentation
- **architecture**: add policy engine ADR and interface spec [1dd8ecab49a1e84d55d19e8bb7fd21f8a7722e88]
- **catalog**: register CommitStatus model in agent catalog [a9e77ad0e6f905b718822e2fcb97d7715b9a5724]
- **config**: add ADR and spec for org-level policy configuration [80b4b3016b7ecb58c1bd9e907062c853d47987e0]
- **config**: add four-tier policy docs, sample TOML, and integration tests [5c23cb7a52f74b7d62a00f9fd32806c8b01f2dd7]
- **samples**: expand org policy sample with prState, issuePropagation, thresholds, change_type_labels [a54c4627acfaf8b47a83ebab5f403c5e503eb6bf]
- **security**: document org policy repository trust threat model [ec78d26a026c4a8f23cddbce0c0a12e675a24682]
- **spec**: add renovate/stability-days label management specifications (#298) [295ba67d0207a5d402c910410a8a061e4417872a]
- **spec**: add renovate/stability-days label management specifications (#299) [4ecf5555552d6eb9a9635090b502ea2b03b8b444]
- **spec**: address PR review comments on renovate stability specs (#298) [0e36492ef488e2cdad8a2e52a8c117533aed9863]
- add Commit statuses permission and webhook event for Renovate stability (FR008) [02a8e47df8d00f320d77468676364aface269efb]
- add Commit statuses permission and webhook for Renovate stability (FR008) (#308) [ff51fd6487ea7ca913aa7a3405f2068b259f54fa]
- address PR review comments on documentation gaps [8f9ef5adedcf9651759fcd5e21c0ffbafbf90abf]
- fill gaps in user documentation [e4c251f39bb5baee99e9e732c02977e115533396]
- fill gaps in user documentation (#307) [b8460a64e113c0caf77c7555e4d1703f0154f604]
- fix two remaining review comments [d98ccc5fce00cb0e364944fc5e06cad9b6fc2c9f]
### Code Refactoring
- **config**: extract BypassRulesConfig::to_bypass_rules to eliminate duplication [f041e022735f12b86fd96241b30137443098e298]
- **core**: introduce PolicySet merge abstraction to replace ad-hoc config merging (#265) [199207ad9ac534381c814b2054b7cd849f5bc76c]
- **core**: replace ad-hoc merge block with PolicySet in load_merge_warden_config [a670af4d47adb897111c4be59fcfb402454060c8]
- **release**: trigger on release:published, not PR close [c32df05e89ed8aa0bcb1813eafffc23c8773b927]
### Tests
- **audit**: Add WireMock tests for get_repository_context [4337aaf1de992c2b348c7829e53bd19fb4d89cfd]
- **config**: add mutation kill tests for task 5 org-policy implementation [938ccbe2cae707cecc595aa17bf9c112788e5c0c]
- **config**: add org-level bypass rules tests [a55b0f827d563b1b7b2cd71ba228c0c0c9376973]
- **config**: expand org-level policy test coverage to all spec §9 scenarios [1d9afc9b142a7e3a861deac0ce8255cec767ac12]
- **core**: add adversarial test suite for PolicySet merge methods [786ef2de80898481508210525417085015b57dbc]
- **core**: add mutation kill tests for PolicySet merge methods [a552fd6a74d6908397246a74040938749b357e27]
- **models**: add CommitStatus mutation kill test for description key presence [7ea07d254c81311efd754b3e9c5a1d3d9cf7ebbd]
- **models**: add CommitStatus test suite (TDD pre-implementation) [508084c3a75033dc19888f02fec6e2f4fe347fd0]
### Continuous Integration
- **workflows**: consolidate PR checks with security gate priority (#271) [0de9ee0d251efd0419159d6019e73f4a32f55a73]
- Add TEST_ORGANIZATION secret reference [05e2b95285471a150e77640f902c7552a34e7a6e]
- Fix jq query in the osv step [dd26ba1603d1465730dbc82321cfb97806ed301b]
- Improve OSV output [2643dcc81df07ac162a25760816acad36926b9f1]
- Only error on the osv scan if the issue is fixable [64baf5e2e2bfb4ce1c853eb63394f38a7ddbb81e]
- PR workflow runs security audit before running tasks [81c6ec75fc4202c9820d6b056888fa5c1992a39f]
- Tweaking the OSV output [0d63dbd9fb52fca324f30bdbde90f1e7fc5f9476]
- Tweaking the security scan section in pr.yml [71a9657a2fe7521267afd9f7319cff7cc4fd2bf1]
### Chores
- **config**: update merge-warden config and label update script [2b8ed51dc46ad0584855635630657749f30022dd]
- **config**: update merge-warden config and label update script (#246) [42b58bf51b631b1619ecbdaae8fc310f19ee5ed8]
- **deps**: bump hickory-net from 0.26.0 to 0.26.1 [cf73bcaee6ae015345fa6a5ad9d130ff3bd8057b]
- **deps**: bump hickory-net from 0.26.0 to 0.26.1 (#238) [b32178ec68ae5cadeb0c3d19d4b14440f0d69a40]
- **deps**: bump hickory-proto from 0.26.0 to 0.26.1 [ffd54e0265af0f2977d41a107fd5b3e5b1c4cc5e]
- **deps**: bump hickory-proto from 0.26.0 to 0.26.1 (#237) [d609be7b3f4471290b4ef3eb398e3edc435f9d88]
- **deps**: bump openssl from 0.10.72 to 0.10.80 [0bc96bd270f4cc5a7a3d064e2fb9e43ecd0f950d]
- **deps**: bump openssl from 0.10.72 to 0.10.80 (#261) [9c48d7010e2138600c416c961026ce546630e148]
- **deps**: bump rand from 0.8.5 to 0.10.1 [06a9f5e79f03ce3afd743fd2f2017b42d11f7cda]
- **deps**: bump rand from 0.8.5 to 0.10.1 (#224) [5d2c8a78b6f6f6d22f535ea3d132832cea7688c0]
- **deps**: pin dependencies [3eb7db984fb0f2a37cc0d9a6c46efe2380fbf4b4]
- **deps**: pin dependencies (#284) [49be2c5bfdde9a2bcf46870333d0516415dee824]
- **deps**: update actions/checkout action to v6 [cdff9d7d4eed55be3014b6e176b2d64871499c49]
- **deps**: update actions/checkout action to v6 (#281) [2dfd604ce95d07d1929e47c132dc5bf10b808a4d]
- **deps**: update actions/checkout digest to df4cb1c [d19a2d595368aa20e45adb05b77f63a5856dc956]
- **deps**: update actions/checkout digest to df4cb1c (#285) [8f7b2214566eb381dc331724f750412f6f10bf8b]
- **deps**: update actions/deploy-pages action to v5 [0e9c76664230e45cd89b2f1bc4b46967d31c7ad7]
- **deps**: update actions/deploy-pages action to v5 (#293) [0342d572f5034830e534c458c05204f9226f4e5c]
- **deps**: update actions/setup-python action to v6 [06fcc90d458393709f03007aa19949fad00fd4e7]
- **deps**: update actions/setup-python action to v6 (#294) [578d742ebe20fd1b7b19de8045d866f5b79963f4]
- **deps**: update actions/upload-pages-artifact action to v5 [483b2bada0bd24d35c6ad4aed4929d1fa902966d]
- **deps**: update actions/upload-pages-artifact action to v5 (#295) [6462997c1f2a57c1a5c2c1d3e1c848eed6a08b72]
- **deps**: update amannn/action-semantic-pull-request action to v6 [0bd76aaa54f11fe4d12d89105a23c86dd65f05df]
- **deps**: update amannn/action-semantic-pull-request action to v6 (#282) [31ec0ccba0c93694f5a71d2b7f6cadfc9880c19b]
- **deps**: update anthropics/claude-code-action digest to a92e7c7 [e3976c899a6dfc0dda63f7f908975f4641da370e]
- **deps**: update anthropics/claude-code-action digest to a92e7c7 (#286) [6b13cd7aa9bdea325074bc2b29c46c6e218eac87]
- **deps**: update codecov/codecov-action action to v6 [798d5c6ac3884059e6cd3d7d65953db5fb653b05]
- **deps**: update codecov/codecov-action action to v6 [ed818da93c6cf5b44f499438622cb3f0ed46e695]
- **deps**: update codecov/codecov-action action to v6 (#296) [063c73a6f88b05cba6f5e3c9a7468f326cada904]
- **deps**: update codecov/codecov-action action to v6 (#300) [b9040cd3189290fe88ffbcc2a130edbea9e532e9]
- **deps**: update codecov/codecov-action action to v7 [60b04ba547523c2bbb0c5bf8bab819dd8d49daa4]
- **deps**: update codecov/codecov-action action to v7 (#310) [816522811cf5c4d694cfcdb82173ebed0b68e279]
- **deps**: update dependency ubuntu to v24 [a37270bcffe23ab799694880046a3249bde259de]
- **deps**: update dependency ubuntu to v24 (#317) [6f0475da85e9608bbe72c39df064c13f207afa99]
- **deps**: update docker/build-push-action action to v7 [b695fc9955948088e994c12d7565bb13020ac1f0]
- **deps**: update docker/build-push-action action to v7 (#290) [5b8747e7d0afe4e508fc29d6d4f3e990dabf52ae]
- **deps**: update docker/login-action action to v4 [e15bf32f3eb93449817d1d83e95934dd59367d9c]
- **deps**: update docker/login-action action to v4 (#291) [f721b3ab49303c2259bcf1ffb0f4adf3b39fc974]
- **deps**: update docker/setup-buildx-action action to v4 [4541acbeacef83780b6465c68f9a2bfb72c73af7]
- **deps**: update docker/setup-buildx-action action to v4 (#292) [ab1b17742b1cdaa3b5a98043254c4663b9e42d7f]
- **deps**: update gcr.io/distroless/cc-debian13 docker digest to a017e74 [06c7ad52270b3aeed3be2a41a27fa92911d1a2aa]
- **deps**: update gcr.io/distroless/cc-debian13 docker digest to a017e74 (#316) [f230ec33cfc4362661fd7a08115e8afb13700738]
- **deps**: update github artifact actions [baf1884d7347517a998f404d6d2b92aa056569cc]
- **deps**: update github artifact actions (major) (#283) [4a3c15447b09c5ba6aa891f0a05d7bdcb3ee9ae9]
- **deps**: update rust crate jsonwebtoken to v10 [security] [e10e9b836cd8c534877c4d732be9408bc50e190e]
- **deps**: update rust crate jsonwebtoken to v10 [security] (#274) [5918e8b8962312f35bd5767ec3e0a4b69860cfae]
- **deps**: update rust crate proptest to v1.11.0 [ca301648049b0a0061c873c0a337b5b2b269122a]
- **deps**: update rust crate proptest to v1.11.0 (#276) [47e7d6a5ea54ce90417e5d7a45374be3bd970814]
- **deps**: update rust crate rand to v0.9.3 [security] [9f097ee0da8d48adef4e38d580ca915d81bcac7f]
- **deps**: update rust crate rand to v0.9.3 [security] (#287) [ba8a7477a5034fb190fb33a7d0c2e0e8e32a1962]
- **deps**: update rust crate tempfile to v3.27.0 [14a6fb5b61e233ed0af1b2454921c8b28328b853]
- **deps**: update rust crate tempfile to v3.27.0 (#277) [f5745ce430e562aed39f58fede0b1f0ce7ebb684]
- **deps**: update swatinem/rust-cache digest to e18b497 [e0ea3daf54632cdc04a6c91953608f8600920818]
- **deps**: update swatinem/rust-cache digest to e18b497 (#275) [e42a31f034302d2e39b708973e3b93465dd64358]
- **deps**: update taiki-e/install-action digest to 9bcaee1 [fcf7f59072ebe43340cc785ab237390006e57267]
- **deps**: update taiki-e/install-action digest to 9bcaee1 (#153) [ce02d1f1845ca2b3f2c4459d8cd359ed31bd76a5]
- **deps**: update taiki-e/setup-cross-toolchain-action digest to 3d9770c [a12439b573c21abc8a888d9df76582d11858c7c4]
- **deps**: update taiki-e/setup-cross-toolchain-action digest to 3d9770c (#177) [ad1367c478872f9158c9637a9776d4a130bba017]
- Fix Rust format [afff35370c51952e0e090c18c46c084479e7a6b4]
- Fix compiler errors due to Octocrab update [6e687fa6b4be140d222ac36d6d1c6240718832d6]
- Fix rust formatting [42e5e0363052a292631fc86dc32bdd83235cc5aa]
- Fix rust formatting [846215897bb94b8d9b85319ed1fb317f96604386]
- Fixing minor issues [44b298587f60c5c87be9d46d00b684a73511f407]
- Merge-warden should ignore deletes in size calcs [e9cab2569774202c4e002c1d1411795cebc07982]
- Merge-warden should ignore deletes in size calcs (#297) [a68ba238c76908210720c9b084989b1e421da1df]
- Renovate configuration update to only run once a week [f38ca63261225b8fd80917d9b5380d2aa62c91b8]
- Renovate configuration update to only run once a week (#304) [fbaaeb507af7b04c5e2414af34c9d791b7a42724]
- Rust formatting fix [d81fbfdfac13194b9ec843cdad49c3440d976163]
- Update the catalog [d4594511159ebba4e519305ab84c48bf8492659f]
- Update the catalog [ada7c88d2928069284b791b0ed9b49023e46518b]
- Updating the release regent configuration [d00772b980a3f255c59cb89c425b35d978b42696]
- Updating the release regent configuration (#264) [810998cea98f940f21c44e9d83dc2210c3466d1e]
### Deps
- update Rust dependencies (axum, toml, keyring, opentelemetry, jsonwebtoken) (#309) [77b47471eb601d68b3cd8bdbfae9088ec1237a31]
- update Rust dependencies from Renovate branches [89b4db702e6b024cef64ba848855b0c9b4ac03e2]
### Config
- pr workflow comments are incorrect [119a1320d63cc47c6cefc19674fe89d45485b656]

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




