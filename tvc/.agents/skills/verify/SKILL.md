---
name: verify
description: Verify the rust-sdk TVC CLI through real commands, isolated local artifacts, HTTP-boundary integration tests, and PTY prompts. Use after changing tvc or tracing how app, deployment, operator, and quorum-key commands work together.
---

# Verify TVC

Read the [feature map](features/README.md) and the feature touched by the change.
Use the existing [deployment guide](../../../skills/tvc-deployments/SKILL.md)
for operational semantics. This developer skill lives outside the bundled
`tvc/skills/` tree and is not exported by `tvc skills save`.

## Launch

From the workspace root containing `Cargo.toml` and `tvc/`, run:

```sh
tvc/.agents/skills/verify/scripts/verify.sh smoke
```

The helper builds `tvc` with `cargo build --locked --offline -p tvc`, then drives
the resulting binary. Rust comes from `rust-toolchain.toml`. Linux needs the
PC/SC development library used by `qos_client`'s smartcard feature. If Cargo
reports an uncached dependency, run `cargo fetch --locked` with network access
and retry. Report build failures before attempting a journey.

There is no server, port, database, or migration to start. Ready means the
current checkout builds and the binary's `--help` succeeds. The helper creates
a disposable working directory and child-process `HOME`, removes inherited
environment inputs, and retains evidence under `target/tvc-verify/`.
Do not redirect the parent shell's `HOME` or drive the user's saved profiles.
Each run owns its scratch directory, so concurrent runs do not share runtime
state. Cargo may serialize builds in this worktree's target directory.

## Doctor

Run `target/debug/tvc --help` before diagnosing runtime failures. Help exits
before configuration loading and is read-only. `tvc version` is not a read-only
doctor on an empty home because it creates default configuration.

For isolated identity and output evidence, the smoke helper also captures
`version`, the binary SHA-256, Git revision, and working-tree status. Rebuild
after source changes. Do not use an installed `tvc` as proof of this checkout.

## Drive

Use [verify.sh](scripts/verify.sh) for the offline journey and existing subprocess
and Unix PTY integration tests:

```sh
tvc/.agents/skills/verify/scripts/verify.sh smoke
tvc/.agents/skills/verify/scripts/verify.sh test pty approve_walks_all_sections_with_yeses
tvc/.agents/skills/verify/scripts/verify.sh test deploy_approve
```

`smoke` generates app, deploy, and quorum templates, refuses an app-template
overwrite, exports the bundled skill twice, and signs a fixture manifest into
an approval file. `test TARGET [FILTER]` compiles an existing integration-test
target and runs its executable under the same isolated environment. A filter
that matches zero tests is a failure. Omit the filter to run the whole target.

JSON mode implies non-interactive operation. Use the PTY target to verify the
review prompts; the smoke path deliberately uses `--dangerous-skip-interactive`
with repository fixtures. That flag is not approval to bypass a real review.

For connected journeys, use a dedicated test organization and the exact paths
in the relevant feature file. Do not promote fixture results to live coverage.
Preserve the scope of authorization for creation, traffic changes, deletion,
key export, and device writes.

## Evidence

The helper prints `Evidence: <absolute directory>` before starting work. Keep
that directory in the report, including on failure. It contains the action,
working directory, sanitized environment contract, stdout, stderr, exit code,
build log, and cleanup result. `exit-code.txt` records the test executable's
status; `cleanup.txt` records the whole helper's status, including failed
assertions. Smoke evidence also retains generated files and
assertions checking the reported approval path and stored template contents.

Exercise real CLI entry points. Capture both the action and its result. For a
mutation, inspect files or perform a second read through the CLI. A help page,
unit test, or successful API activity alone does not prove runtime readiness.
The smoke check proves an approval artifact was produced for a fixture member;
it does not independently validate the cryptographic signature or attestation.

Existing tests mock only the HTTP and hardware boundaries already isolated by
the app. Label their results as integration coverage. Live provisioning requires
fresh attestation; synthetic or expired documents cannot prove that path.
`--dry-run` may fetch a deployment. `--skip-post` still fetches with `--deploy-id`.
The smoke path uses a local manifest, explicit fixture seed, and `--skip-post`;
its evidence checks local writes, including the absence of saved API credentials.
It does not capture network traffic, so do not claim an observed no-network proof.

## Cleanup

The helper traps exit and removes only its own scratch directory, including
temporary keys and configuration, after success or failure. It records removal
in `cleanup.txt`. Evidence stays outside scratch. After the run, require
`cleanup.txt` and confirm the reported evidence directory still exists.

Connected journeys must record created IDs and delete only resources created
by that run when that cleanup is authorized. Capture their final state first.
For `debug-logs --poll`, record the child PID, bound the observation window,
and terminate that PID. Never kill by process name.

## Helpers

`scripts/verify.sh smoke` and `scripts/verify.sh test TARGET [FILTER]` are the
supported invocations. Bash, Python 3, Git, Cargo, and `shasum` are required.
Read [features/README.md](features/README.md) before selecting test targets.
Use `/maintain-verification-skill` when commands or journeys change.
