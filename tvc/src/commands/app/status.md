# tvc app status

## Purpose

Read-only snapshot of what the cluster is actually running for one app: which
deployment is targeted, per-deployment ready/desired replica counts with a
last-updated timestamp, plus the app's egress setting. Run it to check rollout
health after `deploy create` / `app set-live-deploy`.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| app id | `-a`, `--app-id` (parsed as `Uuid`) | `TVC_APP_ID` | — | — (clap-required) | no |
| org id + API key | — | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE` (all three or none) | active org in `~/.config/turnkey/tvc.config.toml` | — | no |
| API base URL | — | `TVC_API_BASE_URL` (env-auth path only) | `orgs.<alias>.api_base_url` | `https://api.turnkey.com` | no |

Deviations from flag > env > config > default:

- App id stops at the env tier: no config-file key, no last-used fallback, no
  prompt (tvc/src/commands/app/status.rs:25-29).
- Auth is all-env-or-all-config, not merged per-value: a partial env set (1-2 of
  the three vars) errors instead of falling back
  (tvc/src/client.rs:225-234).

## Interactive behavior

None. The command never prompts — `ctx` is bound as `_ctx` and unused
(tvc/src/commands/app/status.rs:33). A missing `--app-id`/`TVC_APP_ID` is a clap
parse failure in both modes, so interactive and non-interactive behavior are
identical; `--non-interactive` and JSON mode change nothing except error
rendering. In JSON mode the parse failure is emitted as one NDJSON line
(`reason: command_error`, `code: usage_error`) with exit 2
(tvc/src/cli.rs:154-182).

## Outputs

Human mode (tvc/src/commands/app/status.rs:107-145):

```
App ID: <uuid>
Targeted Deployment: <uuid>
Egress Enabled: yes|no

Deployment: <uuid>
  Healthy / Desired Replicas: <ready>/<desired>
  Last Updated: <epoch-seconds>.<nanos zero-padded to 9>s
```

Repeated per deployment; `Last Updated` omitted when the API sends no
timestamp; `No deployments found.` replaces the deployment blocks when the list
is empty. Deployment ids (targeted and per-deployment) have the API's
`deploy-` prefix stripped so they match `tvc deploy` ids
(tvc/src/commands/app_status.rs:7-15,47-52).

JSON mode: one terminal NDJSON object, `reason: "app_status"`
(tvc/src/outcome.rs:51), camelCase payload:

```json
{"reason":"app_status","appId":"…","targetedDeploymentId":"…","egressEnabled":false,
 "deployments":[{"deploymentId":"…","replicas":{"ready":3,"desired":3},
 "lastUpdated":{"seconds":"1723…","nanos":"123456"}}]}
```

`lastUpdated` is `null` when absent; `seconds`/`nanos` are stringified integers
mirroring the API `Timestamp` (tvc/src/commands/app_status.rs:19-32).

## Side effects

- Reads `~/.config/turnkey/tvc.config.toml`; if the file is missing the shared
  dispatcher creates it with defaults before the command runs
  (tvc/src/cli.rs:215-224) — the only write on this path.
- Config-auth path reads the active org's stored API key file
  (tvc/src/client.rs:103-125).
- Two read-only Turnkey API calls, in order: `get_app_status`, then
  `get_tvc_app` (for `enable_egress` only)
  (tvc/src/commands/app/status.rs:38-54). No activities submitted, no YubiKey
  interaction.

## Failure modes

- Missing or non-UUID `--app-id`: clap usage error, exit 2 (`usage_error` in
  JSON mode).
- Partial env auth / no active org / no stored API key: plain anyhow errors →
  `command_error`, exit 1 (tvc/src/client.rs:106,117,227).
- HTTP failures from either API call: `unauthorized` (401/403), `not_found`
  (404), `api_error` (other statuses), `network_error` (transport)
  (tvc/src/errors.rs:212-243), exit 1.
- 200 response with empty `app_status` payload: `anyhow!("no status returned
  for app: …")` → `command_error`, exit 1 (tvc/src/commands/app/status.rs:49-52;
  see gap 3).
- `get_tvc_app` 200 with empty payload: `MissingResource("app", …)` →
  `not_found`, exit 1 (tvc/src/client.rs:77-80).

## Gaps

1. **[capability] App selection is flag/env-only: no prompt, no picker, no
   config or last-used fallback — despite the CLI promising TTY prompts.**
   `LONG_ABOUT` says "commands may prompt when stdin is a TTY"
   (tvc/src/cli.rs:41-42), but `app status` never prompts (`_ctx` unused,
   tvc/src/commands/app/status.rs:33) and `--app-id` is clap-required, so an
   interactive user without the uuid at hand gets a usage error instead of a
   selector. The machinery for better resolution already exists elsewhere:
   `deploy create`/`deploy init` prompt and offer the last-used app id
   (tvc/src/commands/deploy/create.rs:202-203, tvc/src/commands/deploy/init.rs:85),
   `app create` records it (`Config::set_last_app_id`,
   tvc/src/commands/app/create.rs:272), and `app list` fetches the org's apps —
   none of it reachable from this command.

2. **[consistency] The three status commands render the same
   `TimestampPayload` three different ways, and only this command's is
   correct.** `app status` pads nanos with `{:0>9}`
   (tvc/src/commands/app/status.rs:136), which left-pads the string
   ("123456" → "000123456"); `deploy get-status`
   (tvc/src/commands/deploy/get_status.rs:117) and `deploy status`
   (tvc/src/commands/deploy/status.rs:185-189) use `{:09}`, whose `0` flag is
   ignored for the `String` nanos field
   (client/src/generated/external.data.v1.rs:5-9) and instead left-aligns with
   trailing spaces ("123456" → "123456&nbsp;&nbsp;&nbsp;"), i.e. a wrong fractional value —
   verified by executing both format specs. The fix belongs in the siblings,
   but the shared helper module (tvc/src/commands/app_status.rs) formats
   replica counts and not timestamps, which is how the drift happened.

3. **[bug?] An empty `app_status` payload classifies as `command_error`, but
   the documented taxonomy says a resource that resolved to empty is
   `not_found`.** tvc/src/commands/app/status.rs:49-52 maps `None` through a
   bare `anyhow!`, which `classify` cannot downcast
   (tvc/src/errors.rs:93-102), while `LONG_ABOUT` defines `not_found` as
   "HTTP 404, or a resource that resolved to empty" (tvc/src/cli.rs:59) and the
   repo's own convention prescribes `MissingResource` for exactly this shape
   (used one line later for the app fetch, tvc/src/client.rs:77-80).
   `deploy get-status` shares the defect
   (tvc/src/commands/deploy/get_status.rs:58-61), so the two are mutually
   consistent but both off-taxonomy.

4. **[consistency] App existence is checked after the status fetch, so an
   unknown app id can surface as the gap-3 `command_error` instead of
   `not_found`.** `get_app_status` runs first and `fetch_tvc_app` second
   (tvc/src/commands/app/status.rs:38-54); if the backend answers the status
   call for an unknown app with an empty payload rather than a 404, the user
   sees "no status returned for app" / `command_error`. `deploy get-status`
   resolves its resource first (`fetch_tvc_deployment`,
   tvc/src/commands/deploy/get_status.rs:44) so unknown deployment ids classify
   `not_found` cleanly.

5. **[consistency] An app with no targeted deployment renders a blank value.**
   `targeted_deployment_id` is a plain `String` on the generated `AppStatus`
   (client/src/generated/external.data.v1.rs:835-840); when empty, human output
   prints `Targeted Deployment: ` with nothing after the colon and JSON carries
   `""` (tvc/src/commands/app/status.rs:109-117). `app list` renders the
   comparable absence as `(none)` (tvc/src/commands/app/list.rs:125) and
   `deploy get-status` models targeting as a boolean.

6. **[consistency] `TVC_APP_ID` is typed `Uuid` here but `String` in
   `deploy create`, so the same env var is validated inconsistently.**
   tvc/src/commands/app/status.rs:27-28 vs
   tvc/src/commands/deploy/create.rs:105-106 — a non-UUID value accepted by
   `deploy create` (which defers to the API) is rejected at parse time by
   `app status` and `app delete` (tvc/src/commands/app/delete.rs:21-22). Minor,
   only bites if app ids are ever non-UUID.
