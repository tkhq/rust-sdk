# tvc deploy delete

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy delete command.*

## Purpose (informative)

`tvc deploy delete` marks one deployment for deletion.
It submits the `ACTIVITY_TYPE_DELETE_TVC_DEPLOYMENT` activity to the Turnkey API (tvc/src/commands/deploy/delete.rs:39-43, client/src/generated/client.rs:4294-4295).
The delete is soft: `tvc deploy restore` reverses it (tvc/src/commands/deploy/restore.rs:40-44).
Run it to retire a deployment.

## Acceptance scenario (normative)

Given: a logged-in profile whose active org is `7f0c2d4e-5a6b-4c8d-9e0f-1a2b3c4d5e6f`, and a deployment `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` in that org.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | stdout shows `Deployment delete accepted; deployment is marked for deletion.` followed by the `Deployment ID`, `Activity ID`, and `Activity Status` lines. Exit code 0. |
| 2 | Run `tvc deploy restore --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | Exit code 0. The deployment is active again, which confirms the soft delete. |

All steps pass in one run from a clean start with one logged-in profile and one live deployment.

## Inputs (normative)

| Input | Flag | Env | Config source | Default | Prompted |
|---|---|---|---|---|---|
| deployment id | `--deploy-id <DEPLOY_ID>` (no short) | `TVC_DEPLOY_ID` | none | none (required) | never |
| org id (auth) | none | `TVC_ORG_ID` | the active org's `id` in the tvc config | none | never |
| API key (auth) | none | `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE` | the active org's stored API key file | none | never |
| API base URL | none | `TVC_API_BASE_URL` | the active org's `api_base_url` | `https://api.turnkey.com` | never |

`--deploy-id` MUST parse as a UUID (delete.rs:22).
Its resolution follows Part 00: flag first, then `TVC_DEPLOY_ID`.
No config file source, built-in default, or prompt exists for it (delete.rs:21).

Auth resolution deviates from per-value resolution by design.
The command MUST resolve the three required auth env vars as one group (tvc/src/client.rs:192-242).
When `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` are all set, they MUST override the tvc config.
When none are set, the command MUST use the active org's stored credentials (tvc/src/client.rs:103-125).
When only some are set, the command MUST fail and name the missing variables (tvc/src/client.rs:226-234).
The command MUST NOT merge env auth values with tvc config auth values.
`TVC_API_BASE_URL` is optional under env auth and defaults to `https://api.turnkey.com` (tvc/src/client.rs:24, 197-198).
An empty env var counts as unset (tvc/src/client.rs:181-183).

The inherited `--non-interactive` flag has no observable effect: `run` never reads the context (delete.rs:27).

## Interactive behavior (normative)

The command MUST NOT prompt in any mode: `run` binds the context as `_ctx` and never reads it (delete.rs:27).
Interactive mode and non-interactive mode MUST produce identical output for identical inputs.
A missing deployment id MUST fail as a clap usage error in both modes (tvc/tests/deploy_delete.rs:26-35); no prompt fallback exists.
The command offers no confirmation step before the delete (Gap 1).

## Outputs (normative)

In human mode stdout MUST carry one blank line, then this block (delete.rs:73-88):

```
Deployment delete accepted; deployment is marked for deletion.

Deployment ID: <deployment id>
Activity ID: <activity id>
Activity Status: ACTIVITY_STATUS_COMPLETED
```

In JSON mode stdout MUST carry exactly one NDJSON object (tvc/src/output.rs:97-99).
The object MUST carry `reason` = `deployment_deleted` (tvc/src/outcome.rs:30, 49) plus the fields `deploymentId`, `activityId`, and `activityStatus` (delete.rs:52-59).
`activityStatus` MUST serialize as the stable proto status name (client/src/generated/immutable.activity.v1.rs:5558-5575).
On success the value is always `ACTIVITY_STATUS_COMPLETED` because the client errors on every other terminal status (client/src/lib.rs:360-386; Gap 6).
The success message names no undo command (Gap 5).

## Side effects (normative)

- Dispatch MUST load the tvc config before the command runs and MUST create the file when absent (INV-G4; tvc/src/cli.rs:215-240).
- When env auth is absent, the command MUST read the active org's stored API key file (tvc/src/client.rs:115-117).
- The command MUST submit exactly one `ACTIVITY_TYPE_DELETE_TVC_DEPLOYMENT` activity (client/src/generated/client.rs:4287-4326). The client polls while the activity reports the pending status (client/src/lib.rs:362-371).
- The command MUST NOT write any other file and MUST NOT touch a YubiKey.

## Failure modes (normative)

In JSON mode every failure MUST surface per INV-G3 with `reason` = `command_error` and the listed `code` (tvc/src/output.rs:315, 335-341, 344-352).
In human mode the failure MUST render on stderr with an `error:` prefix (tvc/src/output.rs:161-182).

| Failure | `code` | Exit code | Evidence |
|---|---|---|---|
| `--deploy-id` absent and `TVC_DEPLOY_ID` unset | `usage_error` | 2 | tvc/tests/deploy_delete.rs:26-35 |
| Deployment id value fails UUID parsing | `usage_error` | 2 | delete.rs:22 (clap value parser) |
| Only some auth env vars set | `command_error` | 1 | tvc/src/client.rs:226-234 |
| No active org in the tvc config | `command_error` | 1 | tvc/src/client.rs:104-106 |
| The active org's API key file is missing | `command_error` | 1 | tvc/src/client.rs:115-117 |
| HTTP 401 or 403 | `unauthorized`, `httpStatus` set | 1 | tvc/src/errors.rs:214-226 |
| HTTP 404 | `not_found`, `httpStatus` set | 1 | tvc/src/errors.rs:214-226 |
| Other non-success HTTP status | `api_error`, `httpStatus` set | 1 | tvc/src/errors.rs:214-226 |
| Connect, timeout, or DNS failure | `network_error` | 1 | tvc/src/errors.rs:229-235 |
| Activity needs consensus | `approval_required` | 1 | client/src/lib.rs:375-377, tvc/src/errors.rs:236-238 |
| Activity failed, or an unexpected terminal status | `api_error` | 1 | client/src/lib.rs:372-384, tvc/src/errors.rs:247-258 |

Under a quorum policy the API records the delete activity and waits for approvals.
The command still exits 1 and reports `approval_required` (client/src/lib.rs:375-377).

## Test vectors (normative)

Vector comparison excludes the Part 00 nondeterministic fields.
For this command those are `activityId` and the `Activity ID` line.
V-10 represents the HTTP status classification family; the `not_found` and other-status rows follow the same classifier (tvc/src/errors.rs:214-226).
The failed-activity row also flows through that classifier as `api_error` (tvc/src/errors.rs:247-258).

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Active org `7f0c2d4e-5a6b-4c8d-9e0f-1a2b3c4d5e6f` with valid key files; deployment `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` exists | `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | stdout: one blank line, `Deployment delete accepted; deployment is marked for deletion.`, one blank line, `Deployment ID: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, `Activity ID: <excluded>`, `Activity Status: ACTIVITY_STATUS_COMPLETED`; exit code 0 (delete.rs:73-88). |
| V-2 | Same as V-1 | `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | stdout: one NDJSON object `{"reason":"deployment_deleted","deploymentId":"6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b","activityId":"<excluded>","activityStatus":"ACTIVITY_STATUS_COMPLETED"}`; exit code 0 (tvc/src/outcome.rs:49, delete.rs:52-59). |
| V-3 | Same as V-1 plus `TVC_DEPLOY_ID=6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` exported | `tvc deploy delete` | Same as V-1: the env var satisfies the required argument (delete.rs:21). Pins Gap 2's current behavior. |
| V-4 | `TVC_DEPLOY_ID` unset | `tvc deploy delete` | stderr contains `the following required arguments were not provided` and `--deploy-id <DEPLOY_ID>`; exit code 2 (tvc/tests/deploy_delete.rs:26-35). |
| V-5 | `TVC_DEPLOY_ID` unset | `tvc deploy delete --message-format json` | stdout: one JSON object with `reason` = `command_error`, `code` = `usage_error`, `message` carrying clap's text; exit code 2 (tvc/src/cli.rs:160-176, tvc/src/output.rs:344-352). |
| V-6 | none | `tvc deploy delete --deploy-id not-a-uuid` | clap invalid-value error on stderr; exit code 2 (delete.rs:22). |
| V-7 | Only `TVC_ORG_ID=7f0c2d4e-5a6b-4c8d-9e0f-1a2b3c4d5e6f` exported | `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | Error message: `partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE. Set all three (TVC_ORG_ID, TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE) env vars or none.`; `code` = `command_error`; exit code 1 (tvc/src/client.rs:226-234). |
| V-8 | Fresh HOME (dispatch writes a default tvc config with no active org); no auth env vars | `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | Error message: ``No active organization. Run `tvc login` first.``; `code` = `command_error`; exit code 1 (tvc/src/client.rs:104-106). |
| V-9 | Active org `acme` whose `api_key_path` points to a missing file; no auth env vars | `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | Error message: ``No API key found for org 'acme'. Run `tvc login` first.``; `code` = `command_error`; exit code 1 (tvc/src/client.rs:115-117, tvc/src/config/turnkey/api_key.rs:32-35). |
| V-10 | Valid auth; the API answers HTTP 401 | `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | `code` = `unauthorized`, `httpStatus` = 401; exit code 1 (tvc/src/errors.rs:214-226). |
| V-11 | Valid auth; the org enforces a quorum policy on the delete | `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | `code` = `approval_required`; exit code 1; the API keeps the activity pending approvals (client/src/lib.rs:375-377, tvc/src/errors.rs:236-238). |
| V-12 | Full env auth plus `TVC_API_BASE_URL=https://127.0.0.1:1` | `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | `code` = `network_error`; exit code 1 (tvc/src/errors.rs:229-235). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT prompt in any mode. | `run` binds the context as `_ctx` and contains no prompt call (delete.rs:27-50); INV-G1 covers JSON mode. |
| INV-2 | Every success outcome MUST serialize with `reason` = `deployment_deleted`. | serde internal tagging derives the reason from the `Outcome` variant name; a duplicate variant fails to compile (tvc/src/outcome.rs:30, 49). Behavioral check: tvc/src/outcome.rs:123-137. |
| INV-3 | A run that exits 0 MUST report `activityStatus` = `ACTIVITY_STATUS_COMPLETED`. | `process_activity` returns only on the completed status and errors on every other terminal status (client/src/lib.rs:360-386). |
| INV-4 | The command MUST NOT write files beyond the dispatch-level tvc config creation (INV-G4). | `run` performs only the client build and one API call; no file write exists in the module (delete.rs:27-50). |

## Gaps (informative)

1. **[consistency] A destructive command with no confirmation, while `profile delete` confirms; the purpose-built shared helper is dead code**.
   `deploy delete` submits immediately, with no prompt and no `--yes` flag (delete.rs:27-50).
   `profile delete` requires interactive confirmation through `confirm_or_bail`, and requires `--org` plus `--yes` in non-interactive mode (tvc/src/commands/login.rs:129-136, 147-209).
   `tvc/src/commands/confirmation.rs` (`confirm_yes_no`, `confirm_typed`) has zero production callers.
   The helpers landed in the same commit (c2b9030e) as `deploy delete` and `app delete`, and their tests prompt `Type app id` (confirmation.rs:129-147).
   The commit evidently planned confirmation for the delete commands and never wired it.
   `app delete` shares the gap and deletes the app plus all its deployments (tvc/src/commands/app/delete.rs:27-50).
   The soft delete plus the `deploy restore` round trip is the only mitigation.

2. **[bug?] With `TVC_DEPLOY_ID` exported, bare `tvc deploy delete` deletes with no flag, no prompt, and no output before it acts**.
   The env var satisfies the required argument (delete.rs:21).
   Seven neutral siblings read the same env var: status, get-status, provision, provisioning-details, debug-logs, approve, and restore (status.rs:29, provision.rs:34).
   Exporting it for a status or provision workflow therefore arms an argument-free destructive command.
   This compounds gap 1 with the shared env var. V-3 pins the current behavior.

3. **[capability] The CLI reserves no `--yes` escape hatch, so a later fix for gap 1 becomes a breaking change**.
   Scripts that invoke `tvc deploy delete` today rely on zero confirmation.
   A new prompt without a pre-existing `--yes` (the `profile delete` shape, login.rs:56-58) would break them.
   Adding `--yes` now, as a no-op until confirmation exists, makes the fix non-breaking.

4. **[consistency] `deploy delete` is the only deployment-id command in the deploy group without a short `-d`; this breaks delete and restore round-trip symmetry**.
   `restore`, `status`, `get-status`, `provision`, `provisioning-details`, `debug-logs`, and `approve` all take `-d` (restore.rs:22, status.rs:29, provision.rs:34, approve.rs:71).
   `delete` is long-only (delete.rs:21), so `tvc deploy delete -d` fails while `tvc deploy restore -d` works.
   `app delete` keeps `--app-id` long-only too (tvc/src/commands/app/delete.rs:21), so the friction looks deliberate.
   No help text documents it (no `long_about`, delete.rs:18).

5. **[capability] No pre-delete context and no restore pointer**.
   The command never fetches the deployment before it deletes, although `fetch_tvc_deployment` exists (tvc/src/client.rs:83-100).
   An interactive user therefore sees nothing about the target: its app, status, or liveness.
   The success message (delete.rs:78) also omits `deploy restore` as the undo.
   Contrast `profile delete`, whose pre-deletion warning block names exactly what the deletion destroys (login.rs:153-204).

6. **[docs] `activityStatus` in the JSON output can only ever be `ACTIVITY_STATUS_COMPLETED`**.
   `process_activity` returns only on `Completed` and errors on every other terminal status (client/src/lib.rs:360-386).
   The field (delete.rs:56-58) therefore implies variability that cannot occur.
   `deploy restore` and `app delete` share the shape.
   It is harmless; still, JSON consumers can build dead branches on it.
