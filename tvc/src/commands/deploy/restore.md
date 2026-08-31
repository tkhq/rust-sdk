# tvc deploy restore

## Purpose
Un-marks a deployment that `tvc deploy delete` marked for deletion, by submitting an
`ACTIVITY_TYPE_RESTORE_TVC_DEPLOYMENT` activity. Run it to cancel a pending deletion
before the backend reaps the deployment. Exact inverse of `deploy delete`
(`tvc/src/commands/deploy/delete.rs`).

## Inputs
| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| deployment id | `-d, --deploy-id <DEPLOY_ID>` (Uuid) | `TVC_DEPLOY_ID` | — | — (required) | no |
| auth (org + API key) | — | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` (all-or-none) | active org in `~/.config/turnkey/` (`tvc login`) | — | no |
| API base URL | — | `TVC_API_BASE_URL` | per-org `api_base_url` | `https://api.turnkey.com` | no |

Deviation from the advertised flag > env > config > default order: `deploy_id` has no
config-file source and no default — flag > env only (restore.rs:22). Same for every
sibling deploy command, so `TVC_DEPLOY_ID` exported for one deploy command silently
feeds all of them. Auth follows the shared `build_client` rule: env trio wins outright,
partial env trio is a hard error, otherwise stored login config (tvc/src/client.rs:48,
tvc/src/client.rs:192).

## Interactive behavior
None. `run` takes `_ctx` and never prompts (restore.rs:28). `--non-interactive` and
JSON mode change nothing command-side: `deploy_id` is a hard clap requirement in both
modes; omitting it is a parse error (exit 2), not a `missing_required_input`.

## Outputs
Human mode (stdout):

```
Deployment restore accepted; deployment is no longer marked for deletion.

Deployment ID: <uuid>
Activity ID: <id>
Activity Status: ACTIVITY_STATUS_COMPLETED
```

JSON mode: one terminal NDJSON object, `reason: "deployment_restored"`
(tvc/src/outcome.rs:50), fields `deploymentId`, `activityId`, `activityStatus`.
`activityStatus` is invariantly `ACTIVITY_STATUS_COMPLETED` on success: the client's
`process_activity` only returns `Ok` for a completed activity — it polls `PENDING` and
errors on every other status (client/src/lib.rs:361-385). Wording and field shape are
exactly symmetric with `deploy delete`'s `deployment_deleted` outcome
(delete.rs:52-88).

## Side effects
- Reads `~/.config/turnkey/` CLI config; if the config file is absent the dispatcher
  writes a default one before running any command, this one included (tvc/src/cli.rs:219-223).
- One Turnkey activity submission: POST `/public/v1/submit/restore_tvc_deployment`
  (client/src/generated/client.rs:4330-4369), with client-side polling while pending.
- No other local file writes, no config mutation, no YubiKey interaction.

## Failure modes
- Missing/non-UUID `--deploy-id`: clap parse error; exit 2; JSON mode emits
  `reason: "command_error"`, `code: "usage_error"` (cli.rs:154-182, output.rs:345).
- No auth (no active org / partial env trio): `command_error`, exit 1
  (client.rs:106, client.rs:227).
- HTTP 401/403 → `unauthorized`; 404 → `not_found`; other non-success → `api_error`
  (all with `httpStatus`); connect/timeout/DNS → `network_error` (errors.rs:212-268).
- Activity needs consensus → `ActivityRequiresApproval` → `approval_required`
  (errors.rs:236-238); activity `FAILED` or protocol violations → `api_error`
  (errors.rs:252-258). All runtime failures exit 1 wrapped in
  "failed to restore TVC deployment" (restore.rs:44).
- Restoring a deployment that is not marked for deletion has no preflight: whatever the
  API returns is surfaced through the classification above.

## Gaps
1. **[consistency] The delete→restore round trip has an asymmetric flag surface: restore accepts `-d`, delete is long-only.**
   restore.rs:22 declares `#[arg(short, long, env = "TVC_DEPLOY_ID")]` while delete.rs:21
   is `#[arg(long, env = "TVC_DEPLOY_ID", value_name = "DEPLOY_ID")]`. Every other
   deploy-id consumer has the short flag (status.rs:29, get_status.rs:28,
   debug_logs.rs:72, provisioning_details.rs:27); `deploy delete` and
   `app set-live-deploy` (set_live_deploy.rs:21) are the odd ones out, so recalling the
   delete invocation and swapping the verb only works in one direction.

2. **[capability] There is no way to discover what can be restored — the UUID must come from outside the CLI.**
   No `deploy list` command exists (cli.rs:392-422 is the full deploy surface), restore
   never prompts or offers a picker (restore.rs:28), and marked-for-deletion state is
   only visible via `deploy status -d <id>` (status.rs:110) — which itself requires the
   id. A user who deleted a deployment without capturing the id from the delete output
   cannot recover it with this tool.

3. **[capability] No idempotent/no-op outcome when the deployment is not marked for deletion.**
   restore.rs:40-44 submits the activity blindly and any rejection surfaces as a raw
   `api_error`. The sibling `deploy approve` models the analogous already-done case as
   a first-class terminal outcome (`manifest_approval_already_posted`, outcome.rs:39),
   so scripted delete/restore flows get a friendly signal there but a generic API error
   here.

4. **[docs] Help text says "Restore a deleted deployment", but the command restores a deployment *marked for deletion*.**
   cli.rs:420 and restore.rs:17 both say "deleted deployment"; delete's own help is
   precise ("Delete a deployment by marking it for deletion", cli.rs:418) and the
   command's success message says "no longer marked for deletion" (restore.rs:79). The
   summary overstates what restore can do once the mark has been acted on.
