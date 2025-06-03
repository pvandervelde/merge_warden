# Roo Instructions - ask

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

**scm-git-pull-request-review:** All pull requests should be reviewed by at least one other developer and
GitHub copilot before being merged into the main branch.


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

**wf-documentation:** The coding task is not complete without documentation. All code should be
well-documented. Use comments to explain the purpose of complex code and to provide context for
future developers. Use docstrings to document functions, classes, and modules. The documentation
should be clear and concise.

**wf-documentation-standards:** Follow the documentation standards and best practices for the
programming language being used.


## coding-markdown

**md-lines:** Ensure that lines in markdown are no longer than 100 characters. Use proper formatting for lists, headings, and code blocks.


## coding-rust

**rust-documentation:** For public items documentation comments are always added. For private items
documentation comments are added when the item is complex or not self-explanatory. Use `///` for
documentation comments and `//!` for module-level documentation. Add examples to the documentation
comments when possible.


