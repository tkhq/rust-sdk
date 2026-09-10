# tvc app set-live-deploy

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the app set-live-deploy command.*

## Purpose (informative)

The command points an app's live traffic at one deployment.
It submits the `ACTIVITY_TYPE_UPDATE_TVC_APP_LIVE_DEPLOYMENT` activity for the given deployment id.
The backend resolves the owning app and gates the change on deployment health and `delete=false` (proto/immutable/activity/v1/activity.proto:7640-7645).
The enclave-controller shifts traffic immediately after the activity completes.
Operators run the command after a new deployment reaches a healthy state, to cut traffic over to it.

## Acceptance scenario (normative)

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc login` and select the target org. | Exit code 0. The tvc config stores the org as the active org. |
| 2 | Provision a deployment for app `4f2b9c7d-3e1a-4b8c-9d6e-2a5f8c1b7e94` and wait until it is healthy. | `tvc app status --app-id 4f2b9c7d-3e1a-4b8c-9d6e-2a5f8c1b7e94` lists deployment `5376f492-d014-4e01-a6bb-20fc97448e25` with ready replicas. |
| 3 | Run `tvc app set-live-deploy --deploy-id 5376f492-d014-4e01-a6bb-20fc97448e25`. | Exit code 0. stdout shows `Set-live-deploy accepted.`, `Deployment ID: 5376f492-d014-4e01-a6bb-20fc97448e25`, an `Activity ID` line, and `Activity Status: ACTIVITY_STATUS_COMPLETED`. |
| 4 | Run `tvc app status --app-id 4f2b9c7d-3e1a-4b8c-9d6e-2a5f8c1b7e94` again. | The report shows `Targeted Deployment: 5376f492-d014-4e01-a6bb-20fc97448e25` (tvc/src/commands/app/status.rs:107-115). |

All steps MUST pass in one run against one org that holds one healthy deployment.

## Inputs (normative)

| Input | Flag | Env | Config source | Default | Prompted |
|---|---|---|---|---|---|
| deployment id (UUID) | `--deploy-id` | `TVC_DEPLOY_ID` | none | none (required) | never |
| org id | none | `TVC_ORG_ID` | active org id in the tvc config | none | never |
| API key | none | `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE` | the active org's stored key | none | never |
| API base URL | none | `TVC_API_BASE_URL` | the active org's `api_base_url` | `https://api.turnkey.com` | never |

The command MUST resolve `--deploy-id` in the Part 00 value resolution order.
The input has no config file layer and no built-in default (tvc/src/commands/app/set_live_deploy.rs:19-23).
`clap` MUST parse the value as a UUID (set_live_deploy.rs:22).

When all three auth environment variables hold values, `build_client` MUST use them (tvc/src/client.rs:48-64).
When none hold values, it MUST fall back to the active org in the tvc config (tvc/src/client.rs:103-125).
A partial set MUST fail with the list of missing names (tvc/src/client.rs:226-234).

The command takes no app id input.
The intent MUST carry only `deployment_id` (INV-4); the backend resolves the owning app.

## Interactive behavior (normative)

The command MUST NOT prompt in any mode (INV-1).
`run` ignores its `ctx` parameter (set_live_deploy.rs:27).
A missing `--deploy-id` MUST fail as a clap usage error with exit code 2 in every mode (tvc/tests/app_set_live_deploy.rs:32-37).
It MUST NOT take the `missing_required_input` path.
`--non-interactive` and JSON mode change no command behavior.

## Outputs (normative)

In human mode the command MUST print one blank line and then a `Set-live-deploy accepted.` block (set_live_deploy.rs:75-90).
The block MUST show `Deployment ID`, `Activity ID`, and `Activity Status` lines.

In JSON mode the command MUST emit one NDJSON object with `reason` = `live_deployment_set` (tvc/src/outcome.rs:30,55).
The object MUST carry the camelCase fields `deploymentId`, `activityId`, and `activityStatus` (set_live_deploy.rs:54-61).

On success `activityStatus` MUST equal `ACTIVITY_STATUS_COMPLETED` (INV-2).
The `deploymentId` value MUST equal the input UUID (INV-3).
The activity result proto carries no fields (client/src/generated/immutable.activity.v1.rs:4603).

## Side effects (normative)

The command MUST load the tvc config before it runs and MUST create the file when it is absent (INV-G4; tvc/src/cli.rs:215-223).
It MUST submit one activity POST to `/public/v1/submit/set_tvc_app_live_deployment` (client/src/generated/client.rs:4242-4283).
The client polls while the activity reports a pending status (client/src/lib.rs:362-371).
Production traffic shifts to the target deployment when the activity completes (activity.proto:7643-7645).
The command MUST NOT write any other file and MUST NOT use a YubiKey.

## Failure modes (normative)

Each failure MUST map to the Part 00 error taxonomy as this table states.

| Failure | JSON `code` | Exit code | Mechanism |
|---|---|---|---|
| `--deploy-id` absent or malformed | `usage_error` (JSON `reason` = `command_error`) | 2 | clap parse failure; JSON path through `handle_parse_error` (tvc/src/cli.rs:154-182; tvc/src/output.rs:344-352) |
| `HOME` unset | `command_error` | 1 | tvc/src/cli.rs:215 |
| No active org and no env auth | `command_error` | 1 | tvc/src/client.rs:104-106 |
| Stored API key missing | `command_error` | 1 | tvc/src/client.rs:115-117 |
| Partial env auth | `command_error` | 1 | tvc/src/client.rs:226-234; tvc/tests/app_set_live_deploy.rs:40-53 |
| HTTP 401 or 403 | `unauthorized` with `httpStatus` | 1 | tvc/src/errors.rs:219 |
| HTTP 404 | `not_found` with `httpStatus` | 1 | tvc/src/errors.rs:220 |
| Other non-success HTTP status | `api_error` with `httpStatus` | 1 | tvc/src/errors.rs:221 |
| Connect or timeout failure | `network_error` | 1 | tvc/src/errors.rs:229-235 |
| Activity failed, rejected, or poll retries exceeded | `api_error` | 1 | client/src/lib.rs:362-384; tvc/src/errors.rs:247-258 |
| Activity needs consensus | `approval_required` | 1 | client/src/lib.rs:375-377; tvc/src/errors.rs:236-238 |
| System clock before the Unix epoch | `command_error` | 1 | set_live_deploy.rs:36-39 |

A backend health gate rejection surfaces as a failed activity, so its code is `api_error` (activity.proto:7641-7642).
On `approval_required` the error message reports the activity id.
The CLI offers no command that approves or resumes that pending activity.

## Test vectors (normative)

Vector comparison excludes the nondeterministic fields that Part 00 lists.
For this command the excluded value is the activity id: the `Activity ID` line and the `activityId` field.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Active org with a valid stored API key; healthy deployment `5376f492-d014-4e01-a6bb-20fc97448e25` | `tvc app set-live-deploy --deploy-id 5376f492-d014-4e01-a6bb-20fc97448e25` | Exit code 0. stdout: one blank line, then `Set-live-deploy accepted.`, `Deployment ID: 5376f492-d014-4e01-a6bb-20fc97448e25`, `Activity ID: <excluded>`, `Activity Status: ACTIVITY_STATUS_COMPLETED` (set_live_deploy.rs:75-90). |
| V-2 | Same as V-1 | V-1 invocation plus `--message-format json` | Exit code 0. One NDJSON object: `{"reason":"live_deployment_set","deploymentId":"5376f492-d014-4e01-a6bb-20fc97448e25","activityId":"<excluded>","activityStatus":"ACTIVITY_STATUS_COMPLETED"}` (tvc/src/outcome.rs:30,55; set_live_deploy.rs:54-61). |
| V-3 | Any environment | `tvc app set-live-deploy` | Exit code 2. stderr contains `--deploy-id <DEPLOY_ID>` (tvc/tests/app_set_live_deploy.rs:32-37). |
| V-4 | Any environment | `tvc app set-live-deploy --message-format json` | Exit code 2. One NDJSON object on stdout with `reason` = `command_error` and `code` = `usage_error` (tvc/src/cli.rs:160-176; tvc/src/output.rs:344-352). |
| V-5 | Any environment | `tvc app set-live-deploy --deploy-id not-a-uuid` | Exit code 2. clap value error from the `Uuid` field type (set_live_deploy.rs:22). |
| V-6 | Fresh `HOME`; env `TVC_ORG_ID=org_env`; no other auth env vars | `tvc app set-live-deploy --deploy-id 5376f492-d014-4e01-a6bb-20fc97448e25` | Exit code 1. stderr contains `partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE` (tvc/src/client.rs:226-234; tvc/tests/app_set_live_deploy.rs:40-53). |
| V-7 | Fresh `HOME` with no tvc config; no auth env vars | V-6 invocation plus `--message-format json` | Exit code 1. One NDJSON object with `code` = `command_error`; `message` contains `No active organization` (tvc/src/client.rs:104-106). |
| V-8 | `HOME` unset | `tvc app set-live-deploy --deploy-id 5376f492-d014-4e01-a6bb-20fc97448e25` | Exit code 1. Error contains `HOME environment variable not set`; JSON `code` = `command_error` (tvc/src/cli.rs:215). |
| V-9 | Valid auth; API answers HTTP 401 | V-2 invocation | Exit code 1. One NDJSON object with `code` = `unauthorized` and `httpStatus` = 401 (tvc/src/errors.rs:214-226). |
| V-10 | Valid auth; activity ends `ACTIVITY_STATUS_FAILED` | V-2 invocation | Exit code 1. One NDJSON object with `code` = `api_error` (client/src/lib.rs:372-374; tvc/src/errors.rs:252-258). |
| V-11 | Valid auth; activity needs consensus | V-2 invocation | Exit code 1. One NDJSON object with `code` = `approval_required`; `message` contains the activity id (client/src/lib.rs:375-377; tvc/src/errors.rs:236-238). |

## Invariants (normative)

Global invariants INV-G1 to INV-G4 apply. These invariants are specific to this command.

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT prompt in any mode. | `run` ignores its `ctx` parameter (set_live_deploy.rs:27); the only command input is a required clap flag (set_live_deploy.rs:21-22). Behavioral check: tvc/tests/app_set_live_deploy.rs:32-37. |
| INV-2 | A success outcome MUST carry `activityStatus` = `ACTIVITY_STATUS_COMPLETED`. | `process_activity` returns only completed activities and maps every other terminal status to an error (client/src/lib.rs:360-385). |
| INV-3 | The emitted `deploymentId` MUST equal the parsed `--deploy-id` value in canonical UUID form. | `run` renders the parsed `Uuid` once and moves that one string into both the intent and the outcome (set_live_deploy.rs:30-51). |
| INV-4 | The submitted intent MUST carry only `deployment_id`. | `UpdateTvcAppLiveDeploymentIntent` has one field (set_live_deploy.rs:32-34; activity.proto:7646-7651). |

## Gaps (informative)

1. **[capability] No interactive path exists: the traffic cutover command cannot list or pick a deployment in any mode**.
   `--deploy-id` is a hard clap requirement even on a TTY (set_live_deploy.rs:21-22; tvc/tests/app_set_live_deploy.rs:32-37).
   The data for a picker exists: `get_app_status` returns deployments plus `targeted_deployment_id` (tvc/src/commands/app/status.rs:38-62).
   The prompt primitive exists too (`prompts::select`, tvc/src/prompts.rs:78-80).
   Sibling `deploy create` prompts for a missing app id with a saved default (tvc/src/config/deploy.rs:144-151); this command never uses `ctx` at all.
   An `--app-id` scoped interactive selection (flag, then env, then prompt) would match the CLI's own endpoint pattern.

2. **[capability] No confirmation guards the immediate shift of production traffic**.
   The proto states that the enclave-controller shifts traffic immediately (activity.proto:7643-7645).
   The command submits without any confirm in interactive mode.
   `yubikey unregister` only edits local config, yet it requires `confirm_or_bail` (tvc/src/commands/yubikey/unregister.rs:104).
   `app delete` and `deploy delete` share this hole, so it is family wide; this command is where a `confirm_or_bail` plus `--yes` escape hatch matters most.
   The dedicated `commands/confirmation.rs` helpers have zero production callers (confirmation.rs:11,20); the live idiom is `prompts::confirm_or_bail`.

3. **[capability] The output omits which app changed and what it changed from**.
   The user supplies only a deployment id, and the outcome echoes it back (set_live_deploy.rs:47-51).
   The resolved app id is one lookup away (`fetch_tvc_deployment`, tvc/src/client.rs:83-100).
   So is the previous live deployment (`TvcApp.live_deployment_id`, client/src/generated/external.data.v1.rs:705).
   A JSON consumer cannot tell which app's traffic moved.
   A rollback needs extra calls, and the consumer already has to know how to make them.

4. **[consistency] `activityStatus` in the outcome is vacuous**.
   On success it can only be `ACTIVITY_STATUS_COMPLETED`, because `process_activity` errors on every other terminal status (client/src/lib.rs:360-385).
   The field and its `Default` impl (set_live_deploy.rs:63-73) suggest a range of outcomes that cannot occur.
   `app delete`, `deploy delete`, and `deploy restore` share this shape.

5. **[consistency] The deploy siblings offer a `-d` short flag; `set-live-deploy` accepts only the long flag**.
   `deploy status`, `deploy get-status`, `deploy provision`, and `deploy restore` take `-d` (deploy/status.rs:29, get_status.rs:28, provision.rs:34, restore.rs:22).
   All four bind the same `TVC_DEPLOY_ID` input; this command is long only (set_live_deploy.rs:21).
   The command also hand rolls the epoch timestamp (set_live_deploy.rs:36-39) while the crate offers `timestamp_ms()` (tvc/src/operator.rs:478).
   Several siblings share that repo wide backlog shape.

6. **[docs] The help text omits the preconditions and the immediacy of the cutover**.
   `about` is one line and `long_about` is `None` (set_live_deploy.rs:16-18).
   Nothing tells the user about the health gate and the `delete=false` gate, or that traffic shifts immediately on completion.
   Only the proto documents these facts (activity.proto:7640-7645), and a health gate rejection surfaces as a generic `api_error`.
