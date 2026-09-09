# tvc app delete

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the app delete command.*

## Purpose (informative)

`tvc app delete` marks one app and every one of its deployments for deletion.
It submits the `ACTIVITY_TYPE_DELETE_TVC_APP_AND_DEPLOYMENTS` activity to the Turnkey API (tvc/src/commands/app/delete.rs:39-43, client/src/generated/client.rs:4144-4145).
No restore path exists for an app.
`deploy restore` reverses a deployment delete; no `app restore` command or `ACTIVITY_TYPE_RESTORE_TVC_APP` activity exists (proto/activities.json:383-397).
Run it to retire an app entirely.

## Acceptance scenario (normative)

Given: a logged-in profile whose active org is `7f0c2d4e-5a6b-4c8d-9e0f-1a2b3c4d5e6f`, and two apps in that org: `9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` and `2d3e4f5a-6b7c-4d8e-9f0a-1b2c3d4e5f6a`.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc app delete --app-id 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d`. | stdout shows `App delete accepted.`, `App and deployments marked for deletion.`, then the `App ID`, `Activity ID`, and `Activity Status` lines. Exit code 0. |
| 2 | Run `tvc app delete --app-id 2d3e4f5a-6b7c-4d8e-9f0a-1b2c3d4e5f6a --message-format json`. | stdout carries one NDJSON object with `reason` = `app_deleted`, `appId` = `2d3e4f5a-6b7c-4d8e-9f0a-1b2c3d4e5f6a`, and `activityStatus` = `ACTIVITY_STATUS_COMPLETED`. Exit code 0. |

All steps pass in one run from a clean start with one logged-in profile and two existing apps.

## Inputs (normative)

| Input | Flag | Env | Config source | Default | Prompted |
|---|---|---|---|---|---|
| app id | `--app-id <APP_ID>` (no short) | `TVC_APP_ID` | none | none (required) | never |
| org id (auth) | none | `TVC_ORG_ID` | the active org's `id` in the tvc config | none | never |
| API key (auth) | none | `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE` | the active org's stored API key file | none | never |
| API base URL | none | `TVC_API_BASE_URL` | the active org's `api_base_url` | `https://api.turnkey.com` | never |

`--app-id` MUST parse as a UUID (delete.rs:22); a non-UUID value fails before any config or network work.
Its resolution follows Part 00: flag first, then `TVC_APP_ID`.
No config source, built-in default, or prompt exists for it (delete.rs:21).
`deploy init` and `deploy create` fall back to the per-org `last_created_app_id` in the tvc config (tvc/src/config/turnkey.rs:710-712).
This command reads no such fallback, which suits a delete.

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
A missing app id MUST fail as a clap usage error in both modes (tvc/tests/app_delete.rs:39-48); no prompt fallback exists.
The command offers no confirmation step before the delete (Gap 1).

## Outputs (normative)

In human mode stdout MUST carry exactly this block (delete.rs:73-88):

```
App delete accepted.
App and deployments marked for deletion.

App ID: <app id>
Activity ID: <activity id>
Activity Status: ACTIVITY_STATUS_COMPLETED
```

The `App ID` value MUST come from the activity result's `app_id` field (delete.rs:45-49).

In JSON mode stdout MUST carry exactly one NDJSON object (tvc/src/output.rs:97-99).
The object MUST carry `reason` = `app_deleted` (tvc/src/outcome.rs:30, 56) plus the fields `appId`, `activityId`, and `activityStatus` (delete.rs:52-59).
`activityStatus` MUST serialize as the stable proto status name (delete.rs:57-58).
On success the value is always `ACTIVITY_STATUS_COMPLETED` because the client errors on every other terminal status (client/src/lib.rs:360-386).
The success message says `marked for deletion` although no restore path exists (Gap 7).

## Side effects (normative)

- Dispatch MUST load the tvc config before the command runs and MUST create the file when absent (INV-G4; tvc/src/cli.rs:215-240).
- When env auth is absent, the command MUST read the active org's stored API key file (tvc/src/client.rs:115-117).
- The command MUST submit exactly one `ACTIVITY_TYPE_DELETE_TVC_APP_AND_DEPLOYMENTS` activity (client/src/generated/client.rs:4135-4156). The client polls while the activity reports the pending status (client/src/lib.rs:362-371).
- The command MUST NOT mutate the tvc config. In particular it leaves the org's `last_created_app_id` in place after the delete (Gap 6).
- The command MUST NOT write any other file and MUST NOT touch a YubiKey.

## Failure modes (normative)

In JSON mode every failure MUST surface per INV-G3 with `reason` = `command_error` and the listed `code` (tvc/src/output.rs:315, 335-341, 344-352).
In human mode the failure MUST render on stderr with an `error:` prefix (tvc/src/output.rs:161-182).

| Failure | `code` | Exit code | Evidence |
|---|---|---|---|
| `--app-id` absent and `TVC_APP_ID` unset | `usage_error` | 2 | tvc/tests/app_delete.rs:39-48 |
| App id value fails UUID parsing | `usage_error` | 2 | delete.rs:22 (clap value parser) |
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
No follow-up command exists to resume or inspect a pending app deletion.

## Test vectors (normative)

Vector comparison excludes the Part 00 nondeterministic fields.
For this command those are `activityId` and the `Activity ID` line.
V-10 represents the HTTP status classification family; the `not_found` and other-status rows follow the same classifier (tvc/src/errors.rs:214-226).
The failed-activity row also flows through that classifier as `api_error` (tvc/src/errors.rs:247-258).

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Active org `7f0c2d4e-5a6b-4c8d-9e0f-1a2b3c4d5e6f` with valid key files; app `9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` exists | `tvc app delete --app-id 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` | stdout: `App delete accepted.`, `App and deployments marked for deletion.`, one blank line, `App ID: 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d`, `Activity ID: <excluded>`, `Activity Status: ACTIVITY_STATUS_COMPLETED`; exit code 0 (delete.rs:73-88). |
| V-2 | Same as V-1 | `tvc app delete --app-id 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d --message-format json` | stdout: one NDJSON object `{"reason":"app_deleted","appId":"9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d","activityId":"<excluded>","activityStatus":"ACTIVITY_STATUS_COMPLETED"}`; exit code 0 (tvc/src/outcome.rs:56, delete.rs:52-59). |
| V-3 | Same as V-1 plus `TVC_APP_ID=9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` exported | `tvc app delete` | Same as V-1: the env var satisfies the required argument (delete.rs:21). Pins Gap 3's current behavior. |
| V-4 | `TVC_APP_ID` unset | `tvc app delete` | stderr contains `the following required arguments were not provided` and `--app-id <APP_ID>`; exit code 2 (tvc/tests/app_delete.rs:39-48). |
| V-5 | `TVC_APP_ID` unset | `tvc app delete --message-format json` | stdout: one JSON object with `reason` = `command_error`, `code` = `usage_error`, `message` carrying clap's text; exit code 2 (tvc/src/cli.rs:160-176, tvc/src/output.rs:344-352). |
| V-6 | none | `tvc app delete --app-id not-a-uuid` | clap invalid-value error on stderr; exit code 2 (delete.rs:22). |
| V-7 | Only `TVC_ORG_ID=7f0c2d4e-5a6b-4c8d-9e0f-1a2b3c4d5e6f` exported | `tvc app delete --app-id 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` | Error message: `partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE. Set all three (TVC_ORG_ID, TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE) env vars or none.`; `code` = `command_error`; exit code 1 (tvc/src/client.rs:226-234). |
| V-8 | Fresh HOME (dispatch writes a default tvc config with no active org); no auth env vars | `tvc app delete --app-id 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` | Error message: ``No active organization. Run `tvc login` first.``; `code` = `command_error`; exit code 1 (tvc/src/client.rs:104-106). |
| V-9 | Active org `acme` whose `api_key_path` points to a missing file; no auth env vars | `tvc app delete --app-id 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` | Error message: ``No API key found for org 'acme'. Run `tvc login` first.``; `code` = `command_error`; exit code 1 (tvc/src/client.rs:115-117, tvc/src/config/turnkey/api_key.rs:32-35). |
| V-10 | Valid auth; the API answers HTTP 401 | `tvc app delete --app-id 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` | `code` = `unauthorized`, `httpStatus` = 401; exit code 1 (tvc/src/errors.rs:214-226). |
| V-11 | Valid auth; the org enforces a quorum policy on the delete | `tvc app delete --app-id 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` | `code` = `approval_required`; exit code 1; the API keeps the activity pending approvals (client/src/lib.rs:375-377, tvc/src/errors.rs:236-238). |
| V-12 | Full env auth plus `TVC_API_BASE_URL=https://127.0.0.1:1` | `tvc app delete --app-id 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` | `code` = `network_error`; exit code 1 (tvc/src/errors.rs:229-235). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT prompt in any mode. | `run` binds the context as `_ctx` and contains no prompt call (delete.rs:27-50); INV-G1 covers JSON mode. |
| INV-2 | Every success outcome MUST serialize with `reason` = `app_deleted`. | serde internal tagging derives the reason from the `Outcome` variant name; a duplicate variant fails to compile (tvc/src/outcome.rs:30, 56). Behavioral check: tvc/src/outcome.rs:123-137. |
| INV-3 | A run that exits 0 MUST report `activityStatus` = `ACTIVITY_STATUS_COMPLETED`. | `process_activity` returns only on the completed status and errors on every other terminal status (client/src/lib.rs:360-386). |
| INV-4 | The command MUST NOT write files beyond the dispatch-level tvc config creation (INV-G4). | `run` performs only the client build and one API call; no file write exists in the module (delete.rs:27-50). |

## Gaps (informative)

1. **[consistency] The CLI's most destructive command has no confirmation, while the less destructive `profile delete` confirms**.
   `app delete` irreversibly deletes an app plus all its deployments with no prompt and no `--yes` style flag (delete.rs:27-50, `ctx` unused).
   `profile delete`, a purely local operation, demands interactive confirmation or `--yes`, and requires `--yes` in non-interactive mode (tvc/src/commands/login.rs:56-58, 129-136, 205).
   The original implementation had a double confirmation: `confirm_yes_no` plus a typed app id echo, behind a `--dangerous-skip-confirmation` escape hatch.
   Commit 8234cacf ("review") stripped it and kept the helpers.

2. **[consistency] `tvc/src/commands/confirmation.rs` exists for exactly this command and has no production caller**.
   `confirm_yes_no` and `confirm_typed` have zero callers outside their own unit tests (confirmation.rs:11, 20).
   The tests still prompt `Type app id` (confirmation.rs:133-155).
   Live commands that confirm use `prompts::confirm_or_bail` instead (tvc/src/prompts.rs:70; for example yubikey/unregister.rs:104).
   Either wire `confirm_typed` back into `app delete`, its intended consumer, or delete the module.

3. **[bug?] The shared `TVC_APP_ID` env var silently selects the deletion target**.
   The same env var feeds `app status` and `deploy create` (app/status.rs:27, deploy/create.rs:105).
   An exported `TVC_APP_ID` from an unrelated workflow makes a bare `tvc app delete` valid (delete.rs:21).
   Combined with gap 1, that is an immediate, unconfirmed, unrecoverable deletion.
   Ambient state silently supplies the target of an explicit destructive choice, in its most dangerous direction.
   V-3 pins the current behavior.

4. **[capability] No restore path exists for a deleted app**.
   `deploy delete` pairs with `deploy restore` (tvc/src/cli.rs:418-421).
   No `app restore` command exists, and no `ACTIVITY_TYPE_RESTORE_TVC_APP` activity exists upstream (proto/activities.json:383-397).
   The upstream API accounts for part of the gap; the CLI surface asymmetry is real.
   Whether `deploy restore` can revive the cascade-deleted deployments individually is unclear and undocumented.

5. **[capability] No interactive app selection or name-based targeting exists**.
   `--app-id` is a hard clap requirement even on a TTY (tvc/tests/app_delete.rs:39-48).
   The global contract allows prompts for missing values (tvc/src/cli.rs:41-42).
   `deploy create` fills missing inputs interactively (deploy/create.rs:161-168) and `profile delete` offers a picker (login.rs:313-328).
   The user has to know the UUID; no select-from-`app list` flow exists.
   Strictness is reasonable for a delete once a confirmation step (gap 1) backs it up.

6. **[bug?] Deleting an app leaves a stale `last_created_app_id` that points at it**.
   `app create` records the id per org (app/create.rs:272).
   `deploy init` and `deploy create` default to it (deploy/init.rs:85, deploy/create.rs:202).
   `app delete` never clears it, so a later `deploy init` or `deploy create` silently targets a deleted app.

7. **[docs] Help text omits the soft delete framing and the absence of a restore path**.
   The subcommand help is one line, "Delete an app and all of its deployments" (tvc/src/cli.rs:454-455), with no `long_about` (delete.rs:18).
   Only the post-success output reveals "marked for deletion" (delete.rs:77-78).
   That phrasing implies a recoverability the CLI does not offer (gap 4).
   Sibling `deploy delete` says "by marking it for deletion" up front (tvc/src/cli.rs:418).
