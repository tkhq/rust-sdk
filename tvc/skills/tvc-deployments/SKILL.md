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

> **This skill drives the `tvc` CLI, not the HTTP API.** Unlike the other Turnkey skills (which call `POST /public/v1/...` with `TURNKEY_API_*` credentials), everything here runs `tvc <command> --message-format json` and reads NDJSON from stdout. Auth uses `TVC_*` env vars, a different set. Do not mix the two.
>
> **Scope:** this skill covers driving the CLI programmatically, editing/maintaining an already-established TVC project, and carrying TVC knowledge into any repo (install once, invoke anywhere). First-time human onboarding (org setup, `tvc login`, dashboard key registration) is owned by the [TVC quickstart](https://docs.turnkey.com/features/verifiable-cloud/quickstart).

## When to use

Use this skill when driving the `tvc` CLI: building or shipping a TVC deployment, editing/maintaining an established TVC project, cutting traffic to a new version, checking runtime status, or scripting any of the above in CI or an agent loop. For first-time human onboarding (org setup, `tvc login`), point the user at the [TVC quickstart](https://docs.turnkey.com/features/verifiable-cloud/quickstart). For Turnkey wallet/signing/policy work over the HTTP API, use the other Turnkey skills instead, this one is CLI-only.

## Overview

Turnkey Verifiable Cloud (TVC) runs your container inside a verifiable enclave. The `tvc` CLI is the interface for provisioning operators, creating apps, and shipping deployments. The full lifecycle from an empty directory to a live deployment is:

```
app init → (operator create, only if the scaffold shows a sentinel) → (edit) → app create → deploy init → (edit) → deploy create → deploy approve → poll deploy get-status
# (the first deployment auto-goes-live on approval; app set-live-deploy is only for switching to a new version)
```

| What you want to do | Command(s) |
|---|---|
| Authenticate | `tvc login`, or set `TVC_*` env vars |
| Create a hosted operator (approver; only when the `app.json` scaffold lacks a prefilled operator key) | `tvc operator create` |
| Scaffold + create an app | `tvc app init` → edit → `tvc app create` |
| Scaffold + create a deployment | `tvc deploy init` → edit → `tvc deploy create` |
| Approve a deployment's manifest | `tvc deploy approve` |
| Switch traffic to a new deployment | `tvc app set-live-deploy` (the first deployment auto-targets on approval) |
| Check if a deployment is live | `tvc deploy get-status` (poll) |
| Inspect an app / list apps | `tvc app status`, `tvc app list` |
| Tail logs (debug-mode deploys) | `tvc deploy debug-logs` |
| Roll back / clean up | `tvc deploy restore`, `tvc deploy delete`, `tvc app delete` |

For the exact command-by-command happy path with expected outputs and the polling idiom, read **[references/deploy-lifecycle.md](references/deploy-lifecycle.md)**.

## Authentication

The CLI reads three credentials, **all required together**:

```env
TVC_ORG_ID=            # Turnkey organization UUID
TVC_API_KEY_PUBLIC=    # API key public component (hex)
TVC_API_KEY_PRIVATE=   # API key private component (hex)
TVC_API_BASE_URL=      # optional, defaults to https://api.turnkey.com
```

Resolution order:
1. **All three env vars set** → env auth is used (the CI / agent path). Empty strings count as unset.
2. **None set** → the CLI falls back to disk config at `~/.config/turnkey/`, which requires a prior `tvc login`.
3. **Partial** (some but not all three) → the CLI errors and names the missing variable. Set all three or none.

For interactive/local setup, `tvc login --org <alias> --api-base-url <url>` persists a profile to disk. Env vars always win over disk config when all three are present.

Credentials cannot be created non-interactively: `tvc login`'s key generation needs a human (TTY prompts plus manual dashboard registration). If neither env vars nor a profile are provisioned, stop and ask the user, do not retry `tvc login`.

## Output contract (read this before parsing anything)

Add `--message-format json` to **every** command in scripts and agents:

- Stdout becomes **NDJSON**: exactly one JSON object per line. Parse line by line.
- **JSON mode forces non-interactive.** A command that would otherwise prompt for a missing value instead fails fast with an error line (`missing_required_input` or `usage_error`) rather than hanging. Provide every required value up front.
- Each success line carries a `reason` naming the outcome (e.g. `app_created`, `deployment_created`, `manifest_approval_posted`, `live_deployment_set`, `deployment_runtime_status`). One command → one terminal outcome line (except streaming commands like `deploy debug-logs`, which emit `debug_log_line` events).
- Errors are a single line: `{ "reason": "command_error", "code": "<code>", "httpStatus"?: <n>, "message": "<full chain>" }`. Branch on `code`, not on `message` text. See **[references/error-reference.md](references/error-reference.md)** for the full taxonomy and recovery per code.

Exit codes: `0` success, `1` runtime error, `2` usage error.

## Core workflows

### Build and deploy (happy path)

Summarized here; full commands and outputs in **[references/deploy-lifecycle.md](references/deploy-lifecycle.md)**.

**Before you run anything, confirm you have these four values.** None can be obtained from the `tvc` CLI or discovered with `--help`; they come from the container image you built and published:

| Value | Source |
|---|---|
| `pivotContainerImageUrl` | a `linux/amd64` OCI image in a registry TVC can pull, referenced **by digest** |
| `pivotPath` | the path of the pivot binary *inside* that image |
| `expectedPivotDigest` | sha256 of that pivot binary, **not** the image digest, they are different fields |
| `pivotArgs` | the app's own argument contract (often none) |

If any are missing, **ask for them before step 2**. `app init` is local-only, but everything from `app create` onward (and `operator create`, when needed) creates real resources, so discovering the gap at step 3 leaves an app already provisioned. Never guess a digest: a plausible wrong value passes every local check and only fails once the enclave refuses to start. See **[references/config-files.md](references/config-files.md)** for how each is derived and what the image must satisfy.

1. `tvc app init --output app.json` → `app_config_created`. Check `manifestSetParams.newOperators[0].publicKey`: a **prefilled real key** means your profile already has an operator — keep it and skip `operator create`; a **`<FILL_IN_OPERATOR_PUBLIC_KEY>` sentinel** means run `tvc operator create --message-format json` → `operator_created` and paste its `compositePublicKey` into that field.
2. Fill the remaining `<FILL_IN_*>` sentinels → `tvc app create --config-file app.json --message-format json` → `app_created` (save `appId` **and** `manifestSetOperatorIds` — the operator ids allowed to approve this app's deployments; nothing returns them later)
3. `tvc deploy init --output deploy.json` → edit `deploy.json` (pivot image, ports) → `tvc deploy create --config-file deploy.json --app-id <APP_ID> --message-format json` → `deployment_created` (save `deploymentId`)
4. `tvc deploy approve --deploy-id <DEPLOY_ID> --operator-id <OPERATOR_ID> --dangerous-skip-interactive --message-format json` → `manifest_approval_posted`. `<OPERATOR_ID>` must be one of the app's `manifestSetOperatorIds` from step 2 — the id from `operator create` qualifies only if its key was wired into the manifest set. Its `quorumReached` field is a nullable boolean — tri-state: `true` = quorum met, `false` = more approvals needed, `null`/absent = unknown (not a failure; proceed to polling). A repeat approval by the same operator returns `manifest_approval_already_posted`, which is safe to treat as success.
5. Poll `tvc deploy get-status --deploy-id <DEPLOY_ID> --message-format json` until it returns state, then read `isTargeted`. The **first** deployment of an app auto-targets once approval quorum is reached, so `isTargeted` is already `true` and no set-live call is needed.
6. **Only if `isTargeted == false`** (switching traffic to a new deployment while an older one is live): `tvc app set-live-deploy --deploy-id <DEPLOY_ID> --message-format json`, then poll again. Live == `isTargeted == true` && `replicas.ready == replicas.desired` (see the polling rule below).

Once live, get the app's hostname from `tvc app list --message-format json`: each entry in `apps_listed` carries a `publicDomain` field. **Read it rather than constructing it.** The key is omitted when its value is empty, which is the only "absent" signal the API has; treat that as "no domain available right now" and poll again before assuming the app has none. Never construct the hostname: the pattern is environment-specific (`app-<APP_ID>.turnkey.cloud` in production, a different domain entirely in dev), so a guessed hostname is a dead end. Use the hostname to hit the app's health endpoint (e.g. `curl https://<publicDomain>/health`; the exact path and response are app-specific).

Config-file shapes and the scaffold-then-edit pattern are in **[references/config-files.md](references/config-files.md)**.

### Maintain an established project

- **Find your app:** `tvc app list --message-format json` (optionally `-n <name>`, a substring match) → `apps_listed`. `tvc app status --app-id <APP_ID>` → `app_status` for the app-wide view.
- **Enumerate an app's deployments:** `tvc app status --app-id <APP_ID>` → `app_status`, whose `deployments[]` array gives `deploymentId` and `replicas{ready,desired}` for each, with `targetedDeploymentId` naming the live one. This is the only way to list deployments; ids are returned bare (the CLI strips the API's `deploy` prefix).
- **Inspect a deployment:** `tvc deploy get-status --deploy-id <ID>` for one deployment's runtime readiness (`deployment_runtime_status`); `tvc deploy status --deploy-id <ID>` for config-level info like manifest id, QOS version, debug-mode, marked-for-deletion (`deployment_status`).
- **Debug a deployment** (must be deployed in debug mode): `tvc deploy debug-logs --deploy-id <ID> --tail-lines 200 --message-format json`. Add `--poll` to stream; it never self-terminates, so always bound it with a timeout wrapper. Debug mode is gated at **two** levels: the app must have been created with `--dangerous-enable-debug-mode-deployments` (permanent, decided at `app create`, cannot be added later) *and* the deployment created with `--dangerous-deploy-debug-mode`. An existing non-debug deployment can never produce logs — do not retry; if the app allows it, ship a new debug-mode deployment instead.
- **Ship a new version:** create a new deployment (`deploy create`), approve it, then `app set-live-deploy` to cut traffic over. Use `deploy init --from-deployment <OLD_ID>` to base the new config on an existing deployment.
- **Roll back / clean up:** `tvc deploy restore --deploy-id <ID>` undoes a `deploy delete`. `tvc deploy delete` and `tvc app delete` are destructive (see Rules).

## Rules

1. **Always pass `--message-format json`** for programmatic use. It gives you NDJSON and guarantees non-interactive behavior (no hangs).
2. **`deploy approve` requires `--dangerous-skip-interactive` in JSON/non-interactive mode.** There is no machine-readable manifest review yet, so non-interactive approval is unavoidably blind. Only approve deployments you created and whose inputs (`appId`, image, digest) you control. Surface this to the human when it matters.
3. **"Is it live?" is a manual poll, and set-live is conditional.** There is no `--wait` or lifecycle-phase field. After approve, poll `tvc deploy get-status`; a deployment is live when `isTargeted == true` and `replicas.ready == replicas.desired`. The **first** deployment of an app auto-targets on approval, so do not call `app set-live-deploy` for it, calling set-live on an already-targeted deployment fails with `api_error` (HTTP 400) "already set as the live deployment". Only call `set-live-deploy` when `isTargeted == false` (cutting traffic over to a new deployment). Right after approval, `get-status` may return `replicas: null`, a 404 `not_found` ("app status not found"), or a run of transient `api_error` HTTP 500s — all mean "not ready yet," not a real failure, keep polling. Back off between polls (e.g. 5s) with an overall timeout. If the timeout passes with `isTargeted == true` but `replicas.ready` pinned at 0, stop and escalate to a human with the last status and the exact config — the causes (bad digest, wrong health-check port, cluster-side issues) are invisible to the CLI and not agent-recoverable.
4. **Destructive commands have no undo except where noted.** `tvc app delete` removes the app **and all its deployments**. `tvc deploy delete` can be reversed with `tvc deploy restore`; `app delete` cannot. Confirm the exact `--app-id` / `--deploy-id` before running, and require explicit human confirmation for deletes.
5. **Branch on `code`, never on `message` text.** Error messages carry the full server chain and will change; the `code` is the stable contract.
6. **Bound every streaming command.** `deploy debug-logs --poll` runs until killed. Always wrap with a timeout or use `--tail-lines` for a one-shot read.

## Current limitations (as of this writing)

Do not assume these exist, they do not, and inventing them will fail:

- **No `tvc deploy list` subcommand**, but deployments *are* enumerable per app: `tvc app status --app-id <APP_ID>` returns a `deployments[]` array of `{deploymentId, replicas{ready,desired}, lastUpdated}` plus `targetedDeploymentId`. Use it to recover deployment ids you no longer have. Caveat: that array reflects runtime state, so a freshly created deployment that has not been approved yet may not appear in it. Still save `deploymentId` at create time. `app list` is the app-level view and carries only `liveDeploymentId`, which is by definition already approved.
- **No `tvc operator list`** and **no `tvc whoami`.** Save `operatorId` at creation time, and save `manifestSetOperatorIds` from `app create` output — they are the ids that can approve the app's deployments, and no later command returns them.
- **No `--wait` / phase flag.** Poll `deploy get-status` (Rule 3).
- **No `--version` flag.** Use the `tvc version` subcommand instead.
- **`code: invalid_input`** is in the taxonomy but not currently emitted by the classifier.

If a workflow needs one of these, script around it (save IDs, poll manually), do not call a nonexistent command.

## Troubleshooting

Errors classify into a `code`. Common ones and first move:

- **`missing_required_input`** — a required flag/value was absent in non-interactive mode. The message names it; supply the flag or its `TVC_*` env var.
- **`usage_error`** — bad flag or subcommand (clap parse error), exit 2. Re-check the command against this skill.
- **`unauthorized`** (HTTP 401/403) — credential or permission problem. Verify `TVC_ORG_ID`/`TVC_API_KEY_*` or your `tvc login` profile, and that the key has permission for the action.
- **`not_found`** (HTTP 404) — wrong `--app-id`/`--deploy-id`, or the resource has no state yet. Confirm the ID.
- **`approval_required`** — the manifest needs more approvals before it can proceed. Collect additional operator approvals.
- **`network_error`** — connect/timeout/DNS; the request never reached the server. Check `TVC_API_BASE_URL` and connectivity, then retry.
- **`api_error`** — other non-2xx from the API; read the `message` (it now carries the server's error body).
- **`client_version_too_old`** — the backend refuses `tvc` releases below its minimum version. Upgrade the binary (`cargo install tvc`); do not retry or touch the config.

Full per-code recovery is in **[references/error-reference.md](references/error-reference.md)**.

## Related Skills

- [TVC quickstart](https://docs.turnkey.com/features/verifiable-cloud/quickstart) — first-time human onboarding (org setup, `tvc login`, first app). Use this skill for CLI-driven, maintenance, and cross-repo work.
- `getting-started` — Turnkey HTTP API onboarding (separate `TURNKEY_API_*` credential model; not TVC).
