# Roo Instructions - debug

## general

**general-mention-rules-used:** Every time you choose to apply a rule(s), explicitly state the
rule(s) in the output. You can use the `rule` tag to do this. For example, `#rule: rule_name`.

**general-mention-knowledge:** List all assumptions and uncertainties you need to clear up before
completing this task.

**general-confidence-check:** On a 1-10 scale, where 10 is absolute conviction backed by overwhelming evidence, rate your confidence in each recommendation you are giving me.
Don't hedge - if something is a 3, say it is a 3 and explain why. If it is a 9 defend that high rating.
Do this before saving files, after saving, after rejections, and before task completion

**general-grounding:** Always verify and validate information from multiple sources. Cross-reference findings from
different tools and document results and sources

**general-focus:** Focus on the task at hand. Avoid distractions and stay on topic.
If you need to switch tasks, make sure to finish the current task first.

**general-memory-bank:** Use a memory bank to store information that is relevant to the task at hand.
This can include code snippets, documentation, and other resources. Use the memory bank to help you stay on track and avoid distractions.


## tooling

**general-tool-use-os:** Use operating system relevant tools when possible. For example, use
`bash` on Linux and MacOS, and `powershell` on Windows

**general-tool-use-file-search:** When searching for files in the workspace make sure to also
search hidden directories (e.g. `./.github`, `./.vscode`, etc.). But skip the `.git` directory.


## scm

**scm-hygiene:** Commit changes frequently and in small increments. Follow the `scm-commit-message` format for commit messages. Use
`git fetch --prune` and `git pull` to update your local branch before pushing changes.

**scm-git-pull-request-title:** The pull request title should follow the conventional commit format.
`<type>(<scope>): <subject>` where `type` is one of the following: `feat`, `fix`, `chore`, `docs`,
`style`, `refactor`, `perf`, `test`.

**scm-git-pull-request-template:** Use the pull request template if there is one. The pull request
template can be found in the `./.github/PULL_REQUEST_TEMPLATE.md` file.

**scm-git-pull-request-review:** All pull requests should be reviewed by at least one other developer and
GitHub copilot before being merged into the main branch.

**scm-branch-naming:** The branch name should be a brief summary of the changes being made. Branch
names should be in lowercase and use hyphens to separate words. For example, `fix-bug-in-login-page`
or `feature-add-new-user`.

**scm-commit-message:** For commit messages the
type should be one of the following: `feat`, `fix`, `chore`, `docs`,
`style`, `refactor`, `perf`, `test`. The scope should be the name of the module or component being changed. The subject should
be a short description of the change. The `work_item_ref` is one of the following issue references:
`references` or `related to` followed by the issue number.
Finally those parts make the following format for commit messages:

```text
type(scope): subject

description

 references <work_item_ref>
```


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

**wf-pull-request:** Create a pull request (PR) for all changes made to the codebase.
The PR should include a description which changes were made, why the changes were made, links to
relevant issue numbers, results from testing, and any other relevant information. Assign the PR to the
person who created it. Always invite copilot on the review.


## coding

**coding-design-architecture:** Design modular, maintainable system components using appropriate technologies and frameworks. Ensure that integration
points are clearly defined and documented.

**coding-design-pseudo-code:** Use pseudo-code to outline the logic and structure of the code before implementation. This helps to clarify the
design and identify potential issues early in the development process.

**coding-whitespace:** Always leave a whitespace between a line of code and a comment. This improves readability and helps to distinguish
between code and comments.

**coding-style:** Follow the style guides for the language. Use the appropriate formatters to format your code. This will
help ensure that the code is consistent and easy to read.

**coding-comments:** Use comments to explain why the code is doing something, not what it is doing. Use comments to explain complex
logic or algorithms. Avoid using comments to explain simple code or code that is self-explanatory.


## coding-markdown

**md-lines:** Ensure that lines in markdown are no longer than 100 characters. Use proper formatting for lists, headings, and code blocks.

**md-mermaid:** In mermaid diagrams, if there is a "(" or ")" in the label, put the entire label in quotes. This is to avoid parsing errors in the mermaid parser.


## coding-rust

**rust-element-ordering:** Use the following order for elements in a module. Elements of one type
should be grouped together and ordered alphabetically. The order is as follows:
- imports - organized by standard library, third-party crates, and local modules
- constants
- traits
- structs with their implementations.
- enums with their implementations.
- functions
- the main function

**rust-documentation:** For public items documentation comments are always added. For private items
documentation comments are added when the item is complex or not self-explanatory. Use `///` for
documentation comments and `//!` for module-level documentation. Add examples to the documentation
comments when possible.

**rust-error-handling:** Use the `Result` type for functions that can return an error. Use the `?` operator
to propagate errors. Avoid using `unwrap` or `expect` unless you are certain that the value will not be
`None` or an error.

**rust-error-messages:** Use clear and descriptive error messages. Avoid using generic error messages
like "an error occurred". Instead, provide specific information about what went wrong and how to fix it.

**rust-error-types:** Use custom error types for your application. This will help you provide more
meaningful error messages and make it easier to handle errors in a consistent way. Use the `thiserror`
crate to define custom error types.

**rust-test-location:** Put unit tests in their own file. They are placed next to the file they
are testing and are named `<file_under_test>_tests.rs`. Reference them from the file under test with
an import, which is placed at the end of the other imports and usings. This will look something like:

``` rust
#[cfg(test)]
#[path = "<file_under_test>_tests.rs"]
mod tests;
```


## coding-terraform

**tf-documentation:** Add documentation comments for each resource, module, and variable.
Use the `#` symbol for comments. Use `##` for module-level documentation. Add examples to the
documentation comments when possible.


