---
name: tvc-deployments
description: "Build, deploy, and maintain Turnkey Verifiable Cloud (TVC) apps with the `tvc` CLI: create apps and deployments, approve manifests, set the live deployment, check runtime status, tail debug logs, and consume the NDJSON output contract. Drives the CLI (not the HTTP API); use for scripting and agent-driven TVC workflows."
license: Apache-2.0
compatibility: "Requires the `tvc` CLI on PATH (install: `cargo install tvc`) and TVC API credentials, supplied via TVC_* environment variables or a prior `tvc login`. This skill drives the CLI, not the Turnkey HTTP API."
metadata:
  author: turnkey
  tags: "tvc verifiable-cloud cli deploy deployment enclave operator manifest quorum ndjson"
---

# TVC Deployments

> **This skill drives the `tvc` CLI, not the HTTP API.** Everything here runs `tvc <command> --message-format json` and reads NDJSON from stdout. Auth uses `TVC_*` env vars, not the `TURNKEY_API_*` set the HTTP-API skills use. Do not mix the two.

## When to use

Driving the `tvc` CLI: shipping a TVC deployment, maintaining an established TVC project, cutting traffic to a new version, checking runtime status, or scripting any of that in CI or an agent loop. First-time human onboarding (org setup, `tvc login`, dashboard key registration) belongs to the [TVC quickstart](https://docs.turnkey.com/features/verifiable-cloud/quickstart). Wallet, signing, and policy work over the HTTP API belongs to the other Turnkey skills.

**Out of scope, pointers only:** self-provisioned quorum keys and shares (`tvc keys *`, `deploy provisioning-details`, `provision`, `post-share`), YubiKey operators (`tvc yubikey *`, `operator create --kind yubikey`), and `tvc profile delete`. The commands are listed at the end of **[references/deploy-lifecycle.md](references/deploy-lifecycle.md)**; involve a human before using them.

`tvc skills save` installs this skill to `./.claude/skills` (`--global` for `~/.claude/skills`, `--dir <DIR>` elsewhere). Re-run it after upgrading `tvc`.

## Overview

From an empty directory to a live deployment:

```
app init → (operator create, only if the scaffold shows a sentinel) → edit → app create
→ deploy init → edit → deploy create → deploy approve → poll deploy get-status
# the first deployment goes live on approval; app set-live-deploy only switches versions
```

| What you want to do | Command(s) |
|---|---|
| Authenticate | `tvc login` (human), or `TVC_*` env vars (CI, agents) |
| Create a hosted operator (approver), only when `app.json` shows a sentinel | `tvc operator create` |
| Scaffold + create an app | `tvc app init` → edit → `tvc app create` |
| Scaffold + create a deployment | `tvc deploy init` (`--from-deployment <ID>` to clone) → edit → `tvc deploy create` |
| Approve a deployment's manifest | `tvc deploy approve` |
| Is it live? | `tvc deploy get-status` (poll; no `--wait`) |
| Switch traffic to a new deployment | `tvc app set-live-deploy` (only when `isTargeted == false`) |
| Inspect | `tvc app list`, `tvc app status`, `tvc deploy status` |
| Logs (debug-mode deployments only) | `tvc deploy debug-logs` |
| Roll back / clean up | `tvc deploy delete` → `tvc deploy restore`; `tvc app delete` (irreversible) |
| Version, skill install | `tvc version`, `tvc skills save` |

Command-by-command detail with outputs: **[references/deploy-lifecycle.md](references/deploy-lifecycle.md)**. Config-file shapes: **[references/config-files.md](references/config-files.md)**. Unattended pipeline: **[references/ci-recipe.md](references/ci-recipe.md)**.

## Authentication

Three env vars, **all three or none**:

```env
TVC_ORG_ID=            # organization UUID
TVC_API_KEY_PUBLIC=    # API key public component (hex)
TVC_API_KEY_PRIVATE=   # API key private component (hex)
TVC_API_BASE_URL=      # optional, defaults to https://api.turnkey.com
```

All three set → env auth, which takes precedence over any `tvc login` profile. None → the profile under `~/.config/turnkey/`. Some but not all → error naming the missing one. Empty counts as unset. `tvc login` needs a human; if neither env vars nor a profile exist, stop and ask.

**Env vars authenticate API calls but carry no operator signing key.** With no profile on the machine, `deploy approve` fails with `No active organization` unless you pass `--operator-seed-path <file>` or `--operator-seed` (env `TVC_OPERATOR_SEED`): the hex seed of a local operator whose key is in the app's manifest set. Hosted and YubiKey operators come from the profile. See **[references/ci-recipe.md](references/ci-recipe.md)**.

## Output contract

Add `--message-format json` to **every** command:

- Stdout is NDJSON, one object per line. JSON mode forces non-interactive: a command that would prompt fails fast with `missing_required_input` or `usage_error` instead of hanging.
- Success lines carry a `reason` (`app_created`, `deployment_created`, `manifest_approval_posted`, `live_deployment_set`, `deployment_runtime_status`, ...). One command, one terminal line, except `deploy debug-logs`, which streams `debug_log_line` events.
- Errors are one line: `{"reason":"command_error","code":"<code>","httpStatus"?:n,"message":"..."}`. Branch on `code`, never on `message`. Full taxonomy and recovery: **[references/error-reference.md](references/error-reference.md)**.
- Exit codes: `0` success, `1` runtime error, `2` usage error.

## Build and deploy

**Before running anything, confirm you have these four values.** The CLI cannot discover them; they come from the image you built and published:

| Value | Source |
|---|---|
| `pivotContainerImageUrl` | a `linux/amd64` OCI image in a registry TVC can pull, referenced **by digest** |
| `pivotPath` | path of the pivot binary *inside* that image |
| `expectedPivotDigest` | sha256 of that pivot binary, **not** the image digest; they are different fields |
| `pivotArgs` | the app's argument contract (often none). Plaintext in the signed manifest, visible to every approver and in boot proofs: never secrets |

If any are missing, **ask before step 2**: `app init` is local, but `app create` onward creates real resources. Never guess a digest; a wrong one passes every local check and fails only when the enclave refuses to start. Derivation: **[references/config-files.md](references/config-files.md)**.

1. `tvc app init --output app.json`. If `manifestSetParams.newOperators[0].publicKey` is a real key, your profile already has an operator: keep it, skip `operator create`. If it is `<FILL_IN_OPERATOR_PUBLIC_KEY>`, run `tvc operator create --message-format json` and paste its `compositePublicKey`.
2. Fill the remaining sentinels, then `tvc app create --config-file app.json --message-format json`. Save `appId` **and** `manifestSetOperatorIds` (the ids allowed to approve; nothing returns them later). Region, egress, and debug-mode eligibility are fixed here and cannot change later.
3. `tvc deploy init --output deploy.json` → set image, digest, path, ports → `tvc deploy create --config-file deploy.json --app-id <APP_ID> --message-format json`. Save `deploymentId`.
4. `tvc deploy approve --deploy-id <DEPLOY_ID> --operator-id <OPERATOR_ID> --dangerous-skip-interactive --message-format json`, with one of step 2's `manifestSetOperatorIds`. Operators are deduped on public key **plus operator name**, so an id from an earlier app is rejected even when the key matches; the error names the id to pass, so retry with it. A **hosted** operator's id is fixed in the profile, so that retry fails: create the app under the operator name the profile already uses. `quorumReached` is tri-state: `true` poll, `false` more approvals, `null` unknown, poll anyway.
5. Poll `tvc deploy get-status --deploy-id <DEPLOY_ID> --message-format json`. Live means `isTargeted == true` and `replicas.ready == replicas.desired`. The first deployment auto-targets on approval.
6. **Only if `isTargeted == false`** (an older deployment is live): `tvc app set-live-deploy --deploy-id <DEPLOY_ID> --message-format json`, then poll again.

Hostname: read `publicDomain` from `tvc app list --message-format json`. It is assigned at `app create`, independent of deployments, normally one hostname but possibly several comma-separated (use the first). **Never construct it or write one out from the app id**, not even as a guess. The patterns exist so you can recognize a value, not produce one: `app-<APP_ID>.app.turnkey.cloud` (`us-east-1`), `app-<APP_ID>.app-sin.turnkey.cloud` (`ap-southeast-1`), something else entirely in dev.

## Maintain an established project

- **Find apps:** `tvc app list --message-format json` (`-n <substring>`) → `apps_listed` with `liveDeploymentId` and `publicDomain`.
- **Enumerate deployments:** `tvc app status --app-id <APP_ID>` → `deployments[]` (`deploymentId`, `replicas{ready,desired}`) plus `targetedDeploymentId`. The only list; unapproved deployments may be absent, so save ids at create time.
- **Inspect one:** `deploy get-status` for readiness; `deploy status` for config and `manifestApprovals.quorumReached`.
- **Logs:** `tvc deploy debug-logs --deploy-id <ID> --tail-lines 200 --message-format json`; `--poll` streams and never exits, so bound it with a timeout. Requires the app created with `--dangerous-enable-debug-mode-deployments` (permanent, decided at `app create`) **and** the deployment created with `--dangerous-deploy-debug-mode`. A non-debug deployment can never produce logs; do not retry. If the app allows it, ship a new debug-mode deployment.
- **Ship a new version:** `deploy init --from-deployment <OLD_ID>` → edit → `deploy create` → `deploy approve` → `app set-live-deploy`.
- **Not coming up:** `isTargeted == true` but `replicas.ready` stuck at 0 past your timeout → **[references/failure-diagnosis.md](references/failure-diagnosis.md)**. Do not delete and retry the same config.

## Rules

1. **Always `--message-format json`** in scripts and agents: NDJSON out, no prompts, no hangs.
2. **`deploy approve` needs `--dangerous-skip-interactive` in JSON mode.** The approval is blind; only approve deployments whose inputs you control, and say so to the human when it matters.
3. **Liveness is a poll, set-live is conditional.** Poll `get-status` with backoff (5s) and a timeout. Right after approval, `replicas: null`, a 404 `not_found`, or a run of HTTP 500 `api_error`s all mean "not ready yet" (the 500 is a known backend gap for apps with no live deployment). Call `set-live-deploy` only when `isTargeted == false`; on an already-targeted deployment it fails with HTTP 400 "already set as the live deployment".
4. **Deletes need explicit human confirmation.** `deploy delete` is reversible with `deploy restore`; `app delete` removes the app and every deployment and is not.
5. **Branch on `code`, never on `message`.**
6. **Bound every streaming command.** `debug-logs --poll` runs until killed.

## Known gaps in the CLI (verified against tvc 0.17.1)

Inventing these fails with `usage_error`. Each names the `--help` check for your installed version; when check and text disagree, trust the check.

<!-- Maintainers: a CLI change that closes one of these deletes the bullet in the same PR and bumps the version above (see tvc/AGENTS.md). -->

- **No `tvc deploy list`** (check: `tvc deploy --help`). Use `tvc app status --app-id <APP_ID>` → `deployments[]`; see Maintain above.
- **No `tvc operator list`** and **no `tvc whoami`** (check: `tvc operator --help`, `tvc --help`). Save `operatorId` and `manifestSetOperatorIds` when they are printed; nothing returns them later.
- **No `--wait` / phase flag on `deploy get-status`** (check: `tvc deploy get-status --help`). Poll (Rule 3).
- **No `--version` flag** (check: `tvc --help`). Use `tvc version`.
- **`code: invalid_input` is never emitted.** It is in the taxonomy `tvc --help` prints, but the classifier does not produce it; do not branch on it.
- **`app status` and `deploy get-status` answer HTTP 500 for an app with no live deployment yet.** A backend gap, not an outage: keep polling after approve, and do not read a 500 from `app status` on a never-deployed app as a failure.

## Troubleshooting

First move per `code`: `missing_required_input` / `usage_error` fix the invocation; `unauthorized` check the three `TVC_*` vars or the profile; `not_found` check the organization the credentials are scoped to before the id; `approval_required` collect more approvals; `network_error` retry with backoff; `client_version_too_old` upgrade `tvc`, never retry. Per-code detail: **[references/error-reference.md](references/error-reference.md)**. Approved but never ready: **[references/failure-diagnosis.md](references/failure-diagnosis.md)**.

## Related

- [TVC quickstart](https://docs.turnkey.com/features/verifiable-cloud/quickstart): first-time human onboarding.
- `getting-started`: Turnkey HTTP API onboarding (separate `TURNKEY_API_*` credentials; not TVC).
