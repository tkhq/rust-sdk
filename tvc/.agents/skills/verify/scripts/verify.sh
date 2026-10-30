#!/bin/bash
set -euo pipefail

repo=$(cd "$(dirname "$0")/../../../../.." && pwd)
mode=${1:-smoke}
case "$mode" in
  smoke) test "$#" -le 1 ;;
  test) test "$#" -ge 2 && test "$#" -le 3 ;;
  *) printf 'Usage: %s smoke | test TARGET [FILTER]\n' "$0" >&2; exit 2 ;;
esac

mkdir -p "$repo/target/tvc-verify"
evidence=$(mktemp -d "$repo/target/tvc-verify/run.XXXXXX")
scratch=$(mktemp -d "${TMPDIR:-/tmp}/tvc-verify.XXXXXX")
trap 'result=$?; rm -rf -- "$scratch"; if test ! -e "$scratch"; then printf "Scratch removed: %s\nRun exit: %s\n" "$scratch" "$result" > "$evidence/cleanup.txt"; else exit 1; fi' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
printf 'Evidence: %s\n' "$evidence"
mkdir -p "$scratch/home" "$scratch/work"
cd "$repo"
git rev-parse HEAD > "$evidence/revision.txt"
git status --short > "$evidence/worktree.txt"
printf 'Child environment: HOME=scratch/home PATH=/usr/bin:/bin LANG=C TERM=dumb\nNo inherited TVC_*, credentials, proxy settings, or RUST_LOG.\n' > "$evidence/environment.txt"

if test "$mode" = test; then
  target=$2
  filter=${3:-}
  cargo test --locked --offline --target-dir "$repo/target" -p tvc --test "$target" --no-run --message-format json > "$evidence/build.jsonl" 2> "$evidence/build.log"
  binary=$(python3 - "$evidence/build.jsonl" "$target" <<'PY'
import json, sys
artifacts = [json.loads(line) for line in open(sys.argv[1])]
matches = [a['executable'] for a in artifacts if a.get('reason') == 'compiler-artifact' and a['target']['name'] == sys.argv[2] and a.get('executable')]
assert len(matches) == 1, matches
print(matches[0])
PY
)
  command=("$binary")
  if test -n "$filter"; then command+=("$filter"); fi
  command+=(--nocapture)
else
  cargo build --locked --offline --target-dir "$repo/target" -p tvc > "$evidence/build.log" 2>&1
  binary="$repo/target/debug/tvc"
fi
shasum -a 256 "$binary" > "$evidence/binary.sha256"
cd "$repo/tvc"

if test "$mode" = test; then
  printf '%q ' "${command[@]}" > "$evidence/command.txt"
  printf '\nWorking directory: %s\n' "$PWD" >> "$evidence/command.txt"
  result=0
  env -i HOME="$scratch/home" PATH=/usr/bin:/bin LANG=C TERM=dumb "${command[@]}" > "$evidence/stdout.txt" 2> "$evidence/stderr.txt" || result=$?
  printf '%s\n' "$result" > "$evidence/exit-code.txt"
  cat "$evidence/stdout.txt" "$evidence/stderr.txt"
  test "$result" -eq 0
  python3 - "$evidence/stdout.txt" <<'PY' > "$evidence/validation.txt" 2> "$evidence/validation.stderr.txt"
import re, sys
text = open(sys.argv[1]).read()
result = re.search(r'test result: ok\. (\d+) passed;', text)
assert result and int(result[1]) > 0, 'No tests passed; check target and filter'
PY
  exit 0
fi

cd "$scratch/work"
for step in doctor version app deploy quorum skills skills-repeat approve app-overwrite; do
  expected=0
  case "$step" in
    doctor) command=("$binary" --help) ;;
    version) command=("$binary" --message-format json version) ;;
    app|app-overwrite) command=("$binary" --message-format json app init --output app.json)
      if test "$step" = app-overwrite; then expected=1; fi ;;
    deploy) command=("$binary" --message-format json deploy init --output deploy.json) ;;
    quorum) command=("$binary" --message-format json keys init-local-quorum-key --output quorum.json) ;;
    skills|skills-repeat) command=("$binary" --message-format json skills save --dir skills) ;;
    approve) command=("$binary" --message-format json deploy approve --manifest "$repo/tvc/fixtures/manifest.json" --operator-seed-path "$repo/tvc/fixtures/seed.hex" --dangerous-skip-interactive --skip-post --approval-out approval.json) ;;
  esac
  printf '%q ' "${command[@]}" > "$evidence/$step.command.txt"
  printf '\nWorking directory: %s\n' "$PWD" >> "$evidence/$step.command.txt"
  result=0
  env -i HOME="$scratch/home" PATH=/usr/bin:/bin LANG=C TERM=dumb "${command[@]}" > "$evidence/$step.stdout.txt" 2> "$evidence/$step.stderr.txt" || result=$?
  printf '%s\n' "$result" > "$evidence/$step.exit-code.txt"
  test "$result" -eq "$expected"
  if test "$step" = doctor; then test ! -e "$scratch/home/.config"; fi
  if test "$step" = app; then cp app.json "$evidence/app-before.json"; fi
done
cp -R "$scratch/work" "$evidence/artifacts"
cp "$scratch/home/.config/turnkey/tvc.config.toml" "$evidence/tvc.config.toml"
python3 - "$evidence" "$repo/tvc/fixtures/manifest.json" "$scratch/home" <<'PY' > "$evidence/assertions.txt" 2> "$evidence/assertions.stderr.txt"
import json, pathlib, sys
evidence, manifest_path, child_home = map(pathlib.Path, sys.argv[1:])
messages = {}
for step in ('version', 'app', 'deploy', 'quorum', 'skills', 'skills-repeat', 'approve', 'app-overwrite'):
    lines = (evidence / f'{step}.stdout.txt').read_text().splitlines()
    assert len(lines) == 1, (step, lines)
    messages[step] = json.loads(lines[0])
assert messages['version']['reason'] == 'version'
assert isinstance(messages['version']['version'], str)
assert messages['app']['reason'] == 'app_config_created'
assert messages['app']['template'] is True
assert messages['deploy']['reason'] == 'deployment_config_created'
assert messages['quorum']['reason'] == 'quorum_key_config_created'
assert messages['skills']['reason'] == 'skills_saved'
assert messages['skills']['filesWritten'] > 0
assert messages['skills-repeat']['filesWritten'] == 0
assert messages['skills-repeat']['filesUnchanged'] == messages['skills']['filesWritten']
exported = evidence / 'artifacts/skills/tvc-deployments/SKILL.md'
assert exported.read_text().startswith('---\nname: tvc-deployments\n')
assert (evidence / 'app-before.json').read_bytes() == (evidence / 'artifacts/app.json').read_bytes()
assert messages['app-overwrite']['reason'] == 'command_error'
for name in ('app', 'deploy', 'quorum'):
    assert isinstance(json.loads((evidence / f'artifacts/{name}.json').read_text()), dict)
approval = json.loads((evidence / 'artifacts/approval.json').read_text())
assert messages['approve']['reason'] == 'manifest_approval_generated'
assert messages['approve']['writtenTo'] == 'approval.json'
assert approval['member'] in json.loads(manifest_path.read_text())['manifestSet']['members']
assert len(bytes.fromhex(approval['signature'])) > 0
assert not (child_home / '.config/turnkey/orgs').exists(), 'Unexpected saved credentials'
print('PASS: JSON outcomes, persisted templates, overwrite refusal, idempotent skill export, fixture approval, no saved API credentials')
PY
cat "$evidence/assertions.txt"
