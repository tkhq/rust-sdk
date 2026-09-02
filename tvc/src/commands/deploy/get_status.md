# tvc deploy get-status

## Purpose

Fetches live runtime status for one deployment from the cluster's app-status API:
ready/desired replica counts, whether it is the app's targeted (live) deployment, egress
setting, and when its status last changed. Run it to check whether a deployment is
actually serving. Distinct from `deploy status`, which reports the deployment *record*
(manifest, approvals, debug mode); this command reports the *runtime* view.
Implementation: `tvc/src/commands/deploy/get_status.rs`, dispatched at
`tvc/src/cli.rs:245-247`.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| deployment id | `--deploy-id` / `-d` (UUID, clap-validated) | `TVC_DEPLOY_ID` | none | none — required | no |
| org id + API keys | none | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` (all-or-none trio) | active org in `~/.config/turnkey` (via `tvc login`) | none | no |
| API base URL | none | `TVC_API_BASE_URL` (only with the env trio) | per-org `api_base_url` | `https://api.turnkey.com` | no |

- Deviation from the documented flag > env > config > default order (`tvc/src/cli.rs:19-23`):
  the deployment id has no config-file (or saved-state) source at all — flag and env only
  (`get_status.rs:28-29`).
- Inherited globals (`--non-interactive`, `--message-format`, `--color`) get no special
  treatment; the command never inspects the ctx (`get_status.rs:34` takes `_ctx`).

## Interactive behavior

None. The command never prompts in any mode. A missing `--deploy-id` is a clap usage
error (exit 2, `usage_error` in JSON mode via `cli.rs:154-182`) — interactive mode gains
nothing over `--non-interactive`. A non-UUID value fails clap validation the same way
(unit-tested at `get_status.rs:168-173`).

## Outputs

Human mode (`Display` impl, `get_status.rs:92-131`):

```
Deployment: <id>
App ID: <id>
Egress Enabled: yes|no
Is Targeted Deployment: yes|no
Healthy / Desired Replicas: <ready>/<desired>
Last Updated: <seconds>.<nanos>s        (only when the API supplies a timestamp)
```

When the deployment is not present in the app's current status (e.g. not running), the
replica lines are replaced by:

```
Live Status: unavailable
Reason: deployment not present in current app status
```

JSON mode: one terminal NDJSON object, reason `deployment_runtime_status`
(`tvc/src/outcome.rs:41`), camelCase payload: `deploymentId`, `appId`, `egressEnabled`,
`isTargeted`, `replicas` (`{ready, desired}` or `null`), `lastUpdated`
(`{seconds, nanos}` strings or `null`) (`get_status.rs:80-90`).

## Side effects

Read-only against Turnkey; no local writes by the command itself (dispatch creates a
default `~/.config/turnkey` config if absent — all commands share that,
`cli.rs:219-223`). No YubiKey or key-file interaction. Three API calls per run:

1. `get_tvc_deployment` — resolves the deployment (and its `app_id`) (`get_status.rs:44`,
   `tvc/src/client.rs:83-100`)
2. `get_app_status` — the live status payload (`get_status.rs:51-55`)
3. `get_tvc_app` — fetched solely for `enable_egress` (`get_status.rs:62`)

`deploy-` prefixes on status-API deployment ids are stripped before comparison
(`sanitize_app_status`, `tvc/src/commands/app_status.rs:7-15`), so the `is_targeted`
check and the replica lookup match bare UUIDs.

## Failure modes

- Missing/malformed `--deploy-id`: clap usage error → `usage_error`, exit 2.
- Partial auth env trio: bail listing the missing vars (`client.rs:226-234`) →
  `command_error`, exit 1. No active org / no stored key: `command_error`, exit 1
  (`client.rs:104-117`).
- Deployment or app lookup returns empty: `MissingResource` → `not_found`, exit 1
  (`client.rs:97-99`, `tvc/src/errors.rs:95-96`).
- HTTP failures on any call classify by status: 401/403 `unauthorized`, 404 `not_found`,
  other `api_error`; transport failures `network_error`. Exit 1.
- `get_app_status` succeeds but `app_status` field is absent: plain
  `anyhow!("no status returned for app: …")` → `command_error`, exit 1
  (`get_status.rs:57-61`) — see Gap 2.
- Deployment absent from the app's status list is NOT an error: renders the
  "Live Status: unavailable" block and exits 0 (`get_status.rs:71-76,122-126`).

## Gaps

1. **[bug?] Human-mode `Last Updated` renders nanos wrong whenever they have fewer than
   9 digits.** `get_status.rs:117` formats the *string* nanos with `{:09}`, but the `0`
   flag is a no-op for strings — `format!("{:09}", "5")` yields `"5        "` (verified
   with rustc), so nanos `"5"` prints as `…​.5        s` (reads as half a second, plus
   trailing spaces) instead of `.000000005s`. `app status` already carries the fix,
   `{:0>9}` (`tvc/src/commands/app/status.rs:136`); `deploy status` has the same flaw
   (`tvc/src/commands/deploy/status.rs:185,189`). The shared `TimestampPayload`
   (`app_status.rs:19-32`) has no shared renderer, unlike replica counts
   (`format_replica_counts`, `app_status.rs:43-45`), which is how the two live-status
   commands drifted. JSON output is unaffected.

2. **[bug?] Empty `app_status` in a successful response classifies as `command_error`,
   but the documented taxonomy promises `not_found` for "a resource that resolved to
   empty".** `get_status.rs:57-61` uses a bare `anyhow!` where the sibling lookups in
   the same command use `MissingResource` (→ `not_found`, `errors.rs:95-96`); the
   LONG_ABOUT contract is at `cli.rs:58`. `app status` shares the flaw
   (`app/status.rs:49-53`), so JSON consumers cannot distinguish "app has no status yet"
   from arbitrary failures.

3. **[consistency] The reported `App ID` comes from the status API verbatim while
   deployment ids from the same response get prefix-sanitized.** `sanitize_app_status`
   strips only `deploy-` prefixes (`app_status.rs:7-15`); `get_status.rs:68` then
   reports `app_status.app_id` even though the command already holds the canonical
   `deployment.app_id` (which it uses for both follow-up requests,
   `get_status.rs:48,62`). The in-module fixture uses `app_id: "app-123"`
   (`get_status.rs:178`), suggesting status-API app ids are prefixed — if so, this
   command prints a differently-spelled app id than every other command.

4. **[capability] No interactive or stateful path to a deployment id — you must already
   know the UUID, in every mode.** Flag and env only (`get_status.rs:28`); no
   config-file key, no `last_deploy_id` analog of the saved `last_app_id` that `deploy
   create`/`deploy init` use as prompt defaults (`tvc/src/config/turnkey/mod.rs:699-710`),
   and no interactive list-and-select even though the app-status API enumerates all of an
   app's deployments. This is family-wide (no deploy command prompts for its id), but it
   means interactive mode is identical to CI mode for a command whose sibling data source
   could trivially offer a picker.

5. **[consistency] No exhaustive destructure of the status-API types, unlike `app
   status`.** `app/status.rs:56-76` destructures `AppStatus` and `DeploymentStatus`
   exhaustively "so a new field forces a decision"; `get_status.rs:64-76` uses field
   access on both, so a new upstream field would be silently dropped here while tripping
   a compile error in the sibling.

6. **[consistency] No integration test coverage.** `tvc/tests/` has a file per deploy
   command (approve, delete, provision, provisioning-details, debug-logs, restore,
   post-share…) but nothing for `deploy get-status` (nor `app status`); the only tests
   are in-module UUID parsing and the id-match helper (`get_status.rs:143-193`). The
   Display rendering — where Gap 1 lives — is untested.
