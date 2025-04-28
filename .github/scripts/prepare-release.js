#!/usr/bin/env node

/**
 * Script to prepare a release branch and make a "Verified" commit using the GitHub GraphQL API.
 *
 * Usage:
 *   node .github/scripts/prepare-release.js <next_version> <release_notes_file>
 *
 * Environment variables required:
 *   - GH_TOKEN: GitHub App token with repo write access
 *   - GITHUB_REPOSITORY: "owner/repo" string
 *
 * What this script does:
 *   1. Determines the default branch of the repository.
 *   2. Ensures a release branch (release/<next_version>) exists, creating it if needed.
 *   3. Reads and updates CHANGELOG.md and Cargo.toml for the new version.
 *   4. Uses the GitHub GraphQL API to create a "Verified" commit with both file changes.
 *   5. Opens a pull request if one does not already exist.
 *
 * This approach ensures the commit is attributed to the GitHub App and marked as "Verified" by GitHub.
 *
 * NOTE: Minimal Node.js knowledge is required to run this script. All configuration is via arguments and environment variables.
 */

const fs = require('fs');
const https = require('https');

// Parse command-line arguments
const [, , NEXT_VERSION, RELEASE_NOTES_FILE] = process.argv;

if (!NEXT_VERSION || !RELEASE_NOTES_FILE) {
  console.error('Usage: node .github/scripts/prepare-release.js <next_version> <release_notes_file>');
  process.exit(1);
}

// Read required environment variables
const GH_TOKEN = process.env.GH_TOKEN;
const GITHUB_REPOSITORY = process.env.GITHUB_REPOSITORY;

if (!GH_TOKEN || !GITHUB_REPOSITORY) {
  console.error('GH_TOKEN and GITHUB_REPOSITORY environment variables are required.');
  process.exit(1);
}

// Parse owner and repo from GITHUB_REPOSITORY
const [OWNER, REPO] = GITHUB_REPOSITORY.split('/');
const BRANCH_NAME = `release/${NEXT_VERSION}`;
const CHANGELOG_PATH = 'CHANGELOG.md';
const CARGO_PATH = 'Cargo.toml';

/**
 * Helper to call the GitHub GraphQL API.
 * @param {string} query - GraphQL query or mutation string
 * @param {object} variables - Variables for the query/mutation
 * @returns {Promise<object>} - Parsed JSON response
 */
function githubGraphQL(query, variables) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify({ query, variables });
    const options = {
      hostname: 'api.github.com',
      path: '/graphql',
      method: 'POST',
      headers: {
        'Authorization': `bearer ${GH_TOKEN}`,
        'User-Agent': 'prepare-release-script',
        'Content-Type': 'application/json',
        'Content-Length': data.length,
        'Accept': 'application/vnd.github+json'
      }
    };
    const req = https.request(options, res => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        if (res.statusCode !== 200) {
          reject(new Error(`GitHub GraphQL API error: ${res.statusCode} ${body}`));
        } else {
          resolve(JSON.parse(body));
        }
      });
    });
    req.on('error', reject);
    req.write(data);
    req.end();
  });
}

/**
 * Helper to call the GitHub REST API.
 * Used for file content retrieval and branch creation.
 * @param {string} path - REST API path (e.g., /repos/owner/repo/...)
 * @param {string} method - HTTP method (default: GET)
 * @param {object|null} body - Request body for POST/PATCH (default: null)
 * @returns {Promise<object>} - Parsed JSON response
 */
function githubRest(path, method = 'GET', body = null) {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'api.github.com',
      path,
      method,
      headers: {
        'Authorization': `bearer ${GH_TOKEN}`,
        'User-Agent': 'prepare-release-script',
        'Accept': 'application/vnd.github+json'
      }
    };
    const req = https.request(options, res => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        if (res.statusCode < 200 || res.statusCode >= 300) {
          reject(new Error(`GitHub REST API error: ${res.statusCode} ${data}`));
        } else {
          resolve(JSON.parse(data));
        }
      });
    });
    req.on('error', reject);
    if (body) req.write(JSON.stringify(body));
    req.end();
  });
}

async function main() {
  // --- Handle stale release branches and PRs ---
  console.log(`[INFO] Checking for stale release branches...`);
  // List all remote branches matching release/*
  const branchesResp = await githubRest(`/repos/${OWNER}/${REPO}/branches?per_page=100`);
  const releaseBranches = branchesResp.filter(b => b.name.startsWith('release/'));
  for (const branch of releaseBranches) {
    const branchVersion = branch.name.replace('release/', '');
    if (branch.name !== BRANCH_NAME) {
      console.log(`[INFO] Found stale release branch: ${branch.name} (version: ${branchVersion})`);
      // Find open PRs for this branch
      const prsResp = await githubGraphQL(`
        query($owner: String!, $repo: String!, $head: String!) {
          repository(owner: $owner, name: $repo) {
            pullRequests(headRefName: $head, states: OPEN, first: 1) {
              nodes { number url }
            }
          }
        }
      `, { owner: OWNER, repo: REPO, head: branch.name });
      if (prsResp.data.repository.pullRequests.nodes.length > 0) {
        const pr = prsResp.data.repository.pullRequests.nodes[0];
        console.log(`[INFO] Closing PR #${pr.number} for stale branch ${branch.name}...`);
        // Close the PR
        await githubRest(`/repos/${OWNER}/${REPO}/pulls/${pr.number}`, 'PATCH', { state: 'closed' });
      } else {
        console.log(`[INFO] No open PR found for stale branch ${branch.name}.`);
      }
      // Delete the remote branch
      console.log(`[INFO] Deleting stale remote branch ${branch.name}...`);
      try {
        await githubRest(`/repos/${OWNER}/${REPO}/git/refs/heads/${branch.name}`, 'DELETE');
        console.log(`[INFO] Successfully deleted branch ${branch.name}.`);
      } catch (error) {
        console.error(`[ERROR] Failed to delete branch ${branch.name}:`, error.message || error);
      }
    }
  }
  // 1. Get the default branch name (e.g., "main" or "master")
  // This is needed to know which branch to base the release branch on.
  console.log(`[INFO] Fetching default branch for ${OWNER}/${REPO}...`);
  const repoInfo = await githubGraphQL(`
    query($owner: String!, $repo: String!) {
      repository(owner: $owner, name: $repo) {
        defaultBranchRef { name }
      }
    }
  `, { owner: OWNER, repo: REPO });
  const DEFAULT_BRANCH = repoInfo.data.repository.defaultBranchRef.name;
  console.log(`[INFO] Default branch is: ${DEFAULT_BRANCH}`);

  // 2. Get the latest commit OID (object ID) on the default branch
  // This is needed to create the release branch if it doesn't exist.
  console.log(`[INFO] Fetching latest commit OID for default branch (${DEFAULT_BRANCH})...`);
  const branchInfo = await githubGraphQL(`
    query($owner: String!, $repo: String!, $branch: String!) {
      repository(owner: $owner, name: $repo) {
        ref(qualifiedName: $branch) {
          target { ... on Commit { oid } }
        }
      }
    }
  `, { owner: OWNER, repo: REPO, branch: `refs/heads/${DEFAULT_BRANCH}` });
  const baseCommitOid = branchInfo.data.repository.ref.target.oid;
  console.log(`[INFO] Latest commit OID on ${DEFAULT_BRANCH}: ${baseCommitOid}`);

  // 3. Check if the release branch exists
  let branchExists = false;
  try {
    await githubRest(`/repos/${OWNER}/${REPO}/git/refs/heads/${BRANCH_NAME}`);
    branchExists = true;
    console.log(`[INFO] Release branch "${BRANCH_NAME}" already exists.`);
  } catch {
    branchExists = false;
    console.log(`[INFO] Release branch "${BRANCH_NAME}" does not exist. Will create it.`);
    // Branch does not exist, create it from the default branch's latest commit
    await githubRest(`/repos/${OWNER}/${REPO}/git/refs`, 'POST', {
      ref: `refs/heads/${BRANCH_NAME}`,
      sha: baseCommitOid
    });
    console.log(`[INFO] Created release branch "${BRANCH_NAME}" from ${DEFAULT_BRANCH} (${baseCommitOid}).`);
  }

  // 4. Check if an open PR already exists for this release branch
  console.log(`[INFO] Checking for existing open PRs for branch "${BRANCH_NAME}"...`);
  const prs = await githubGraphQL(`
    query($owner: String!, $repo: String!, $head: String!) {
      repository(owner: $owner, name: $repo) {
        pullRequests(headRefName: $head, states: OPEN, first: 1) {
          nodes { number url }
        }
      }
    }
  `, { owner: OWNER, repo: REPO, head: BRANCH_NAME });

  if (branchExists && prs.data.repository.pullRequests.nodes.length > 0) {
    // Both branch and PR exist, do not proceed
    console.log(`[INFO] Both release branch "${BRANCH_NAME}" and an open PR already exist.`);
    console.log(`[INFO] Existing PR: ${prs.data.repository.pullRequests.nodes[0].url}`);
    console.log(`[INFO] Exiting without making changes.`);
    return;
  }

  // 5. Prepare updated file contents for CHANGELOG.md and Cargo.toml

  // CHANGELOG.md: Prepend release notes before the first "## " header, or create if missing
  let changelogContent = '';
  try {
    const changelogResp = await githubRest(`/repos/${OWNER}/${REPO}/contents/${CHANGELOG_PATH}?ref=${BRANCH_NAME}`);
    const orig = Buffer.from(changelogResp.content, 'base64').toString('utf8');
    let releaseNotes = '';
    try {
      releaseNotes = fs.readFileSync(RELEASE_NOTES_FILE, 'utf8');
    } catch (err) {
      console.error(`[ERROR] Error reading release notes file "${RELEASE_NOTES_FILE}": ${err.message}`);
      process.exit(1);
    }
    // Insert release notes before first "## " header
    // Check if the "## " header exists and insert release notes accordingly
    if (/^## /m.test(orig)) {
      // Insert release notes before the first "## " header
      changelogContent = orig.replace(/^## /m, `${releaseNotes}\n\n## `);
    } else {
      // Prepend release notes to the beginning of the file
      changelogContent = `${orig}\n\n${releaseNotes}\n`;
    }
    console.log(`[INFO] Prepared updated CHANGELOG.md content.`);
  } catch {
    // File does not exist, create new
    try {
      changelogContent = `## Changelog\n\n${fs.readFileSync(RELEASE_NOTES_FILE, 'utf8')}\n`;
      console.log(`[INFO] CHANGELOG.md did not exist on branch, created new content.`);
    } catch (err) {
      console.error(`[ERROR] Error reading release notes file "${RELEASE_NOTES_FILE}": ${err.message}`);
      process.exit(1);
    }
  }

  // Cargo.toml: Update the version field
  let cargoContent = '';
  try {
    const cargoResp = await githubRest(`/repos/${OWNER}/${REPO}/contents/${CARGO_PATH}?ref=${BRANCH_NAME}`);
    const orig = Buffer.from(cargoResp.content, 'base64').toString('utf8');
    cargoContent = orig.replace(/^version = ".*"$/m, `version = "${NEXT_VERSION}"`);
    console.log(`[INFO] Prepared updated Cargo.toml content with version ${NEXT_VERSION}.`);
  } catch {
    console.error(`[ERROR] Cargo.toml must exist on the branch "${BRANCH_NAME}".`);
    throw new Error('Cargo.toml must exist on the branch');
  }

  // 6. Get the latest commit OID on the release branch (for optimistic concurrency)
  // Retry logic for fetching the latest commit OID on the release branch
  console.log(`[INFO] Fetching latest commit OID for release branch "${BRANCH_NAME}" with retry...`);
  let relBranchInfo = null;
  let retries = 5;
  let delayMs = 2000;
  for (let attempt = 1; attempt <= retries; attempt++) {
    relBranchInfo = await githubGraphQL(`
      query($owner: String!, $repo: String!, $branch: String!) {
        repository(owner: $owner, name: $repo) {
          ref(qualifiedName: $branch) {
            target { ... on Commit { oid } }
          }
        }
      }
    `, { owner: OWNER, repo: REPO, branch: `refs/heads/${BRANCH_NAME}` });
    if (relBranchInfo.data.repository.ref) {
      break;
    }
    if (attempt < retries) {
      console.warn(`[WARN] Could not find ref for branch "${BRANCH_NAME}" (attempt ${attempt}/${retries}). Retrying in ${delayMs}ms...`);
      await new Promise(res => setTimeout(res, delayMs));
    }
  }
  if (!relBranchInfo || !relBranchInfo.data.repository.ref) {
    console.error(`[ERROR] Could not find ref for branch "${BRANCH_NAME}" after ${retries} attempts.`);
    throw new Error(`Could not find ref for branch "${BRANCH_NAME}" after ${retries} attempts.`);
  }
  const releaseBranchOid = relBranchInfo.data.repository.ref.target.oid;
  console.log(`[INFO] Latest commit OID on "${BRANCH_NAME}": ${releaseBranchOid}`);

  // 7. Create a "Verified" commit on the release branch using the GraphQL API
  // This will update both files in a single commit
  const commitMutation = `
    mutation($input: CreateCommitOnBranchInput!) {
      createCommitOnBranch(input: $input) {
        commit { oid url }
      }
    }
  `;
  const input = {
    branch: {
      repositoryNameWithOwner: `${OWNER}/${REPO}`,
      branchName: BRANCH_NAME
    },
    message: {
      headline: `chore(release): prepare for ${NEXT_VERSION}`,
      body: "Generated by GitHub Actions."
    },
    fileChanges: {
      additions: [
        {
          path: CHANGELOG_PATH,
          contents: Buffer.from(changelogContent, 'utf8').toString('base64')
        },
        {
          path: CARGO_PATH,
          contents: Buffer.from(cargoContent, 'utf8').toString('base64')
        }
      ]
    },
    expectedHeadOid: releaseBranchOid
  };

  // Perform the commit
  console.log(`[INFO] Creating verified commit on branch "${BRANCH_NAME}"...`);
  const commitResp = await githubGraphQL(commitMutation, { input });
  const commitUrl = commitResp.data.createCommitOnBranch.commit.url;
  console.log(`[INFO] Created verified commit: ${commitUrl}`);

  // 8. Create a pull request if one does not already exist for this branch
  if (prs.data.repository.pullRequests.nodes.length === 0) {
    // No open PR, create one
    console.log(`[INFO] No open PR found for branch "${BRANCH_NAME}". Creating new PR...`);
    const prMutation = `
      mutation($input: CreatePullRequestInput!) {
        createPullRequest(input: $input) {
          pullRequest { number url }
        }
      }
    `;
    // We need the repositoryId for the mutation
    const repoIdInfo = await githubGraphQL(`
      query($owner: String!, $repo: String!) {
        repository(owner: $owner, name: $repo) {
          id
        }
      }
    `, { owner: OWNER, repo: REPO });
    const repositoryId = repoIdInfo.data.repository.id;
    const prInput = {
      repositoryId: repositoryId,
      baseRefName: DEFAULT_BRANCH,
      headRefName: BRANCH_NAME,
      title: `chore(release): ${NEXT_VERSION}`,
      body: `Prepare release ${NEXT_VERSION}. Please review the changes and merge to trigger the release.`
    };
    const prResp = await githubGraphQL(prMutation, { input: prInput });
    if (
      prResp &&
      prResp.data &&
      prResp.data.createPullRequest &&
      prResp.data.createPullRequest.pullRequest &&
      prResp.data.createPullRequest.pullRequest.url
    ) {
      console.log(`[INFO] Created PR: ${prResp.data.createPullRequest.pullRequest.url}`);
    } else {
      console.error('[ERROR] Failed to create PR. Response:', JSON.stringify(prResp, null, 2));
    }
  } else {
    // PR already exists
    console.log(`[INFO] PR already exists: ${prs.data.repository.pullRequests.nodes[0].url}`);
  }
}

// Entry point: run main() and handle errors
main().catch(err => {
  console.error(err);
  process.exit(1);
});
