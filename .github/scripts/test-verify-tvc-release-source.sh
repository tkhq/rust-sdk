#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
test_dir=$(mktemp -d)
trap 'rm -rf "${test_dir}"' EXIT

cat > "${test_dir}/gh" <<'MOCK_GH'
#!/usr/bin/env bash
set -euo pipefail
case "${2:-}" in
  repos/tkhq/rust-sdk/actions/workflows/release-plz.yml)
    echo 179313667
    ;;
  repos/tkhq/rust-sdk/actions/runs/123)
    cat "${TEST_RUN_JSON}"
    ;;
  'repos/tkhq/rust-sdk/actions/runs/123/jobs?filter=latest&per_page=100')
    cat "${TEST_JOBS_JSON}"
    ;;
  'repos/tkhq/rust-sdk/commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/pulls?per_page=100')
    cat "${TEST_PRS_JSON}"
    ;;
  *)
    echo "Unexpected gh api request: $*" >&2
    exit 2
    ;;
esac
MOCK_GH
chmod +x "${test_dir}/gh"

cat > "${test_dir}/run.json" <<'JSON'
{"workflow_id":179313667,"name":"Release-plz","path":".github/workflows/release-plz.yml","event":"pull_request","conclusion":"success","head_repository":{"full_name":"tkhq/rust-sdk"},"head_sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","head_branch":"release-plz-test"}
JSON
cat > "${test_dir}/jobs.json" <<'JSON'
{"jobs":[{"name":"Release-plz release","conclusion":"success"}]}
JSON
cat > "${test_dir}/prs.json" <<'JSON'
[{"merged_at":"2026-11-19T18:18:46Z","base":{"ref":"main"},"head":{"repo":{"full_name":"tkhq/rust-sdk"},"sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","ref":"release-plz-test"},"labels":[{"name":"release"}]}]
JSON

run_case() {
  local label=$1 expected=$2
  local run_json=${3:-${test_dir}/run.json}
  local jobs_json=${4:-${test_dir}/jobs.json}
  local prs_json=${5:-${test_dir}/prs.json}
  local expected_text=${6:-}
  : > "${test_dir}/output"
  if PATH="${test_dir}:${PATH}" GITHUB_REPOSITORY=tkhq/rust-sdk \
    WORKFLOW_RUN_ID=123 EXPECTED_HEAD_SHA=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
    GITHUB_OUTPUT="${test_dir}/output" TEST_RUN_JSON="${run_json}" \
    TEST_JOBS_JSON="${jobs_json}" TEST_PRS_JSON="${prs_json}" \
    bash "${script_dir}/verify-tvc-release-source.sh" > "${test_dir}/log" 2>&1; then
    actual=success
  else
    actual=failure
  fi
  if [[ "${actual}" != "${expected}" ]]; then
    echo "${label}: expected ${expected}, got ${actual}" >&2
    cat "${test_dir}/log" >&2
    exit 1
  fi
  if [[ -n "${expected_text}" ]] && ! grep -Fq "${expected_text}" "${test_dir}/log"; then
    echo "${label}: missing expected diagnostic: ${expected_text}" >&2
    cat "${test_dir}/log" >&2
    exit 1
  fi
  echo "${label}: ${actual}"
}

run_case "merged release" success "${test_dir}/run.json" "${test_dir}/jobs.json" \
  "${test_dir}/prs.json" "Verified merged release PR"
jq '.head_repository.full_name = "outsider/rust-sdk"' "${test_dir}/run.json" > "${test_dir}/fork.json"
run_case "fork workflow" failure "${test_dir}/fork.json" "${test_dir}/jobs.json" \
  "${test_dir}/prs.json" "triggering run is not the repository release workflow"
jq '.path = ".github/workflows/other.yml"' "${test_dir}/run.json" > "${test_dir}/other-workflow.json"
run_case "wrong workflow" failure "${test_dir}/other-workflow.json" "${test_dir}/jobs.json" \
  "${test_dir}/prs.json" "triggering run is not the repository release workflow"
jq '.jobs[0].conclusion = "skipped"' "${test_dir}/jobs.json" > "${test_dir}/skipped-job.json"
run_case "skipped release job" failure "${test_dir}/run.json" "${test_dir}/skipped-job.json" \
  "${test_dir}/prs.json" "release job did not succeed exactly once"
jq '.[0].merged_at = null' "${test_dir}/prs.json" > "${test_dir}/unmerged.json"
run_case "unmerged PR" failure "${test_dir}/run.json" "${test_dir}/jobs.json" \
  "${test_dir}/unmerged.json" "no matching merged release PR"
jq '.[0].labels = []' "${test_dir}/prs.json" > "${test_dir}/no-label.json"
run_case "missing release label" failure "${test_dir}/run.json" "${test_dir}/jobs.json" \
  "${test_dir}/no-label.json" "no matching merged release PR"
