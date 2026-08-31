# tvc app set-live-deploy

## Purpose

Points an app's live traffic at a specific deployment. Submits the
`ACTIVITY_TYPE_UPDATE_TVC_APP_LIVE_DEPLOYMENT` activity for the given deployment id; the
backend resolves the owning app, gates on deployment health and delete=false, and the
enclave-controller shifts traffic immediately once the activity completes
(`proto/immutable/activity/v1/activity.proto:7640-7645`). Run it after a new deployment
is provisioned and healthy to cut traffic over to it.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| deployment id (UUID) | `--deploy-id` | `TVC_DEPLOY_ID` | — | — (required) | never |
| auth: org id | — | `TVC_ORG_ID` | active org in `~/.config/turnkey` | — | never |
| auth: API key | — | `TVC_API_KEY_PUBLIC` / `TVC_API_KEY_PRIVATE` | active org's stored key | — | never |
| auth: API base URL | — | `TVC_API_BASE_URL` | active org's `api_base_url` | `https://api.turnkey.com` | never |

No deviations from flag > env > config > default: `--deploy-id` simply has no config-file
layer and no default (`tvc/src/commands/app/set_live_deploy.rs:19-23`). Auth follows the
global all-three-env-vars-or-none rule; partial env auth errors
(`tvc/src/client.rs:48-64,192-242`). No app id input exists anywhere — the intent carries
only `deployment_id` and the backend infers the app.

## Interactive behavior

None. The command never prompts and ignores `ctx` entirely (`_ctx`,
`set_live_deploy.rs:27`). `--non-interactive` / JSON mode change nothing: a missing
`--deploy-id` is a clap parse error in every mode (exit 2; verified by
`tvc/tests/app_set_live_deploy.rs:32-37`), never a prompt and never the
`missing_required_input` path.

## Outputs

- Human: a `Set-live-deploy accepted.` block (preceded by one blank line) with
  `Deployment ID`, `Activity ID`, `Activity Status` (`set_live_deploy.rs:75-90`).
- JSON: one NDJSON object, `reason: "live_deployment_set"` (`tvc/src/outcome.rs:55`),
  camelCase fields `deploymentId`, `activityId`, `activityStatus` (stable proto name).
  On success `activityStatus` is always `ACTIVITY_STATUS_COMPLETED`: the client's
  `process_activity` polls Pending and turns every other terminal status into an error
  (`client/src/lib.rs:360-385`), so the field carries no information.
- The echoed `deploymentId` is the input value; the activity result proto is empty
  (`client/src/generated/immutable.activity.v1.rs:4603`).

## Side effects

- Reads `~/.config/turnkey/config.toml`; like every dispatched command, writes a default
  config file if none exists (`tvc/src/cli.rs:219-223`).
- One Turnkey activity POST to `/public/v1/submit/set_tvc_app_live_deployment`
  (`client/src/generated/client.rs:4242-4283`), with retry-polling while the activity is
  Pending. Production traffic shifts to the target deployment on completion.
- No files written by the command itself; no YubiKey interaction.

## Failure modes

- Missing/non-UUID `--deploy-id`: clap usage error, exit 2 (JSON: `command_error` /
  `usage_error` NDJSON via `cli.rs:154-182`).
- `HOME` unset, no active org, missing stored API key, partial env auth: `command_error`,
  exit 1 (`cli.rs:215`, `client.rs:103-125,225-234`; test
  `app_set_live_deploy.rs:40-53`).
- HTTP 401/403 → `unauthorized`; 404 → `not_found`; other non-success → `api_error` with
  `httpStatus`; connect/timeout → `network_error` (`tvc/src/errors.rs:214-234`).
- Activity Failed / Rejected / poll retries exceeded → `api_error`
  (`errors.rs:250-258`) — this is how a health-gate rejection surfaces.
- Activity needs consensus → `approval_required` (`errors.rs:236-238`), exit 1; the CLI
  reports the activity id inside the error message but has no command to approve or
  resume that pending activity later.
- Clock before epoch → `command_error` (`set_live_deploy.rs:36-38`).

## Gaps

1. **[capability] No interactive path: the traffic-cutover command cannot list or pick a
   deployment, in any mode.** `--deploy-id` is a hard clap requirement even on a TTY
   (`set_live_deploy.rs:21-22`; `tests/app_set_live_deploy.rs:32-37`), while the data to
   drive a picker exists (`get_app_status` returns deployments plus
   `targeted_deployment_id`, `tvc/src/commands/app/status.rs:38-62`) and the prompt
   primitive exists (`prompts::select`, `tvc/src/prompts.rs:78-80`). Sibling `deploy
   create` prompts for a missing app id with a saved default
   (`tvc/src/config/deploy.rs:144-151`); this command never uses `ctx` at all. A
   `--app-id`-scoped interactive selection (flag > env > prompt) would match the CLI's
   own endpoint pattern.

2. **[capability] No confirmation before immediately shifting production traffic.** The
   proto is explicit that the enclave-controller shifts traffic immediately
   (`activity.proto:7643-7645`), yet the command submits without any confirm in
   interactive mode — while `yubikey unregister`, which only edits local config,
   requires `confirm_or_bail` (`tvc/src/commands/yubikey/unregister.rs:104`). `app
   delete` / `deploy delete` share this hole, so it is family-wide; this is the command
   where a `confirm_or_bail` + `--yes` escape hatch matters most. (The dedicated
   `commands/confirmation.rs` helpers have zero production callers —
   `confirmation.rs:11,20` — the live idiom is `prompts::confirm_or_bail`.)

3. **[capability] Output omits which app changed and what it changed from.** The user
   supplies only a deployment id; the outcome echoes it back
   (`set_live_deploy.rs:47-51`) without the resolved app id or the previous live
   deployment, both of which are one lookup away (`fetch_tvc_deployment`,
   `tvc/src/client.rs:83-100`; `TvcApp.live_deployment_id`,
   `client/src/generated/external.data.v1.rs:705`). A JSON consumer cannot tell which
   app's traffic moved, or roll back, without extra calls it must already know how to
   make.

4. **[consistency] `activityStatus` in the outcome is vacuous.** On success it can only
   be `ACTIVITY_STATUS_COMPLETED` because `process_activity` errors on every other
   terminal status (`client/src/lib.rs:360-385`); the field (and its `Default` impl,
   `set_live_deploy.rs:63-73`) suggests a range of outcomes that cannot occur. Shared
   with `app delete` / `deploy delete` / `deploy restore`.

5. **[consistency] No `-d` short flag where the sibling deploy commands have one.**
   `deploy status` / `get-status` / `provision` / `restore` all take `-d` for the same
   `TVC_DEPLOY_ID` input (`deploy/status.rs:29`, `deploy/get_status.rs:28`,
   `deploy/provision.rs:34`, `deploy/restore.rs:22`); `set-live-deploy` is long-only
   (`set_live_deploy.rs:21`). Also hand-rolls the epoch timestamp
   (`set_live_deploy.rs:36-39`) instead of the crate's `timestamp_ms()`
   (`tvc/src/operator.rs:478`) — a repo-wide backlog shape shared with several siblings.

6. **[docs] Help text omits the preconditions and the immediacy of the cutover.** `about`
   is one line with `long_about = None` (`set_live_deploy.rs:16-18`); nothing tells the
   user the deployment must be healthy and not deleted, or that traffic shifts
   immediately on completion — all documented only in the proto
   (`activity.proto:7640-7645`), with health-gate rejections surfacing as a generic
   `api_error`.
