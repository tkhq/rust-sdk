# tvc deploy restore

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy restore command.*

## Purpose (informative)

`tvc deploy restore` removes the deletion mark that `tvc deploy delete` placed on a deployment.
It submits one `ACTIVITY_TYPE_RESTORE_TVC_DEPLOYMENT` activity (restore.rs:31, client/src/generated/client.rs:4338).
Run it to cancel a pending deletion before the backend reaps the deployment.
The command is the exact inverse of `deploy delete` (tvc/src/commands/deploy/delete.rs).

## Acceptance scenario (normative)

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc deploy delete --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | The `deployment_deleted` outcome reports the deletion mark (tvc/src/commands/deploy/delete.rs:78). |
| 2 | Run `tvc deploy status -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | The report shows `Marked for deletion: yes` (tvc/src/commands/deploy/status.rs:230). |
| 3 | Run `tvc deploy restore -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | stdout shows the success block with `Deployment ID: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` and `Activity Status: ACTIVITY_STATUS_COMPLETED`; exit code 0 (restore.rs:76). |
| 4 | Run `tvc deploy status -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | The report shows `Marked for deletion: no` (tvc/src/commands/deploy/status.rs:235). |

Pass criterion: all steps pass in one run against one logged-in org that owns deployment `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`.

## Inputs (normative)

| Input | Flag | Env | Config source | Default | Prompted |
|---|---|---|---|---|---|
| deployment id | `-d, --deploy-id <DEPLOY_ID>` (UUID) | `TVC_DEPLOY_ID` | none | none (required) | no |
| auth (org id + API key) | none | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` (all or none) | active org in the tvc config (`tvc login`) | none | no |
| API base URL | none | `TVC_API_BASE_URL` | per org `api_base_url` | `https://api.turnkey.com` | no |

The command MUST resolve `deploy_id` from the flag first, then from `TVC_DEPLOY_ID` (restore.rs:22).
This deviates from the Part 00 resolution order: `deploy_id` has no config file source and no built-in default.
Every deploy command reads the same `TVC_DEPLOY_ID` variable, so one exported value feeds all of them (tvc/src/commands/deploy/delete.rs:21).

Authentication MUST follow the shared `build_client` rule (tvc/src/client.rs:48).
A complete env trio MUST win over the stored active org credentials (tvc/src/client.rs:192).
A partial env trio MUST fail with the names of the missing variables (tvc/src/client.rs:226).
With no auth env set, the command MUST use the active org credentials that `tvc login` stored (tvc/src/client.rs:103).

## Interactive behavior (normative)

The command never prompts: `run` binds the context as `_ctx` (restore.rs:28).
Interactive mode and non-interactive mode MUST behave identically.
JSON mode adds no command-side change (INV-G1 already forces non-interactive mode).
`deploy_id` is a hard clap requirement in every mode (restore.rs:22).
An invocation without `deploy_id` MUST fail as a parse error with exit code 2 (tvc/tests/deploy_restore.rs:26).
The command MUST NOT emit a `missing_required_input` error for an absent `deploy_id`.

## Outputs (normative)

On success in human mode, the command MUST write this block to stdout and exit with code 0 (restore.rs:76).

```
Deployment restore accepted; deployment is no longer marked for deletion.

Deployment ID: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b
Activity ID: <activity id>
Activity Status: ACTIVITY_STATUS_COMPLETED
```

The rendering starts with one blank line (restore.rs:78).
The activity id is an API assigned value; Part 00 excludes it from vector comparison.

In JSON mode the command MUST emit one terminal NDJSON object with `reason` = `deployment_restored` (tvc/src/outcome.rs:50).
The object MUST carry the fields `deploymentId`, `activityId`, and `activityStatus` (restore.rs:53).
On success `activityStatus` MUST equal `ACTIVITY_STATUS_COMPLETED` (INV-1).
`activityStatus` serializes as the stable proto status name (restore.rs:58).
The wording and the field shape mirror the `deployment_deleted` outcome of `deploy delete` (tvc/src/commands/deploy/delete.rs:52).

## Side effects (normative)

INV-G4 governs tvc config load and creation; this command adds no config behavior of its own.
The command MUST submit exactly one activity: POST `/public/v1/submit/restore_tvc_deployment` (client/src/generated/client.rs:4347).
The client polls while the activity reports `PENDING` (client/src/lib.rs:362).
The command MUST NOT write other local files, MUST NOT change the tvc config, and MUST NOT use a YubiKey (restore.rs:28).

## Failure modes (normative)

In human mode the command MUST render a runtime error on stderr (tvc/src/cli.rs:132).
In JSON mode every runtime failure carries `reason` = `command_error`, and `code` carries the classification (tvc/src/output.rs:315).
A failed API call MUST carry the context `failed to restore TVC deployment` (restore.rs:44).
The command submits the activity without a preflight state check.
When the deployment carries no deletion mark, the command MUST surface the server rejection through the taxonomy below (restore.rs:40).

| Failure | Observed behavior | Source |
|---|---|---|
| `--deploy-id` absent, human mode | clap parse error on stderr; exit code 2. | tvc/tests/deploy_restore.rs:26 |
| `--deploy-id` absent or malformed, JSON mode | One JSON object: `reason` = `command_error`, `code` = `usage_error`; exit code 2. | tvc/src/cli.rs:160, tvc/src/output.rs:345 |
| `--deploy-id` value is no UUID | clap value parse error; exit code 2. | restore.rs:23 |
| No active org and no auth env | `code` = `command_error`; exit code 1; message: "No active organization. Run `tvc login` first." | tvc/src/client.rs:106 |
| Partial auth env trio | `code` = `command_error`; exit code 1; message names the missing variables. | tvc/src/client.rs:226 |
| No stored API key for the active org | `code` = `command_error`; exit code 1. | tvc/src/client.rs:117 |
| HTTP 401 or 403 | `code` = `unauthorized` with `httpStatus`; exit code 1. | tvc/src/errors.rs:219 |
| HTTP 404 | `code` = `not_found` with `httpStatus`; exit code 1. | tvc/src/errors.rs:220 |
| Other non-success HTTP status | `code` = `api_error` with `httpStatus`; exit code 1. | tvc/src/errors.rs:221 |
| Connect, timeout, or DNS failure | `code` = `network_error`; exit code 1. | tvc/src/errors.rs:229 |
| Activity needs consensus | `code` = `approval_required`; exit code 1. | client/src/lib.rs:375, tvc/src/errors.rs:236 |
| Activity `FAILED` or protocol violation | `code` = `api_error`; exit code 1. | client/src/lib.rs:372, tvc/src/errors.rs:252 |

## Test vectors (normative)

- **logged-in start**: a tvc config whose active org is `9d2f5c1e-7a4b-4c3d-9e8f-2a1b3c4d5e6f`, with API key files that `tvc login` stored.
- **empty HOME**: `HOME` points at an empty directory and no `TVC_*` variable is set (tvc/tests/deploy_restore.rs:7).
- Vector comparison excludes the Part 00 nondeterministic fields: `activityId`, `Activity ID`, and timestamps.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | logged-in start; the API completes the activity | `tvc deploy restore -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | stdout carries the human success block with `Deployment ID: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` and `Activity Status: ACTIVITY_STATUS_COMPLETED`; exit code 0 (restore.rs:76). |
| V-2 | logged-in start; the API completes the activity | `tvc deploy restore -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | One NDJSON object: `reason` = `deployment_restored`, `deploymentId` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, `activityStatus` = `ACTIVITY_STATUS_COMPLETED`; exit code 0 (tvc/src/outcome.rs:50, restore.rs:53). |
| V-3 | empty HOME | `tvc deploy restore` | stderr carries a clap error naming `--deploy-id <DEPLOY_ID>`; exit code 2 (tvc/tests/deploy_restore.rs:26). |
| V-4 | empty HOME | `tvc deploy restore --message-format json` | stdout carries one JSON object with `reason` = `command_error`, `code` = `usage_error`; exit code 2 (tvc/src/cli.rs:160, tvc/src/output.rs:345). |
| V-5 | empty HOME | `tvc deploy restore -d not-a-uuid --message-format json` | One JSON object with `reason` = `command_error`, `code` = `usage_error`; exit code 2 (restore.rs:23, tvc/src/cli.rs:160). |
| V-6 | empty HOME (INV-G4 writes a default tvc config) | `tvc deploy restore -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | `code` = `command_error`; message contains "No active organization. Run `tvc login` first."; exit code 1 (tvc/src/client.rs:106). |
| V-7 | empty HOME plus `TVC_ORG_ID=9d2f5c1e-7a4b-4c3d-9e8f-2a1b3c4d5e6f` only | `tvc deploy restore -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | `code` = `command_error`; message names missing `TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE`; exit code 1 (tvc/src/client.rs:226). |
| V-8 | logged-in start; the API answers HTTP 404 | `tvc deploy restore -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | `code` = `not_found`, `httpStatus` = 404; exit code 1 (tvc/src/errors.rs:220). |
| V-9 | logged-in start; the activity returns `CONSENSUS_NEEDED` | `tvc deploy restore -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | `code` = `approval_required`; exit code 1 (client/src/lib.rs:375, tvc/src/errors.rs:236). |
| V-10 | logged-in start; the activity returns `FAILED` | `tvc deploy restore -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | `code` = `api_error`; message contains `failed to restore TVC deployment`; exit code 1 (client/src/lib.rs:372, tvc/src/errors.rs:252, restore.rs:44). |
| V-11 | empty HOME | `tvc deploy restore --help` | Help text lists `--deploy-id <DEPLOY_ID>`; exit code 0 (tvc/tests/deploy_restore.rs:15). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | On success `activityStatus` MUST equal `ACTIVITY_STATUS_COMPLETED`. | `process_activity` returns `Ok` only for a completed activity; every other status maps to an error (client/src/lib.rs:361). |
| INV-2 | The command MUST NOT prompt in any mode. | `run` binds the context as `_ctx` and calls no prompt API (restore.rs:28); `deploy_id` is a required clap argument, so no input remains to collect (restore.rs:22). Behavioral check: tvc/tests/deploy_restore.rs:26. |
| INV-3 | The success `reason` MUST stay `deployment_restored`. | serde internal tagging derives the reason from the `DeploymentRestored` variant name (tvc/src/outcome.rs:30, tvc/src/outcome.rs:50). Behavioral checks: tvc/src/outcome.rs:124, tvc/src/outcome.rs:140. |

## Gaps (informative)

1. **[consistency] The delete and restore round trip has an asymmetric flag surface: restore accepts `-d`, delete is long only.**

   restore.rs:22 declares `#[arg(short, long, env = "TVC_DEPLOY_ID")]` while delete.rs:21
   declares `#[arg(long, env = "TVC_DEPLOY_ID", value_name = "DEPLOY_ID")]`. Every other
   deploy id consumer has the short flag (status.rs:29, get_status.rs:28, debug_logs.rs:72,
   provisioning_details.rs:27). `deploy delete` and `app set-live-deploy`
   (tvc/src/commands/app/set_live_deploy.rs:21) are the odd ones out. Recalling the delete
   invocation and swapping the verb therefore works in one direction only.

2. **[capability] The CLI offers no way to discover what a user can restore; the UUID comes from outside the CLI.**

   No `deploy list` command exists: tvc/src/cli.rs:392 declares the full deploy surface.
   restore never prompts or offers a picker (restore.rs:28). Only `deploy status -d <id>`
   shows the deletion mark (status.rs:110), and that command also requires the id. A user
   who deleted a deployment without capturing the id from the delete output cannot recover
   it with this tool.

3. **[capability] No idempotent outcome exists for a deployment without a deletion mark.**

   restore.rs:40 submits the activity blindly, and any rejection surfaces as a generic
   `api_error`. The sibling `deploy approve` models the analogous already-done case as a
   first-class terminal outcome, `manifest_approval_already_posted` (tvc/src/outcome.rs:39).
   Scripted delete and restore flows get a friendly signal there and a generic API error here.

4. **[docs] The help text says "Restore a deleted deployment", yet the command restores a deployment that carries a deletion mark.**

   tvc/src/cli.rs:420 and restore.rs:17 both say "deleted deployment". The help for `delete`
   is precise: "Delete a deployment by marking it for deletion" (tvc/src/cli.rs:418). The
   success message itself says "no longer marked for deletion" (restore.rs:79). The summary
   overstates what restore can do once the backend acts on the mark.
