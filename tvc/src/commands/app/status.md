# tvc app status

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the app status command.*

## Purpose (informative)

`tvc app status` reads the live cluster state for one app.
It reports the targeted deployment, ready and desired replica counts per deployment, each deployment's last update time, and the app egress setting.
Operators run it to check rollout health after `tvc deploy create` or `tvc app set-live-deploy`.

## Acceptance scenario (normative)

The scenario universe: app id `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`.
Its status lists one deployment with API id `deploy-5376f492-d014-4e01-a6bb-20fc97448e25`, 3 of 3 replicas ready, and egress disabled.
The deployment's last update time carries seconds `1723480000` and nanos `123456`.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Export `TVC_ORG_ID=1f8e0d7c-2a4b-4c6d-9e8f-3a5b7c9d1e2f`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` with valid credentials for that org. | No file under `~/.config/turnkey/` is needed for auth; env auth wins (tvc/src/client.rs:48). |
| 2 | Run `tvc app status --app-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | stdout shows the human block from Outputs, including `Healthy / Desired Replicas: 3/3` and `Last Updated: 1723480000.000123456s`; no prompt; exit code 0. |
| 3 | Run the same invocation with `--message-format json` appended. | stdout carries exactly one NDJSON object with `reason` = `app_status` matching the JSON example in Outputs; exit code 0. |

Pass criterion: all steps pass in one run against one org whose app reports one healthy deployment.

## Inputs (normative)

| Input | Flag | Env | tvc config | Default |
|---|---|---|---|---|
| App id | `-a`, `--app-id` (UUID) | `TVC_APP_ID` | none | none; clap requires the value |
| Org id and API key | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE` | active org plus its stored API key file | none |
| API base URL | none | `TVC_API_BASE_URL` (env auth path only) | the active org's `api_base_url` | `https://api.turnkey.com` |

The app id MUST resolve flag first, then environment variable (tvc/src/commands/app/status.rs:27).
The app id resolution stops there: the tvc config holds no app id key for this command, and no built-in default exists.

Authentication MUST NOT merge env and tvc config sources per value.
When all three auth variables carry values, the command MUST authenticate from them (tvc/src/client.rs:48).
When none carry values, the command MUST authenticate from the active org and its stored API key file (tvc/src/client.rs:103).
A partial set of one or two variables MUST fail and name the missing variables (tvc/src/client.rs:227).

On the env auth path, `TVC_API_BASE_URL` MAY replace the default `https://api.turnkey.com` (tvc/src/client.rs:24).
On the tvc config path, the command MUST use the active org's `api_base_url` (tvc/src/client.rs:121).

## Interactive behavior (normative)

The command MUST NOT prompt in any mode (INV-1).
`run` binds the prompt context as `_ctx` and never uses it (tvc/src/commands/app/status.rs:33).
Interactive mode and non-interactive mode behave identically; `--non-interactive` changes nothing.

A run without `--app-id` and without `TVC_APP_ID` MUST fail as a clap parse error with exit code 2.
In JSON mode that parse failure MUST surface as one NDJSON object with `reason` = `command_error` and `code` = `usage_error` (tvc/src/cli.rs:154).
The test tvc/tests/error_output.rs:135 pins this shape for a sibling command.

## Outputs (normative)

### Human mode

The command MUST print this block (tvc/src/commands/app/status.rs:107), shown with the acceptance scenario values:

```
App ID: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b
Targeted Deployment: 5376f492-d014-4e01-a6bb-20fc97448e25
Egress Enabled: no

Deployment: 5376f492-d014-4e01-a6bb-20fc97448e25
  Healthy / Desired Replicas: 3/3
  Last Updated: 1723480000.000123456s
```

- The `App ID` value MUST echo the API response payload without transformation (tvc/src/commands/app/status.rs:58).
- The command MUST repeat the deployment block once per deployment, in API response order.
- The command MUST print `No deployments found.` in place of the deployment blocks when the deployment list is empty (tvc/src/commands/app/status.rs:120).
- The command MUST omit the `Last Updated` line when the API sends no timestamp (tvc/src/commands/app/status.rs:133).
- The `Last Updated` value MUST render as `<seconds>.<nanos>s`, with nanos zero padded on the left to nine characters (tvc/src/commands/app/status.rs:136).
- The egress line MUST render as `Egress Enabled: yes` or `Egress Enabled: no` (tvc/src/commands/display.rs:21).
- Deployment ids MUST appear without the API `deploy-` prefix (INV-3; tvc/src/commands/app_status.rs:7). This applies to the targeted id and each deployment block, so ids match `tvc deploy` output.

### JSON mode

The command MUST emit exactly one terminal NDJSON object with `reason` = `app_status` (tvc/src/outcome.rs:51).
Payload keys use camelCase (tvc/src/commands/app/status.rs:91).
With the acceptance scenario values, the object is:

```json
{"reason":"app_status","appId":"6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b","targetedDeploymentId":"5376f492-d014-4e01-a6bb-20fc97448e25","egressEnabled":false,"deployments":[{"deploymentId":"5376f492-d014-4e01-a6bb-20fc97448e25","replicas":{"ready":3,"desired":3},"lastUpdated":{"seconds":"1723480000","nanos":"123456"}}]}
```

- `lastUpdated` MUST serialize as `null` when the API sends no timestamp (tvc/src/commands/app/status.rs:83).
- `seconds` and `nanos` MUST stay stringified integers that mirror the API `Timestamp` (tvc/src/commands/app_status.rs:21).
- JSON mode carries the raw `nanos` string; only human mode pads it.

## Side effects (normative)

- The command MUST load the tvc config before it runs (INV-G4). The dispatcher creates the file with defaults when the file does not exist (tvc/src/cli.rs:219). That creation is the only local write on this path.
- On the tvc config auth path, the command MUST read the active org's stored API key file (tvc/src/client.rs:115).
- The command MUST issue exactly two read-only API calls in a fixed order. `get_app_status` runs first (tvc/src/commands/app/status.rs:43), then `get_tvc_app` (tvc/src/commands/app/status.rs:54), which supplies `enable_egress` only.
- The command MUST NOT submit an activity and MUST NOT touch a YubiKey (INV-2).

## Failure modes (normative)

A failed run MUST exit with the listed exit code.
In JSON mode the error object MUST carry the listed `code` value (INV-G3).

| Failure | Message and mechanism | JSON `code` | Exit code |
|---|---|---|---|
| Missing or non-UUID app id | clap parse error (tvc/src/commands/app/status.rs:27) | `usage_error` | 2 |
| Partial env auth | "partial env var auth: missing ..." names the missing variables (tvc/src/client.rs:227) | `command_error` | 1 |
| No active org | "No active organization. Run `tvc login` first." (tvc/src/client.rs:106) | `command_error` | 1 |
| No stored API key for the active org | "No API key found for org '<alias>'. Run `tvc login` first." (tvc/src/client.rs:117) | `command_error` | 1 |
| HTTP 401 or 403 from either API call | classification of the typed client error (tvc/src/errors.rs:219) | `unauthorized` | 1 |
| HTTP 404 from either API call | tvc/src/errors.rs:220 | `not_found` | 1 |
| Other non-success HTTP status | tvc/src/errors.rs:221 | `api_error` | 1 |
| Connect, timeout, or DNS failure | tvc/src/errors.rs:229 | `network_error` | 1 |
| 200 response with empty `app_status` payload | "no status returned for app: <id>" through a bare `anyhow!` (tvc/src/commands/app/status.rs:52); see gap 3 | `command_error` | 1 |
| 200 response with empty `tvc_app` payload | `MissingResource::new("app", <id>)` (tvc/src/client.rs:79) | `not_found` | 1 |

## Test vectors (normative)

Vector comparison excludes the global nondeterministic fields (Part 00).
These vectors add no per-command exclusions: the fixture pins `lastUpdated`.
The vectors reuse the acceptance scenario universe; "the fixture" below names its API responses.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Env auth set; the fixture returns one deployment `deploy-5376f492-d014-4e01-a6bb-20fc97448e25`, ready 3, desired 3, time `1723480000`/`123456`; `enable_egress` false | `tvc app status --app-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | The exact human block in Outputs; both ids appear without the `deploy-` prefix (tvc/src/commands/app_status.rs:60); exit code 0 (tvc/src/commands/app/status.rs:107). |
| V-2 | Same as V-1 | V-1 invocation plus `--message-format json` | The exact NDJSON object in Outputs; exit code 0 (tvc/src/outcome.rs:51). |
| V-3 | `get_app_status` returns zero deployments and an empty `targeted_deployment_id` | V-1 invocation | `No deployments found.` replaces the deployment blocks (tvc/src/commands/app/status.rs:120); the `Targeted Deployment:` line shows an empty value (gap 5); exit code 0. |
| V-4 | Any auth; `TVC_APP_ID` unset | `tvc app status` | clap usage error on stderr; exit code 2 (tvc/src/commands/app/status.rs:27). |
| V-5 | Any auth; `TVC_APP_ID` unset | `tvc app status --message-format json` | One NDJSON object on stdout, `reason` = `command_error`, `code` = `usage_error`; exit code 2 (tvc/src/cli.rs:154; tvc/tests/error_output.rs:135). |
| V-6 | Any auth | `tvc app status --app-id not-a-uuid` | clap value parse error; exit code 2 (tvc/src/commands/app/status.rs:28). |
| V-7 | Only `TVC_ORG_ID` and `TVC_API_KEY_PUBLIC` set | V-1 invocation | Error text contains `partial env var auth` and `TVC_API_KEY_PRIVATE`; exit code 1 (tvc/src/client.rs:227; tvc/tests/auth_env.rs:54). |
| V-8 | No auth env vars; tvc config has no active org | V-1 invocation | "No active organization. Run `tvc login` first."; `code` = `command_error`; exit code 1 (tvc/src/client.rs:106). |
| V-9 | `get_app_status` answers 200 with no `app_status` payload | V-1 invocation | Error "no status returned for app: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b"; `code` = `command_error`; exit code 1 (tvc/src/commands/app/status.rs:52). |
| V-10 | `get_app_status` succeeds; `get_tvc_app` answers 200 with no `tvc_app` payload | V-1 invocation | `code` = `not_found` from `MissingResource::new("app", ...)`; exit code 1 (tvc/src/client.rs:79). |
| V-11 | `get_app_status` answers HTTP 401 | V-1 invocation plus `--message-format json` | `code` = `unauthorized`, `httpStatus` = 401; exit code 1 (tvc/src/errors.rs:219). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT prompt in any mode. | `run` binds the prompt context as `_ctx` and never calls it (tvc/src/commands/app/status.rs:33). INV-G1 covers JSON mode. Behavioral check: tvc/tests/auth_env.rs:35 runs the command without a TTY. |
| INV-2 | The command MUST NOT change server state. | `run` calls only `get_app_status` and `fetch_tvc_app`, both reads (tvc/src/commands/app/status.rs:43, tvc/src/commands/app/status.rs:54). |
| INV-3 | Output deployment ids MUST NOT carry the API `deploy-` prefix. | `sanitize_app_status` strips the prefix from the targeted id and every deployment id (tvc/src/commands/app_status.rs:7). Behavioral check: tvc/src/commands/app_status.rs:60. |
| INV-4 | A new field on the API `AppStatus` or `Timestamp` MUST force a compile-time decision in this command. | Exhaustive destructures with no `..` (tvc/src/commands/app/status.rs:58, tvc/src/commands/app_status.rs:29). |
| INV-5 | Human mode and JSON mode MUST render the same report value. | Both modes consume the one `AppStatusReport` inside `Outcome::AppStatus` (tvc/src/outcome.rs:51): `Display` for human mode (tvc/src/commands/app/status.rs:107), `Serialize` for JSON mode (tvc/src/commands/app/status.rs:90). |

## Gaps (informative)

1. **[capability] App selection accepts only the flag or the env var: no prompt, no picker, no tvc config fallback.**

   `LONG_ABOUT` promises that commands can prompt when stdin is a TTY (tvc/src/cli.rs:41).
   `app status` never prompts (tvc/src/commands/app/status.rs:33), and clap requires `--app-id`.
   An interactive user without the UUID at hand gets a usage error.
   The resolution machinery exists elsewhere.
   `deploy create` and `deploy init` prompt with the last used app id (tvc/src/commands/deploy/create.rs:202, tvc/src/commands/deploy/init.rs:85).
   `app create` records that id (tvc/src/commands/app/create.rs:272), and `app list` fetches the org's apps.
   None of that machinery is reachable from this command.

2. **[consistency] The three status commands format the shared `TimestampPayload` three different ways, and only this command's way is correct.**

   `app status` pads nanos with `{:0>9}` (tvc/src/commands/app/status.rs:136), which left pads the string with zeros ("123456" becomes "000123456").
   `deploy get-status` (tvc/src/commands/deploy/get_status.rs:117) and `deploy status` (tvc/src/commands/deploy/status.rs:185) use `{:09}`.
   The `0` flag has no effect on the `String` nanos field (client/src/generated/external.data.v1.rs:9).
   The value left aligns with trailing spaces ("123456" becomes "123456   "), a wrong fractional value.
   Executing both format specs confirmed the difference.
   The fix belongs in the siblings.
   The shared helper module formats replica counts and no timestamps (tvc/src/commands/app_status.rs:43), which is how the drift happened.

3. **[bug?] An empty `app_status` payload classifies as `command_error`; the documented taxonomy assigns `not_found` to a resource that resolved to empty.**

   The `None` payload maps through a bare `anyhow!` (tvc/src/commands/app/status.rs:52), which `classify` cannot downcast (tvc/src/errors.rs:93).
   `LONG_ABOUT` defines `not_found` as "HTTP 404, or a resource that resolved to empty" (tvc/src/cli.rs:58).
   The repo convention prescribes `MissingResource` for exactly this shape; the app fetch two lines later uses it (tvc/src/client.rs:79).
   `deploy get-status` shares the defect (tvc/src/commands/deploy/get_status.rs:60), so the two commands agree with each other and both sit off taxonomy.

4. **[consistency] The app existence check runs after the status fetch, so an unknown app id can surface as the gap 3 `command_error`.**

   `get_app_status` runs first and `fetch_tvc_app` second (tvc/src/commands/app/status.rs:43, tvc/src/commands/app/status.rs:54).
   Suppose the backend answers the status call for an unknown app with an empty payload and no 404.
   The user then sees "no status returned for app" with `command_error`.
   `deploy get-status` resolves its resource first (tvc/src/commands/deploy/get_status.rs:44), so unknown deployment ids classify as `not_found` cleanly.

5. **[consistency] An app with no targeted deployment renders a blank value.**

   `targeted_deployment_id` is a plain `String` on the generated `AppStatus` (client/src/generated/external.data.v1.rs:839).
   When the value is empty, human mode prints `Targeted Deployment: ` with nothing after the colon (tvc/src/commands/app/status.rs:111).
   JSON mode carries `""`.
   `app list` renders the comparable absence as `(none)` (tvc/src/commands/app/list.rs:125), and `deploy get-status` models targeting as a boolean.

6. **[consistency] `TVC_APP_ID` parses as `Uuid` here and as `String` in `deploy create`, so the same env var validates inconsistently.**

   Compare tvc/src/commands/app/status.rs:27 with tvc/src/commands/deploy/create.rs:106.
   `deploy create` accepts a non-UUID value and defers to the API.
   The same value fails at parse time in `app status` and `app delete` (tvc/src/commands/app/delete.rs:21).
   The mismatch is minor and only bites if app ids are ever non-UUID.
