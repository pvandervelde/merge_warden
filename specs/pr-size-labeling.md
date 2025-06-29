# Design: Automatic PR Size Labeling with Optional Oversized PR Check Failure

## Title

Automatic PR Size Labeling with Optional Oversized PR Check Failure

## Problem Description

Currently, merge_warden has no visual indication of pull request size, making it difficult for
reviewers to quickly assess the scope and complexity of a PR before starting their review.
Additionally, there's no mechanism to prevent oversized PRs from being merged, which can lead to
inefficient reviews and reduced code quality.

Research shows that PRs over 250 lines are significantly harder to review effectively, with
review quality decreasing substantially for PRs over 500 lines. This feature will help teams
maintain better code quality through more manageable PR sizes.

## Surrounding Context

- merge_warden currently validates PR titles (conventional commits) and work item references
- The application uses a TOML-based configuration system with schema versioning
- There's an existing label management system and GitHub check status integration
- The codebase follows a modular architecture with separate core logic and developer platform integration
- Configuration is repository-specific via `.github/merge-warden.toml`

## Proposed Solution

Implement automatic labeling of pull requests based on the number of lines changed, using
industry-standard size categories, with an optional check that can fail oversized PRs.

### Size Categories

| Label | Size Range | Color | Description |
|-------|------------|-------|-------------|
| `size/XS` | 1-10 lines | `#3CBF00` | Trivial changes |
| `size/S` | 11-50 lines | `#5D9801` | Small changes |
| `size/M` | 51-100 lines | `#A8A800` | Medium changes |
| `size/L` | 101-250 lines | `#DFAB00` | Large changes |
| `size/XL` | 251-500 lines | `#FE6D00` | Extra large changes |
| `size/XXL` | 500+ lines | `#FE2C01` | Should be split - **FAILS CHECK** |

### Alternatives Considered

1. **Manual labeling**: Requires reviewers to manually assess and label PRs, which is inconsistent and error-prone
2. **File count-based sizing**: Less accurate than line count for determining review complexity
3. **Third-party integrations**: Would add external dependencies and complexity
4. **Branch protection without size awareness**: Doesn't provide context about why a PR might be problematic
5. **Hard-coded size limits**: Less flexible than configurable thresholds for different project needs

## Design

### Configuration Schema Extension

#### Application-Level Configuration (CLI defaults)

Application defaults that can be overridden by repository configuration:

```toml
# CLI application configuration
[default]
# ... existing fields

[policies]
# ... existing policies

# New PR size check defaults
[policies.pr_size_check]
enabled = false  # Disabled by default for backward compatibility
fail_oversized_prs = false
max_pr_size = 500
thresholds = { xs = 10, s = 50, m = 100, l = 250, xl = 500 }
exclude_patterns = [
    "*.lock",
    "*.generated.*",
    "*.min.*",
    "docs/",
    "test/fixtures/"
]

# Fallback label creation settings (used when auto-creating labels)
# These settings control appearance and cannot be overridden by repository config
[policies.pr_size_check.fallback_labels]
format = "size: {category}"  # Label name format
colors = { xs = "#3CBF00", s = "#5D9801", m = "#A8A800", l = "#DFAB00", xl = "#FE6D00", xxl = "#FE2C01" }
descriptions = {
    xs = "Size: XS (1-10 lines)",
    s = "Size: S (11-50 lines)",
    m = "Size: M (51-100 lines)",
    l = "Size: L (101-250 lines)",
    xl = "Size: XL (251-500 lines)",
    xxl = "Size: XXL (500+ lines)"
}
```

#### Repository-Level Configuration

Extend the existing repository TOML configuration schema:

```toml
schemaVersion = 1

# Existing configuration sections...
[policies.pullRequests.prTitle]
# ... existing fields

[policies.pullRequests.workItem]
# ... existing fields

# New PR size check configuration (overrides application defaults)
[policies.pullRequests.prSizeCheck]
enabled = true
fail_oversized_prs = false  # Set to true to fail checks for XXL PRs
max_pr_size = 500          # Lines threshold for XXL classification
thresholds = { xs = 10, s = 50, m = 100, l = 250, xl = 500 }
exclude_patterns = [
    "*.lock",
    "*.generated.*",
    "*.min.*",
    "docs/",
    "test/fixtures/"
]

# Note: Repository configuration cannot override fallback label appearance (colors, format, descriptions)
# Label visual properties are controlled exclusively by application configuration
```

#### Bypass Rules Integration

Extend existing bypass rules to support PR size validation:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BypassRules {
    pub title_convention: BypassRule,
    pub work_items: BypassRule,
    pub pr_size_check: BypassRule,  // New bypass rule for PR size validation
}
```

Configuration example with bypass rules:

```toml
# Application configuration with bypass rules
[policies.bypass_rules]
# ... existing bypass rules

[policies.bypass_rules.pr_size_check]
enabled = true
users = ["release-bot", "dependabot[bot]", "admin"]
```

### Smart Label Discovery Strategy

Instead of creating new labels, the system will intelligently discover and use existing repository labels that indicate PR size:

#### Label Detection Algorithm

1. **Name-based detection**: Search for labels with size indicators as standalone words
   - Regex pattern: `\b(size[/-]?)?(XS|S|M|L|XL|XXL)\b` (case-insensitive)
   - Examples: `size/M`, `size-L`, `XL`, `Size: Small`, `PR-XS`

2. **Description-based detection**: Search label descriptions for size metadata
   - Pattern: `\(size:\s*(XS|S|M|L|XL|XXL)\)` (case-insensitive)
   - Examples: `"Documentation changes (size: XS)"`, `"Major refactor (size: XL)"`

#### Label Selection Priority

1. **Exact size match**: `size/XS`, `size/S`, etc.
    - Examples: `size/XS`, `size/S`, `size/M`, etc.
2. **Size with separator**: `size-M`, `size: L`, etc.
   - Examples: `size-M`, `size_L`, `size: M`, `
3. **Standalone size**: `XS`, `M`, `XXL`, etc.
4. **Description-based**: Any label with `(size: M)` in description
5. **No match**: Skip labeling (log warning for manual label creation)

#### Fallback Label Creation

If no suitable labels are found and the repository allows label creation:

- Create labels with format: `size: {category}`
- **Color and text are sourced exclusively from application configuration** (no repository override)
- Use the predefined colors from the requirements table
- Include descriptive text: `"Size: XS (1-10 lines)"`

**Note**: When the application creates fallback labels, the label format, colors, and descriptive text are controlled entirely by the application-level configuration. Repository-specific configurations cannot override the visual appearance of auto-created labels to ensure consistency across all repositories using merge_warden.

```rust
#[derive(Debug, Clone)]
pub struct DiscoveredSizeLabels {
    pub xs: Option<String>,
    pub s: Option<String>,
    pub m: Option<String>,
    pub l: Option<String>,
    pub xl: Option<String>,
    pub xxl: Option<String>,
}

pub struct LabelDiscovery {
    config: PrSizeCheckConfig,
}

impl LabelDiscovery {
    /// Discover existing size labels in the repository
    pub async fn discover_size_labels(
        &self,
        provider: &dyn PullRequestProvider,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<DiscoveredSizeLabels, Error> {
        // Implementation will search all repository labels
        // and match them against size patterns
    }
}
```

### Educational PR Comment System

When `fail_oversized_prs` is enabled and a PR exceeds the size threshold, the system will post an educational comment explaining why smaller PRs are beneficial.

#### Comment Template

```markdown
<!-- PR_SIZE_CHECK_OVERSIZED -->
## 📏 Pull Request Size Notice

This pull request is **too large** ({total_lines} lines changed) and exceeds the configured limit of {max_size} lines.

### 🔬 The Science Behind Smaller PRs

Research in software engineering shows that smaller pull requests lead to:

- **Better Review Quality**: Studies indicate that review effectiveness drops significantly after 250 lines, with defect detection rates falling below 60% for PRs over 500 lines
- **Faster Review Cycles**: Smaller PRs are reviewed 40% faster on average, reducing overall development cycle time
- **Reduced Cognitive Load**: Reviewers can maintain focus and provide more thorough feedback on smaller changes
- **Lower Defect Rates**: Smaller PRs have 15% fewer defects that escape to production compared to larger ones

### 💡 Recommended Actions

1. **Split by Feature**: Break this PR into smaller, logically related changes
2. **Incremental Approach**: Consider implementing features in multiple phases
3. **Extract Refactoring**: Separate pure refactoring changes from new functionality
4. **Documentation Separately**: Move documentation updates to a separate PR

### 📊 Size Analysis

- **Total lines changed**: {total_lines}
- **Files modified**: {file_count}
- **Size threshold**: {max_size} lines
- **Recommended target**: Under 250 lines for optimal review

{exclusion_info}

### 🔧 Configuration

This check can be adjusted by repository maintainers in `.github/merge-warden.toml`.
The size limits are configurable to match your team's workflow preferences.

---
*This check helps maintain code quality through more manageable pull request sizes.*
```

#### Comment Context Variables

```rust
#[derive(Debug, Clone)]
pub struct OversizedPrCommentContext {
    pub total_lines: u32,
    pub file_count: usize,
    pub max_size: u32,
    pub exclusion_info: String,  // Details about excluded files if any
}
```

### Data Model Extensions

#### New Models for PR File Changes

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestFile {
    /// The file path
    pub filename: String,

    /// Number of lines added
    pub additions: u32,

    /// Number of lines deleted
    pub deletions: u32,

    /// Total changes (additions + deletions)
    pub changes: u32,

    /// File status (added, modified, deleted, renamed)
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct PrSizeInfo {
    /// Total lines changed (excluding filtered files)
    pub total_lines_changed: u32,

    /// List of files included in the count
    pub included_files: Vec<PullRequestFile>,

    /// List of files excluded from the count
    pub excluded_files: Vec<PullRequestFile>,

    /// The size category determined
    pub size_category: PrSizeCategory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrSizeCategory {
    XS,  // 1-10 lines
    S,   // 11-50 lines
    M,   // 51-100 lines
    L,   // 101-250 lines
    XL,  // 251-500 lines
    XXL, // 500+ lines
}
```

#### PullRequestProvider Extension

```rust
#[async_trait]
pub trait PullRequestProvider {
    // ... existing methods

    /// Gets the list of files changed in a pull request
    async fn get_pull_request_files(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<PullRequestFile>, Error>;
}
```

### Core Logic Architecture

#### Size Calculation Logic

```rust
pub struct PrSizeChecker {
    config: PrSizeCheckConfig,
    label_discovery: LabelDiscovery,
}

impl PrSizeChecker {
    /// Calculate the size of a PR based on file changes
    pub fn calculate_pr_size(&self, files: &[PullRequestFile]) -> PrSizeInfo {
        // Filter out excluded files using glob patterns
        // Sum additions + deletions for included files
        // Determine size category based on configurable thresholds
    }

    /// Check if a file should be excluded from size calculation
    fn is_file_excluded(&self, filename: &str) -> bool {
        // Use glob pattern matching against exclude_patterns
    }

    /// Determine size category from total lines changed using configurable thresholds
    fn categorize_size(&self, total_lines: u32) -> PrSizeCategory {
        // Apply configured thresholds (xs, s, m, l, xl, xxl) to determine category
        // Support for custom thresholds per repository
    }

    /// Apply the appropriate size label to the PR
    pub async fn apply_size_label(
        &self,
        provider: &dyn PullRequestProvider,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        size_info: &PrSizeInfo,
        discovered_labels: &DiscoveredSizeLabels,
    ) -> Result<(), Error> {
        // Remove any existing size labels (exclusive labeling)
        // Apply the new size label based on discovered labels
    }

    /// Post educational comment for oversized PRs
    pub async fn post_oversized_comment(
        &self,
        provider: &dyn PullRequestProvider,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        context: &OversizedPrCommentContext,
    ) -> Result<(), Error> {
        // Check if comment already exists to avoid duplicates
        // Post educational comment with size analysis and recommendations
    }
}
```

#### Integration with Main Validation Pipeline

Extend the `MergeWarden::process_pull_request()` method:

```rust
impl<P: PullRequestProvider + std::fmt::Debug> MergeWarden<P> {
    pub async fn process_pull_request(&self, /*...*/) -> Result<CheckResult, MergeWardenError> {
        // ... existing validation logic for title and work items

        // New: PR size validation with bypass support
        let size_result = if self.config.pr_size_check.enabled {
            self.check_pr_size(repo_owner, repo_name, &pr).await?
        } else {
            validation_result::ValidationResult::valid()
        };

        // Update labels to include size labels (exclusive)
        // Update check status to include size validation results
        // Handle size validation failures for oversized PRs
        // Post educational comment if oversized and blocking enabled
    }

    async fn check_pr_size(&self, repo_owner: &str, repo_name: &str, pr: &PullRequest)
        -> Result<validation_result::ValidationResult, MergeWardenError> {

        // Check for bypass rules first
        if let Some(bypass_info) = self.check_size_bypass(&pr) {
            return Ok(validation_result::ValidationResult::bypassed(bypass_info));
        }

        // Get PR files and calculate size
        let files = self.provider.get_pull_request_files(repo_owner, repo_name, pr.number).await?;
        let size_info = self.size_checker.calculate_pr_size(&files);

        // Discover existing size labels in repository
        let discovered_labels = self.size_checker.label_discovery
            .discover_size_labels(&self.provider, repo_owner, repo_name).await?;

        // Apply appropriate size label (exclusive)
        self.size_checker.apply_size_label(
            &self.provider, repo_owner, repo_name, pr.number,
            &size_info, &discovered_labels
        ).await?;

        // Check if PR is oversized and blocking is enabled
        let is_oversized = matches!(size_info.size_category, PrSizeCategory::XXL)
            && size_info.total_lines_changed > self.config.pr_size_check.max_pr_size;

        if is_oversized && self.config.pr_size_check.fail_oversized_prs {
            // Post educational comment about PR size
            let comment_context = OversizedPrCommentContext {
                total_lines: size_info.total_lines_changed,
                file_count: size_info.included_files.len(),
                max_size: self.config.pr_size_check.max_pr_size,
                exclusion_info: self.build_exclusion_info(&size_info.excluded_files),
            };

            self.size_checker.post_oversized_comment(
                &self.provider, repo_owner, repo_name, pr.number, &comment_context
            ).await?;

            return Ok(validation_result::ValidationResult::invalid(
                "PR exceeds maximum size limit and should be split into smaller changes"
            ));
        }

        Ok(validation_result::ValidationResult::valid())
    }

    fn check_size_bypass(&self, pr: &PullRequest) -> Option<validation_result::BypassInfo> {
        // Check if user is in PR size bypass list
        // Similar to existing title and work item bypass logic
    }
}
```

### Label Management Strategy

- **Smart Discovery**: Use existing repository labels when possible via pattern matching
- **Exclusive size labels**: Only one size label should be present per PR
- **Automatic updates**: Size labels should be updated when PR content changes
- **Label cleanup**: Remove previous size labels before applying new ones
- **Fallback creation**: Create standard size labels only if none exist and repository allows it
- **Bypass aware**: Respect bypass rules and log bypass usage for audit trails

### Check Status Integration

Extend the existing check status system to include size validation:

- **Success case**: "All PR requirements satisfied (Size: M, 78 lines)"
- **Bypass case**: "All PR requirements satisfied (Size validation bypassed)"
- **Failure case**: "PR is too large (XXL, 650+ lines). Consider splitting into smaller PRs for better reviewability."
- **Educational failure**: Include link to posted comment with detailed guidance
- **Disabled case**: Size checking not mentioned in status

### File Exclusion Logic

Use glob pattern matching for flexible file exclusion:

- Support patterns like `"*.lock"`, `"docs/"`, `"test/fixtures/**"`
- Default exclusions for common generated files (configurable at app level)
- Repository-specific exclusions override or extend app defaults
- Configurable per repository via TOML configuration
- Clear logging of excluded files for transparency

### Error Handling Strategy

- **API failures**: Graceful degradation if PR files cannot be fetched
- **Label discovery failures**: Fall back to creating standard labels or skip labeling
- **Configuration errors**: Log warnings and use app-level defaults
- **Pattern matching errors**: Log errors and include all files (safe default)
- **Bypass rule conflicts**: Log conflicts and default to allowing (non-blocking)
- **Comment posting failures**: Log warnings but don't fail the entire check

## Other Relevant Details

### Performance Considerations

- **API efficiency**: Single call to fetch all PR files rather than individual file requests
- **Label discovery caching**: Cache discovered labels per repository to minimize API calls
- **Large PRs**: Implement reasonable limits to prevent excessive API usage (e.g., max 1000 files)
- **Asynchronous processing**: Use asynchronous operations for all API calls to prevent blocking
- **Smart label updates**: Only update labels when size category actually changes

### Security Considerations

- **Input validation**: Validate file patterns to prevent regex injection attacks
- **Path traversal**: Ensure file paths are properly sanitized
- **Rate limiting**: Respect GitHub API rate limits when fetching file data
- **Pattern safety**: Use safe glob pattern matching libraries to prevent ReDoS attacks
- **Comment injection**: Sanitize any user-provided data included in educational comments
- **Label creation**: Validate label names before creation to prevent malicious labels

### Backward Compatibility

- **Schema versioning**: New configuration is optional and defaults to disabled
- **Existing behavior**: No changes to existing title and work item validation
- **Migration path**: Repositories can opt-in by updating their configuration

### Testing Strategy

1. **Unit tests**: Size calculation logic, pattern matching, categorization, label discovery
2. **Integration tests**: End-to-end validation with mock PR data and label scenarios
3. **Configuration tests**: TOML parsing, validation, app vs repo config precedence
4. **Error handling tests**: API failures, malformed data, edge cases
5. **Bypass rule tests**: Verify bypass functionality works correctly
6. **Label discovery tests**: Test various label naming conventions and fallbacks
7. **Comment generation tests**: Verify educational comments are posted correctly

## Conclusion

This design extends merge_warden's validation capabilities with automatic PR size labeling and
optional size-based check failures. The implementation follows existing architectural patterns,
maintains backward compatibility, and provides flexible configuration options to meet different
team needs.

The feature will help development teams maintain better code quality through more manageable PR
sizes while providing immediate visual feedback about review complexity.
