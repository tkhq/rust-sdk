# tvc deploy delete

## Purpose

Marks a deployment for deletion by submitting `ACTIVITY_TYPE_DELETE_TVC_DEPLOYMENT` to
the Turnkey API (`tvc/src/commands/deploy/delete.rs:27-50`). Soft delete: the inverse
command `tvc deploy restore` un-marks it (`tvc/src/commands/deploy/restore.rs:27-51`).
Run it to retire a deployment you no longer want serving or provisioned.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| deployment id | `--deploy-id <DEPLOY_ID>` (no short) | `TVC_DEPLOY_ID` | — | — (required) | never |
| org id (auth) | — | `TVC_ORG_ID` | active org in `~/.config/turnkey/` | — | never |
| API key (auth) | — | `TVC_API_KEY_PUBLIC` / `TVC_API_KEY_PRIVATE` | active org's stored key file | — | never |
| API base URL | — | `TVC_API_BASE_URL` | active org's `api_base_url` | `https://api.turnkey.com` | never |

- `--deploy-id` follows flag > env normally (clap); there is no config-file or prompt
  fallback (`delete.rs:19-23`).
- Auth deviates from per-value resolution by design: the three auth env vars are
  all-or-nothing — all set wins over disk, none set falls back to the active org's
  config, a partial set is an error, never a merge (`tvc/src/client.rs:36-64,
  191-242`).
- Inherited globals (`--non-interactive`, `--message-format`) get no special
  treatment; the command ignores `ctx` entirely (`delete.rs:27` binds `_ctx`).

## Interactive behavior

None. No prompts, no confirmation, in either mode — interactive and
`--non-interactive`/JSON runs are byte-identical. A missing `--deploy-id` (with the
env var unset) is a clap usage error in both modes, not a
`missing_required_input`-style prompt fallback.

## Outputs

- Human: a blank leading line, then
  `Deployment delete accepted; deployment is marked for deletion.` plus
  `Deployment ID` / `Activity ID` / `Activity Status` lines (`delete.rs:73-88`).
  The message does not mention that `deploy restore` can undo it.
- JSON: one object, `reason: "deployment_deleted"` (`tvc/src/outcome.rs:30,49`),
  fields `deploymentId`, `activityId`, `activityStatus` (`delete.rs:52-59`).
- `activityStatus` is always `ACTIVITY_STATUS_COMPLETED` in practice: the client
  errors on every other terminal status (`client/src/lib.rs:360-386`).

## Side effects

- Dispatch-level: loads `~/.config/turnkey/` config, writing a default config file if
  none exists (`tvc/src/cli.rs:215-240`).
- Reads the active org's stored API key file when env auth is absent
  (`tvc/src/client.rs:103-125`).
- Submits the delete activity, polling/retrying while pending
  (`client/src/generated/client.rs:4287-4326`, `client/src/lib.rs:347-387`).
- No other file writes, no YubiKey interaction.

## Failure modes

- Missing/invalid `--deploy-id` (not a UUID): clap usage error, `usage_error`,
  exit 2 (`tvc/tests/deploy_delete.rs:26-35`).
- Partial auth env vars, no active org, missing key file: `command_error`, exit 1
  (`tvc/src/client.rs:104-117, 226-234`).
- HTTP 401/403 → `unauthorized`; 404 → `not_found`; other non-success →
  `api_error` (+`httpStatus`); connect/timeout/DNS → `network_error`
  (`tvc/src/errors.rs:212-268`). Exit 1.
- Activity needs consensus → `ActivityRequiresApproval` → `approval_required`,
  exit 1 (`client/src/lib.rs:375-377`, `tvc/src/errors.rs:236-238`): under a quorum
  policy the delete was accepted but reports as an error.
- Failed/unexpected activity status → `api_error`, exit 1.

## Gaps

1. **[consistency] Destructive command with no confirmation, while `profile delete`
   confirms — and the purpose-built shared helper is dead code.** `deploy delete`
   submits immediately with no prompt and no `--yes` (`delete.rs:27-50`); `profile
   delete` requires `confirm_or_bail` interactively and `--org` + `--yes`
   non-interactively (`tvc/src/commands/login.rs:129-136, 147-209`).
   `tvc/src/commands/confirmation.rs` (`confirm_yes_no`, `confirm_typed`) has zero
   production callers — it landed in the same commit (c2b9030e) that added
   `deploy delete`/`app delete`, and its tests literally prompt "Type app id"
   (`confirmation.rs:129-147`), so wiring confirmation into the delete commands was
   evidently intended and never done. `app delete` (deletes the app AND all its
   deployments, `tvc/src/commands/app/delete.rs:27-50`) shares the gap; the soft
   delete + `deploy restore` round trip is the only mitigation.

2. **[bug?] With `TVC_DEPLOY_ID` exported, bare `tvc deploy delete` deletes with no
   flag, no prompt, no output before acting.** The required arg is satisfiable by env
   (`delete.rs:21`), and the same env var feeds seven neutral siblings (status,
   get-status, provision, provisioning-details, debug-logs, approve, restore —
   e.g. `status.rs:29`, `provision.rs:34`), so exporting it for a status/provision
   workflow arms an argument-less destructive command. Compound of gap 1 and the
   shared env var.

3. **[capability] No `--yes` escape hatch is reserved, so fixing gap 1 later is a
   breaking change.** Scripts invoking `tvc deploy delete` today rely on
   zero-confirmation; introducing a prompt without a pre-existing `--yes` (the
   `profile delete` shape, `login.rs:56-58`) would break them. Adding `--yes` now (a
   no-op until confirmation exists) makes the fix non-breaking.

4. **[consistency] Only deploy-id-taking deploy command without a short `-d`;
   breaks delete↔restore round-trip symmetry.** `restore`, `status`, `get-status`,
   `provision`, `provisioning-details`, `debug-logs`, `approve` all take `-d`
   (`restore.rs:22`, `status.rs:29`, `provision.rs:34`, …); `delete` is long-only
   (`delete.rs:21`), so `tvc deploy delete -d <id>` fails while
   `tvc deploy restore -d <id>` works. `app delete`'s `--app-id` is also long-only
   (`app/delete.rs:21`), so this may be deliberate friction on deletes — but it is
   documented nowhere (no long_about, `delete.rs:18`).

5. **[capability] No pre-delete context and no restore pointer.** The command never
   fetches the deployment before deleting — `fetch_tvc_deployment` exists
   (`tvc/src/client.rs:83-100`) — so an interactive user sees nothing about what
   they are deleting (app, status, live-ness), and the success message
   (`delete.rs:78`) never mentions `deploy restore` as the undo. Contrast `profile
   delete`'s pre-deletion warning block naming exactly what is destroyed
   (`login.rs:153-204`).

6. **[docs] `activityStatus` in the JSON output can only ever be
   `ACTIVITY_STATUS_COMPLETED`.** `process_activity` returns only on `Completed` and
   errors on every other terminal status (`client/src/lib.rs:360-386`), so the field
   (`delete.rs:56-58`) implies variability that cannot occur. Shared with
   `deploy restore` and `app delete`; harmless, but consumers may build dead
   branches on it.
