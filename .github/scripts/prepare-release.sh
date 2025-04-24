#!/usr/bin/env bash
set -euo pipefail

# This script prepares a release branch, updates files, commits, and creates a PR using the GitHub CLI (gh)
# All file updates and commits are performed via the GitHub API, so the commit is attributed to the GitHub App.
# Required environment variables: GH_TOKEN, GITHUB_REPOSITORY

NEXT_VERSION="${1:?Missing next version argument}"
RELEASE_NOTES_FILE="${2:?Missing release notes file argument}"

REPO="${GITHUB_REPOSITORY}"
OWNER="${REPO%%/*}"
REPO_NAME="${REPO##*/}"
BRANCH_NAME="release/${NEXT_VERSION}"

# Get the default branch using the GitHub API
DEFAULT_BRANCH=$(gh repo view "${OWNER}/${REPO_NAME}" --json defaultBranchRef --jq .defaultBranchRef.name)

# Create the release branch from the default branch using the API
BASE_SHA=$(gh api \
  -H "Accept: application/vnd.github+json" \
  "/repos/${OWNER}/${REPO_NAME}/git/ref/heads/${DEFAULT_BRANCH}" \
  --jq .object.sha)

# Create the branch (ignore error if it exists)
gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  "/repos/${OWNER}/${REPO_NAME}/git/refs" \
  -f ref="refs/heads/${BRANCH_NAME}" \
  -f sha="${BASE_SHA}" || true

# Prepare updated file contents
RELEASE_NOTES=$(cat "$RELEASE_NOTES_FILE")
CHANGELOG_PATH="CHANGELOG.md"
CARGO_PATH="Cargo.toml"

# Download the current files from the branch
gh api \
  -H "Accept: application/vnd.github+json" \
  "/repos/${OWNER}/${REPO_NAME}/contents/${CHANGELOG_PATH}?ref=${BRANCH_NAME}" \
  --output "${CHANGELOG_PATH}.orig" || touch "${CHANGELOG_PATH}.orig"

if [ -s "${CHANGELOG_PATH}.orig" ]; then
    awk '/^## / && !inserted {print notes; inserted=1} 1' notes="$RELEASE_NOTES" "${CHANGELOG_PATH}.orig" > "${CHANGELOG_PATH}"
else
    printf "## Changelog\n\n%s\n" "$RELEASE_NOTES" > "${CHANGELOG_PATH}"
fi

# Download Cargo.toml
gh api \
  -H "Accept: application/vnd.github+json" \
  "/repos/${OWNER}/${REPO_NAME}/contents/${CARGO_PATH}?ref=${BRANCH_NAME}" \
  --output "${CARGO_PATH}.orig"

cp "${CARGO_PATH}.orig" "${CARGO_PATH}"
sed -i "s/^version = \".*\"/version = \"${NEXT_VERSION}\"/" "${CARGO_PATH}"

# Get SHAs for update
CHANGELOG_SHA=$(gh api \
  -H "Accept: application/vnd.github+json" \
  "/repos/${OWNER}/${REPO_NAME}/contents/${CHANGELOG_PATH}?ref=${BRANCH_NAME}" \
  --jq .sha || echo "")

CARGO_SHA=$(gh api \
  -H "Accept: application/vnd.github+json" \
  "/repos/${OWNER}/${REPO_NAME}/contents/${CARGO_PATH}?ref=${BRANCH_NAME}" \
  --jq .sha)

# Base64 encode file contents
CHANGELOG_B64=$(base64 -w 0 "${CHANGELOG_PATH}")
CARGO_B64=$(base64 -w 0 "${CARGO_PATH}")

# Committer info (GitHub App)
COMMITTER_NAME="github-actions[bot]"
COMMITTER_EMAIL="41898282+github-actions[bot]@users.noreply.github.com"

# Update CHANGELOG.md via API
gh api \
  --method PUT \
  -H "Accept: application/vnd.github+json" \
  "/repos/${OWNER}/${REPO_NAME}/contents/${CHANGELOG_PATH}" \
  -f message="chore(release): update CHANGELOG for ${NEXT_VERSION}" \
  -f content="${CHANGELOG_B64}" \
  -f branch="${BRANCH_NAME}" \
  -f committer[name]="${COMMITTER_NAME}" \
  -f committer[email]="${COMMITTER_EMAIL}" \
  ${CHANGELOG_SHA:+-f sha="${CHANGELOG_SHA}"}

# Update Cargo.toml via API
gh api \
  --method PUT \
  -H "Accept: application/vnd.github+json" \
  "/repos/${OWNER}/${REPO_NAME}/contents/${CARGO_PATH}" \
  -f message="chore(release): update Cargo.toml for ${NEXT_VERSION}" \
  -f content="${CARGO_B64}" \
  -f branch="${BRANCH_NAME}" \
  -f committer[name]="${COMMITTER_NAME}" \
  -f committer[email]="${COMMITTER_EMAIL}" \
  -f sha="${CARGO_SHA}"

# Create PR if it doesn't exist
EXISTING_PR=$(gh pr list --head "${BRANCH_NAME}" --state open --json number --jq '.[0].number // empty')
if [ -z "$EXISTING_PR" ]; then
    gh pr create \
      --base "${DEFAULT_BRANCH}" \
      --head "${BRANCH_NAME}" \
      --title "chore(release): ${NEXT_VERSION}" \
      --body "Prepare release ${NEXT_VERSION}. Please review the changes and merge to trigger the release."
else
    echo "PR #$EXISTING_PR already exists for branch $BRANCH_NAME"
fi
