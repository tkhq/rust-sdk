# tvc deploy get-status

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy get-status command.*

## Purpose (informative)

`tvc deploy get-status` reports the live runtime view of one deployment from the app status API.
The report carries ready and desired replica counts, the targeted flag, the app egress setting, and the last status change time.
Operators run it to check whether a deployment serves.
The sibling command `deploy status` reports the stored deployment record: manifest, approvals, debug mode.
This command reports the runtime view instead.
Implementation: tvc/src/commands/deploy/get_status.rs, dispatched from tvc/src/cli.rs:245-247.

## Acceptance scenario (normative)

The scenario universe, shared with the test vectors as universe U:

- An active org with a valid stored API key.
- App `9b8e6a4d-2c1f-4a3b-8d7e-5f0a1b2c3d4e` with `enable_egress` = `true`.
- Deployment `5376f492-d014-4e01-a6bb-20fc97448e25` of that app.
- The app status payload lists `deploy-5376f492-d014-4e01-a6bb-20fc97448e25` with `ready_replicas` 3, `desired_replicas` 3, and a timestamp of seconds `1756800000`, nanos `123456789`.
- The payload's `targeted_deployment_id` is the same prefixed id, and its `app_id` equals the app id above.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc deploy get-status --deploy-id 5376f492-d014-4e01-a6bb-20fc97448e25` | stdout equals the worked human example in Outputs; exit code 0. |
| 2 | Run the same invocation with `--message-format json` | stdout equals the worked JSON example in Outputs; exit code 0. |

All steps pass in one run from a clean start, after one `tvc login`.

## Inputs (normative)

| Input | Flag | Environment variable | Config source | Default | Prompt |
|---|---|---|---|---|---|
| deployment id | `--deploy-id`, `-d` (UUID) | `TVC_DEPLOY_ID` | none | none, required | never |
| org id and API key | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE` (all or none) | active org in the tvc config | none | never |
| API base URL | none | `TVC_API_BASE_URL` (env auth only) | `api_base_url` of the active org | `https://api.turnkey.com` | never |

- The command MUST resolve the deployment id from `--deploy-id` first, then from `TVC_DEPLOY_ID` (tvc/src/commands/deploy/get_status.rs:28).
- The deployment id has no tvc config source and no built-in default. The Part 00 resolution order applies to the two sources that exist (Gap 4).
- clap MUST reject a deployment id that does not parse as a UUID (get_status.rs:29; test at get_status.rs:168-173).
- When `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` all hold values, the command MUST authenticate from them (tvc/src/client.rs:48-64).
- When none of the three hold values, the command MUST authenticate from the active org's stored API key (client.rs:103-125).
- A partial trio MUST fail before any network call. Values never merge between the environment and the tvc config (client.rs:226-234).
- The command MUST read `TVC_API_BASE_URL` only with env auth, with `https://api.turnkey.com` as its default (client.rs:196-198). With stored auth the API base URL comes from the active org's `api_base_url` (client.rs:119-124).

## Interactive behavior (normative)

- The command MUST NOT prompt in any mode. `run` receives the ctx as `_ctx` and never reads it (get_status.rs:34).
- Interactive mode and non-interactive mode MUST produce the same observable behavior for equal inputs.
- A missing or malformed `--deploy-id` MUST surface as a usage error with exit code 2 in every mode (tvc/src/cli.rs:154-182). Interactive mode adds no fallback source.
- JSON mode implies non-interactive mode per INV-G1. This command has no prompts for that to disable.

## Outputs (normative)

**Human mode.** The command MUST print one status block (get_status.rs:92-131):

```
Deployment: <deployment id>
App ID: <app id>
Egress Enabled: yes|no
Is Targeted Deployment: yes|no
Healthy / Desired Replicas: <ready>/<desired>
Last Updated: <seconds>.<nanos>s
```

- The `Deployment` value MUST be the id from the deployment lookup (get_status.rs:67).
- The `App ID` value MUST be the status payload's `app_id`, verbatim (get_status.rs:68; Gap 3).
- The `Egress Enabled` line MUST come from the shared renderer (tvc/src/commands/display.rs:21).
- The replica line MUST render only when the app status lists the deployment (get_status.rs:106-112). It MUST use the shared renderer `format_replica_counts` (tvc/src/commands/app_status.rs:43-45).
- The `Last Updated` line MUST render only when the replica line renders and the status entry carries a timestamp (get_status.rs:114-119).
- The `Last Updated` format MUST render the nanos string left aligned in a nine character field, space padded (get_status.rs:117; Gap 1).
- When the app status does not list the deployment, the command MUST replace the last two lines with this block (get_status.rs:122-126):

```
Live Status: unavailable
Reason: deployment not present in current app status
```

Worked example, from the acceptance scenario universe:

```
Deployment: 5376f492-d014-4e01-a6bb-20fc97448e25
App ID: 9b8e6a4d-2c1f-4a3b-8d7e-5f0a1b2c3d4e
Egress Enabled: yes
Is Targeted Deployment: yes
Healthy / Desired Replicas: 3/3
Last Updated: 1756800000.123456789s
```

**JSON mode.** The command MUST emit exactly one terminal NDJSON object with `reason` = `deployment_runtime_status` (tvc/src/outcome.rs:41; get_status.rs:80-90).

| Field | Type | Source |
|---|---|---|
| `deploymentId` | string | the deployment lookup id (get_status.rs:67) |
| `appId` | string | the status payload's `app_id`, verbatim (get_status.rs:68) |
| `egressEnabled` | boolean | the app record's `enable_egress` (get_status.rs:69) |
| `isTargeted` | boolean | sanitized targeted id equals the requested id (get_status.rs:70) |
| `replicas` | `{ready, desired}` numbers, or `null` | the matching status entry (get_status.rs:71-74) |
| `lastUpdated` | `{seconds, nanos}` strings, or `null` | the entry's timestamp (get_status.rs:75-76; app_status.rs:21-24) |

`replicas` and `lastUpdated` MUST serialize as `null` when the app status does not list the deployment (get_status.rs:87-89).

Worked example, from the acceptance scenario universe:

```json
{"reason":"deployment_runtime_status","deploymentId":"5376f492-d014-4e01-a6bb-20fc97448e25","appId":"9b8e6a4d-2c1f-4a3b-8d7e-5f0a1b2c3d4e","egressEnabled":true,"isTargeted":true,"replicas":{"ready":3,"desired":3},"lastUpdated":{"seconds":"1756800000","nanos":"123456789"}}
```

## Side effects (normative)

- The command MUST NOT write to the tvc config or to any other local file. Dispatch creates the tvc config when it is absent, per INV-G4 (tvc/src/cli.rs:219-223).
- The command MUST NOT submit an activity. All three API calls read state.
- The command MUST make exactly three API calls per run, in this order (get_status.rs:44-62):
  1. `get_tvc_deployment` resolves the deployment and its `app_id` (get_status.rs:44; tvc/src/client.rs:82-100).
  2. `get_app_status` fetches the live status payload (get_status.rs:51-55).
  3. `get_tvc_app` fetches the app record; only `enable_egress` feeds the output (get_status.rs:62,69).
- Before comparison the command MUST strip the `deploy-` prefix from the status payload's targeted id and per entry deployment ids (app_status.rs:7-15). The targeted check and the replica lookup then compare bare UUIDs (get_status.rs:64,70).
- Stored auth reads the active org's API key files (client.rs:115-117). The command touches no YubiKey hardware.

## Failure modes (normative)

In JSON mode each failure MUST surface as one NDJSON error object per INV-G3.

| Condition | `code` | Exit code | Mechanism |
|---|---|---|---|
| `--deploy-id` absent or malformed | `usage_error` | 2 | clap validation; the JSON path exits through `handle_parse_error` (tvc/src/cli.rs:154-182) |
| One or two auth environment variables hold values | `command_error` | 1 | bail naming the missing variables (tvc/src/client.rs:226-234) |
| No active org, or no stored API key for it | `command_error` | 1 | client.rs:104-117 |
| Deployment lookup resolves to empty | `not_found` | 1 | `MissingResource` (client.rs:97-99; tvc/src/errors.rs:95-96) |
| App lookup resolves to empty | `not_found` | 1 | `MissingResource` (client.rs:77-79) |
| Successful status response with an absent `app_status` field | `command_error` | 1 | bare `anyhow!` (get_status.rs:57-61; Gap 2) |
| HTTP 401 or 403 on any call | `unauthorized` | 1 | errors.rs:219 |
| HTTP 404 on any call | `not_found` | 1 | errors.rs:220 |
| Other non-success HTTP status | `api_error` | 1 | errors.rs:221 |
| Transport failure (connect, timeout, DNS) | `network_error` | 1 | errors.rs:229-235 |

A deployment that the app status omits is a success. The command MUST render the unavailable output and exit with code 0 (get_status.rs:71-76,122-126).

## Test vectors (normative)

Universe U is the acceptance scenario setup. A vector's Given column states its deltas from U.
Vector comparison excludes the Part 00 global nondeterministic fields. U pins every timestamp, so `lastUpdated` compares exactly.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | U | `tvc deploy get-status --deploy-id 5376f492-d014-4e01-a6bb-20fc97448e25` | stdout equals the worked human example in Outputs; exit code 0 (get_status.rs:92-131; app_status.rs:43-45). |
| V-2 | U | V-1 plus `--message-format json` | stdout equals the worked JSON example in Outputs; exit code 0 (get_status.rs:80-90; outcome.rs:30,41). |
| V-3 | U, except the status payload lists only `deploy-0a1b2c3d-4e5f-6a7b-8c9d-0e1f2a3b4c5d` and targets it | V-1 invocation | The first four human lines with `Is Targeted Deployment: no`, then `Live Status: unavailable` and `Reason: deployment not present in current app status`; exit code 0 (get_status.rs:122-126). |
| V-4 | as V-3 | V-2 invocation | One JSON object with `isTargeted` = `false`, `replicas` = `null`, `lastUpdated` = `null`; exit code 0 (get_status.rs:71-76,87-89). |
| V-5 | U, except the timestamp nanos value is `5` | V-1 invocation | The last line is `Last Updated: 1756800000.5` followed by eight spaces, then `s`; exit code 0 (get_status.rs:117; Gap 1). |
| V-6 | any | `tvc deploy get-status --message-format json` with `TVC_DEPLOY_ID` unset | One JSON object with `reason` = `command_error`, `code` = `usage_error`, `message` carrying clap's rendered text; exit code 2 (cli.rs:160-176; tvc/src/output.rs:344-352). |
| V-7 | any | `tvc deploy get-status --deploy-id not-a-uuid` | clap value validation failure; exit code 2 (pinned by the test at get_status.rs:168-173). |
| V-8 | only `TVC_ORG_ID` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` holds a value | V-1 invocation | Error `partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE. Set all three (TVC_ORG_ID, TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE) env vars or none.`; `code` = `command_error`; exit code 1 (client.rs:226-234). |
| V-9 | no auth environment variables; tvc config without an active org | V-1 invocation | Error ``No active organization. Run `tvc login` first.``; `code` = `command_error`; exit code 1 (client.rs:104-106). |
| V-10 | U, except the deployment lookup returns an empty payload | V-1 invocation | Error `deployment not found: 5376f492-d014-4e01-a6bb-20fc97448e25`; `code` = `not_found`; exit code 1 (client.rs:97-99; errors.rs:95-96). |
| V-11 | U, except `get_app_status` succeeds with no `app_status` field | V-1 invocation | Error `no status returned for app: 9b8e6a4d-2c1f-4a3b-8d7e-5f0a1b2c3d4e`; `code` = `command_error`; exit code 1 (get_status.rs:57-61; Gap 2). |
| V-12 | U, except the API answers 401 on `get_tvc_deployment` | V-2 invocation | One JSON object with `reason` = `command_error`, `code` = `unauthorized`, `httpStatus` = 401, `message` starting `failed to fetch deployment 5376f492-d014-4e01-a6bb-20fc97448e25`; exit code 1 (client.rs:95; errors.rs:214-225). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST reject a non-UUID deployment id during argument parsing, before any config load or network call. | The `deploy_id` field is typed `Uuid`, so clap's value parser enforces the format (get_status.rs:28-29). Behavioral check: get_status.rs:168-173. |
| INV-2 | Deployment id comparison MUST use bare UUIDs on both sides. | `sanitize_app_status` strips `deploy-` prefixes before any comparison (get_status.rs:57-61; app_status.rs:7-15). Behavioral checks: app_status.rs:59-82; get_status.rs:175-192. |
| INV-3 | A deployment that the app status omits MUST NOT fail the command. | `find_deployment_status` returns `Option`; absence maps to `null` payload fields and the unavailable block, never an error (get_status.rs:64-77,122-126). Behavioral check: none (Gap 6). |
| INV-4 | The JSON `reason` MUST stay `deployment_runtime_status`. | Serde internal tagging derives the reason from the `Outcome` variant name, with no per-variant rename (outcome.rs:30,41). Behavioral check: the registry tests at outcome.rs:123-161. |

## Gaps (informative)

1. **[bug?] Human mode `Last Updated` renders nanos wrong whenever they have fewer than nine digits**.
   The write at `get_status.rs:117` formats the string nanos with `{:09}`, and the `0` flag is a no-op for strings.
   `format!("{:09}", "5")` yields `5` plus eight trailing spaces (verified with rustc), so nanos `5` reads as half a second.
   `app status` already carries the fix, `{:0>9}` (`tvc/src/commands/app/status.rs:136`). `deploy status` shares the flaw (`tvc/src/commands/deploy/status.rs:185,189`).
   The shared `TimestampPayload` (`app_status.rs:19-32`) has no shared renderer, unlike replica counts (`format_replica_counts`, `app_status.rs:43-45`). That asymmetry is how the two live status commands drifted.
   The flaw does not touch JSON output.

2. **[bug?] An empty `app_status` in a successful response classifies as `command_error`**.
   The documented taxonomy promises `not_found` for "a resource that resolved to empty" (`tvc/src/cli.rs:58`).
   `get_status.rs:57-61` uses a bare `anyhow!` where the sibling lookups in the same command use `MissingResource` (`errors.rs:95-96`).
   `app status` shares the flaw (`app/status.rs:49-53`). JSON consumers therefore cannot distinguish "app has no status yet" from arbitrary failures.

3. **[consistency] The reported `App ID` comes from the status API verbatim while deployment ids from the same response get prefix sanitization**.
   `sanitize_app_status` strips only `deploy-` prefixes (`app_status.rs:7-15`).
   `get_status.rs:68` then reports `app_status.app_id` even though the command already holds the canonical `deployment.app_id`.
   The command uses that canonical id for both follow-up requests (`get_status.rs:48,62`).
   The in-module fixture uses `app_id: "app-123"` (`get_status.rs:178`), which suggests status API app ids carry prefixes.
   If so, this command prints a differently spelled app id than every other command.

4. **[capability] No interactive or stateful path to a deployment id exists; the caller already knows the UUID, in every mode**.
   The sources are flag and environment variable only (`get_status.rs:28`).
   There is no config key and no `last_deploy_id` analog of the saved `last_app_id` (`tvc/src/config/turnkey.rs:699-710`).
   `deploy create` and `deploy init` use that saved value as a prompt default.
   There is also no interactive list and select flow, even though the app status API enumerates all deployments of an app.
   The whole family behaves this way: no deploy command prompts for its id.
   Interactive mode therefore equals CI mode for a command whose sibling data source could offer a picker.

5. **[consistency] The command reads the status API types by field access; `app status` destructures them exhaustively**.
   `app/status.rs:58-76` destructures `AppStatus` and `DeploymentStatus` so a new field forces a decision.
   `get_status.rs:64-76` uses field access on both.
   A new upstream field would vanish silently here while it trips a compile error in the sibling.

6. **[consistency] No integration test coverage**.
   `tvc/tests/` holds a file per deploy command (approve, delete, provision, provisioning details, debug logs, restore, post share) and nothing for `deploy get-status` or `app status`.
   The only tests are in-module: UUID parsing and the id match helper (`get_status.rs:143-193`).
   No test covers the `Display` rendering, where Gap 1 lives.
