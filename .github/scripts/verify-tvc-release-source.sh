#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "Untrusted TVC release source: $*" >&2
  exit 1
}

[[ "${GITHUB_REPOSITORY:-}" == "tkhq/rust-sdk" ]] || fail "unexpected repository"
[[ "${WORKFLOW_RUN_ID:-}" =~ ^[0-9]+$ ]] || fail "invalid workflow run ID"
[[ "${EXPECTED_HEAD_SHA:-}" =~ ^[0-9a-f]{40}$ ]] || fail "invalid head SHA"

workflow_id=$(gh api "repos/${GITHUB_REPOSITORY}/actions/workflows/release-plz.yml" --jq .id)
run=$(gh api "repos/${GITHUB_REPOSITORY}/actions/runs/${WORKFLOW_RUN_ID}")
if ! jq -e --argjson workflow_id "${workflow_id}" \
  --arg repo "${GITHUB_REPOSITORY}" --arg sha "${EXPECTED_HEAD_SHA}" '
  .workflow_id == $workflow_id and
  .name == "Release-plz" and
  .path == ".github/workflows/release-plz.yml" and
  .event == "pull_request" and
  .conclusion == "success" and
  .head_repository.full_name == $repo and
  .head_sha == $sha and
  (.head_branch | startswith("release-plz-"))
' <<< "${run}" >/dev/null; then
  fail "triggering run is not the repository release workflow"
fi

head_branch=$(jq -r .head_branch <<< "${run}")
jobs=$(gh api "repos/${GITHUB_REPOSITORY}/actions/runs/${WORKFLOW_RUN_ID}/jobs?filter=latest&per_page=100")
if ! jq -e '
  [.jobs[] | select(.name == "Release-plz release")] |
  length == 1 and .[0].conclusion == "success"
' <<< "${jobs}" >/dev/null; then
  fail "release job did not succeed exactly once"
fi

# Closed PR workflow runs can omit pull_requests; look up the commit instead.
pull_requests=$(gh api "repos/${GITHUB_REPOSITORY}/commits/${EXPECTED_HEAD_SHA}/pulls?per_page=100")
if ! jq -e --arg repo "${GITHUB_REPOSITORY}" --arg sha "${EXPECTED_HEAD_SHA}" \
  --arg branch "${head_branch}" '
  any(.[];
    .merged_at != null and
    .base.ref == "main" and
    .head.repo.full_name == $repo and
    .head.sha == $sha and
    .head.ref == $branch and
    any(.labels[]?; .name == "release")
  )
' <<< "${pull_requests}" >/dev/null; then
  fail "no matching merged release PR"
fi

echo "Verified merged release PR for ${EXPECTED_HEAD_SHA}"
echo "head_sha=${EXPECTED_HEAD_SHA}" >> "${GITHUB_OUTPUT}"
