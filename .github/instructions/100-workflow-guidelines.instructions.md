---
applyTo: "**"
---

# Copilot Instructions

## workflow-guidelines

**wf-coding-flow:** The coding flow is as follows:
1. Create an issue for the task. Follow the guidelines in `wf-issue-use`, `wf-find-issue`, `wf-issue-template`, and `wf-issue-creation`.
2. Create a design document for the task. Follow the guidelines in `wf-design-before-code` and `wf-design-spec-layout`.
3. Create a branch for the task. Follow the guidelines in `wf-branch-selection` and the source control guidelines.
4. Write code for the task. Follow the guidelines in `wf-code-tasks`, `wf-code-style`, and the language specific guidelines.
5. Write tests for the code. Follow the guidelines in `wf-code-tasks`, `wf-code-style`, `wf-unit-test-coverage`, and `wf-test-methods` and the language specific guidelines.
6. Document the code. Follow the guidelines in `wf-documentation`.
7. Create a pull request for the code. Follow the guidelines in `wf-pull-request` and the source control guidelines.
8. Review and merge the pull request.

**wf-issue-use:** Before starting any task determine if you need an issue for it. If so search for the
appropriate issue in the issue tracker. If there is no issue, suggest to create one.

**wf-find-issue:** When searching for issues
do an approximate comparison of the issue title and description with the task at hand. If you find multiple
issues that are an approximate match, ask the user to clarify which issue should be used.

**wf-issue-template:** When creating an issue use the issue templates. Issue templates can be found in the
`./.github/ISSUE_TEMPLATE` directory.

**wf-issue-creation:** All issues should be created in the repository. This includes bugs, new features,
and any other changes to the codebase. Issues should be created for all tasks, even if they are small.
Issues should be linked together to show the relationship between them.

**wf-branch-selection:** Each task is done on its own branch. Before you start a task check that you are on the
correct branch. Code is *never* directly committed to the `main` or `master` branches. If no
suitable branch exist create a new local branch from `main` or `master` for your changes and switch to that branch.
For example `git checkout -b feature-add-new-user main` or `git checkout -b fix-bug-in-login-page master`.

**wf-design-before-code:** Before writing any code for a new feature or bug fix, create a design document
that outlines the architecture, data flow, and any other relevant details. Place design documents in the
`specs` directory of the repository.

**wf-design-spec-layout:** The design document should be in markdown format and any diagrams should
should follow the mermaid language. Follow the markdown style guide and ensure that lines are no
longer than 100 characters. It should follow the following structure:
- Title
- Problem description
- Surrounding context
- Proposed solution
  - Design goals
  - Design constraints
  - Design decisions
  - Alternatives considered
- Design
  - Architecture
  - Data flow
  - Module breakdown
  - Other relevant details
- Conclusion

**wf-code-tasks:** Coding starts with an implementation issue. During the session we only solve the
implementation issue. If we find other changes that we want to make, we create new issues for
them.

**wf-code-style:** All code should be easy to understand and maintain. Use clear and descriptive
names for variables, functions, and classes. Always follow the coding standards and best practices
for the programming language being used.

**wf-unit-test-coverage:** All business logic should be covered by unit tests. We're aiming to cover
all input and output paths of the code. This includes edge cases and error handling. Use coverage
tools to measure the test coverage and use mutation testing to ensure that the tests are
effective.

**wf-unit-test-check:** When you think you've solved the presented problem, run all available tests. Fix any issues that
you find.

**wf-unit-test-create-new:** Whenever you create a new test, run it to verify that it passes. If it doesn't pass, revise
either the test (or the code it tests) until the test passes.

**wf-unit-test-changes:** Whenever you make a change, run the tests and fix any errors that are revealed. Fix one error at
a time and provide an explanation of why you think the change you made fixes the error

**wf-test-methods:** Employ different test approaches to get good coverage of both happy path
and error handling. Consider approaches like unit tests, property based testing, fuzz testing,
integration tests, end-to-end tests, and performance tests. Use the appropriate testing
frameworks and tools for the programming language being used.

**wf-documentation:** The coding task is not complete without documentation. All code should be
well-documented. Use comments to explain the purpose of complex code and to provide context for
future developers. Use docstrings to document functions, classes, and modules. The documentation
should be clear and concise.

**wf-documentation-standards:** Follow the documentation standards and best practices for the
programming language being used.

**wf-ci:** All changes should be checked with a continuous integration (CI) tool before being
merged into the main branch. Use CI tools to run tests, check code style, and perform other checks
automatically.

**wf-pull-request:** Create a pull request (PR) for all changes made to the codebase.
The PR should include a description which changes were made, why the changes were made, links to
relevant issue numbers, results from testing, and any other relevant information. Assign the PR to the
person who created it. Always invite copilot on the review.

**wf-release-management:** Use a release management tool to manage the release process. This
includes creating release notes, tagging releases, and managing version numbers. Use semantic
versioning to version releases. Use a language specific tool if it is available, otherwise use
something like `release-please` or `semantic-release` to automate the release process.

**wf-release-notes:** All releases should have release notes that describe the changes made in
the release. This includes new features, bug fixes, and any other relevant information. Use a
consistent format for release notes to make them easy to read and understand.

**wf-deployment:** All code should be deployed to a staging environment before being deployed to
production. This will help ensure that the code is working as expected and that there are no
regressions. Use continuous integration and continuous deployment (CI/CD) tools to automate the
deployment process.


