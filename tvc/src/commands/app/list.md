# tvc app list

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the app list command.*

## Purpose (informative)

`tvc app list` prints every TVC app in the authenticated org, with an optional client-side name filter.
Operators run it to discover app IDs, because every other `app` and `deploy` command addresses apps by `--app-id`.
The output also gives a quick view of quorum public keys and live deployment state across the org.

## Acceptance scenario (normative)

The scenario uses one org (alias `acme`, org id `7f8a9b0c-1d2e-4f3a-8b4c-5d6e7f8a9b0c`).
The org holds exactly one app: `payments-prod`, app id `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, with a live deployment and a public domain.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc login` and select org `acme`. | The tvc config stores `acme` as the active org with a stored API key. |
| 2 | Run `tvc app list`. | stdout carries one app block: `Name: payments-prod`, `ID: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, the remaining fields, then the separator line. Exit code 0. |
| 3 | Run `tvc app list --message-format json`. | stdout carries one NDJSON object with `reason` = `apps_listed` and one `apps` entry with `id` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. Exit code 0. |
| 4 | Run `tvc app list --name payments`. | The same app block as step 2. Exit code 0. |
| 5 | Run `tvc app list --name warehouse`. | stdout is `No apps found.`. Exit code 0. |

All steps pass in one run from a clean start, against an org that holds exactly the one app above.

## Inputs (normative)

| Input | Flag | Env | Config source | Default | Prompted |
|---|---|---|---|---|---|
| Name filter | `-n`, `--name` | `TVC_APP_NAME` | none | none (no filter) | never |
| Org and API key | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE` (all three or none) | active org in the tvc config | none | never |
| API base URL | none | `TVC_API_BASE_URL` (env auth path only) | active org `api_base_url` | `https://api.turnkey.com` | never |

The name filter MUST resolve in the Part 00 order: flag first, then environment variable (tvc/src/commands/app/list.rs:24).
The name filter has no config source and no default.

Auth deviates from per-value resolution.
The three auth environment variables MUST resolve as a set (tvc/src/client.rs:48-64).
When all three are set, they MUST win over the tvc config.
When only one or two are set, the command MUST fail (tvc/src/client.rs:226-234).
The command MUST NOT merge auth values across sources.
An empty environment variable counts as unset (tvc/src/client.rs:181-183).

When env auth is absent, the command MUST act on the active org (tvc/src/client.rs:104-106).
The command has no `--org` flag (Gap 1).
On the env auth path, `TVC_API_BASE_URL` MAY override the default API base URL (tvc/src/client.rs:197-198).
On the tvc config path, the command MUST use the active org's `api_base_url` (tvc/src/client.rs:121).

## Interactive behavior (normative)

The command MUST NOT prompt in any mode.
`run` ignores the interaction context (tvc/src/commands/app/list.rs:30).
Behavior MUST be identical in interactive mode, non-interactive mode, and JSON mode.
Non-interactive mode adds no hard requirements: the command collects no input through prompts.

## Outputs (normative)

### Human mode

For each app, the command MUST print one block with this exact field order (tvc/src/commands/app/list.rs:123-147):

```text
Name: payments-prod
ID: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b
Quorum Public Key: 02a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6
Live Deployment: 9d8c7b6a-5e4f-4a3b-9c2d-1e0f9a8b7c6d
Egress Enabled: yes
Debug Mode Deployments: yes
Public Domain: payments.example.com
────────────────────────────────────────
```

The `Live Deployment` value MUST read `(none)` when the app has no live deployment (tvc/src/commands/app/list.rs:125).
The command MUST omit the `Public Domain` line when the field is empty (tvc/src/commands/app/list.rs:141-143).
The command MUST end every block, including the last, with a separator of 40 U+2500 characters (tvc/src/commands/app/list.rs:145).
When the result list is empty, the command MUST print `No apps found.` and exit 0 (tvc/src/commands/app/list.rs:109-111).

### JSON mode

The command MUST emit one NDJSON object with `reason` = `apps_listed` (tvc/src/outcome.rs:52):

```json
{"reason":"apps_listed","apps":[{"id":"6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b","name":"payments-prod","quorumPublicKey":"02a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6","liveDeploymentId":"9d8c7b6a-5e4f-4a3b-9c2d-1e0f9a8b7c6d","egressEnabled":true,"debugModeDeploymentsEnabled":true,"publicDomain":"payments.example.com"}]}
```

App keys MUST use camelCase (tvc/src/commands/app/list.rs:63).
`liveDeploymentId` MUST be `null` when the app has no live deployment (tvc/src/commands/app/list.rs:68).
The object MUST omit `publicDomain` when the field is empty (tvc/src/commands/app/list.rs:72-73).
An empty result MUST serialize as `"apps": []` with exit code 0.

### Dropped API fields

The output MUST drop the API's `manifest_set`, `share_set`, `created_at`, `updated_at`, and `organization_id` fields (tvc/src/commands/app/list.rs:88-92).
The exhaustive destructure makes each drop a deliberate decision (INV-2, Gap 2).

## Side effects (normative)

- The command reads the tvc config. When the file is absent, the dispatcher MUST create a default one before the command runs (tvc/src/cli.rs:219-223, INV-G4). This is the only file write in a run.
- When env auth is absent, the command reads the active org's stored API key file (tvc/src/client.rs:115-117).
- The command MUST make exactly one API call, the read-only `get_tvc_apps` (tvc/src/commands/app/list.rs:33-39). The command MUST NOT submit an activity and MUST NOT touch a YubiKey.
- The name filter MUST run client side, after the fetch (tvc/src/commands/app/list.rs:43-54). The API request carries only the org id (tvc/src/commands/app/list.rs:35-37).

## Failure modes (normative)

| Condition | Message | JSON `code` | Exit code |
|---|---|---|---|
| One or two of the three auth env vars set | `partial env var auth: missing <names>. Set all three (TVC_ORG_ID, TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE) env vars or none.` (tvc/src/client.rs:226-234) | `command_error` | 1 |
| No active org in the tvc config | ``No active organization. Run `tvc login` first.`` (tvc/src/client.rs:104-106) | `command_error` | 1 |
| No stored API key for the active org | ``No API key found for org '<alias>'. Run `tvc login` first.`` (tvc/src/client.rs:115-117) | `command_error` | 1 |
| HTTP or network failure from `get_tvc_apps` | Chain starts with `failed to list TVC apps` (tvc/src/commands/app/list.rs:39) | Part 00 error taxonomy code (tvc/src/errors.rs:212-235) | 1 |
| Argument parse failure | clap usage text; one NDJSON object when the invocation requested JSON output (tvc/src/cli.rs:154-182) | `usage_error` | 2 |

Every runtime error object carries `reason` = `command_error`, except `missing_required_input` (tvc/src/output.rs:315-316). This command never raises `missing_required_input`.

The HTTP mapping follows the Part 00 error taxonomy (tvc/src/errors.rs:212-235).
Status 401 or 403 gives `unauthorized`, 404 gives `not_found`, and another non-success status gives `api_error` or `client_version_too_old`.
A connect, timeout, or DNS failure gives `network_error`.

A name filter that matches zero apps MUST succeed with exit code 0 and the empty-list output.
The command reserves `not_found` for HTTP 404 responses.

## Test vectors (normative)

Vector comparison excludes the Part 00 global nondeterministic fields.
These vectors name no additional exclusions.
Vectors V-1 through V-6 and V-9 through V-11 start from the acceptance scenario's logged-in tvc config.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | The org holds the one acceptance-scenario app, all fields set. | `tvc app list` | stdout equals the human-mode block in Outputs, ending with the separator line; exit code 0. Golden: tvc/src/commands/app/list.rs:271-283. |
| V-2 | The app has no live deployment and an empty public domain. | `tvc app list` | The block shows `Live Deployment: (none)` and no `Public Domain` line; exit code 0. Golden: tvc/src/commands/app/list.rs:286-297. |
| V-3 | Same as V-1. | `tvc app list --message-format json` | stdout is one NDJSON object equal to the JSON-mode example in Outputs; exit code 0 (tvc/src/commands/app/list.rs:56-74, tvc/src/outcome.rs:52). |
| V-4 | The org holds zero apps. | `tvc app list` | stdout is `No apps found.`; exit code 0 (tvc/src/commands/app/list.rs:109-111). |
| V-5 | The org holds apps `alpha` and `beta`. | `tvc app list --name gamma --message-format json` | stdout is `{"reason":"apps_listed","apps":[]}`; exit code 0 (tvc/src/commands/app/list.rs:197-201). |
| V-6 | The org holds `my-app-prod`, `my-app-staging`, and `other`. | `tvc app list --name my-app` | Exactly two blocks, `my-app-prod` and `my-app-staging`; exit code 0 (tvc/src/commands/app/list.rs:186-194). |
| V-7 | Only `TVC_ORG_ID=7f8a9b0c-1d2e-4f3a-8b4c-5d6e7f8a9b0c` set among the three auth env vars. | `tvc app list --message-format json` | One NDJSON object, `code` = `command_error`, `message` contains `partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE`; exit code 1 (tvc/src/client.rs:226-234). |
| V-8 | tvc config with no active org; no auth env vars. | `tvc app list --message-format json` | `code` = `command_error`, `message` contains ``No active organization. Run `tvc login` first.``; exit code 1 (tvc/src/client.rs:104-106). |
| V-9 | Active org `acme` present; its API key file absent. | `tvc app list` | Error output contains ``No API key found for org 'acme'. Run `tvc login` first.``; exit code 1 (tvc/src/client.rs:115-117). |
| V-10 | The server answers `get_tvc_apps` with HTTP 401. | `tvc app list --message-format json` | One NDJSON object, `code` = `unauthorized`, `httpStatus` = 401; exit code 1 (tvc/src/errors.rs:219). |
| V-11 | The API base URL points at a closed port. | `tvc app list --message-format json` | One NDJSON object, `code` = `network_error`; exit code 1 (tvc/src/errors.rs:229-235). |
| V-12 | Any tvc config. | `tvc app list --bogus --message-format json` | One NDJSON object, `code` = `usage_error`; exit code 2 (tvc/src/cli.rs:154-182; Part 00 GV-1). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT prompt in any mode. | `run` binds the interaction context as `_ctx` and calls no prompt (tvc/src/commands/app/list.rs:30). INV-G1 covers JSON mode. Behavioral check: V-1 through V-6 pass unchanged with `--non-interactive`. |
| INV-2 | A new field on the generated `TvcApp` MUST force a compile error in this command. | Exhaustive destructure with no `..` in `From<TvcApp> for AppSummary` (tvc/src/commands/app/list.rs:80-93). |
| INV-3 | The API request MUST carry only the org id; the name filter MUST NOT reach the server. | Request construction (tvc/src/commands/app/list.rs:35-37); `filter_by_name` mutates the fetched list (tvc/src/commands/app/list.rs:50-54). Behavioral check: filter unit tests (tvc/src/commands/app/list.rs:170-201). |
| INV-4 | An empty result MUST exit 0 in both output modes. | `run` returns `Ok` for an empty list (tvc/src/commands/app/list.rs:45-47); `Cli::run` maps `Ok` to exit code 0 (tvc/src/cli.rs:120). Behavioral check: V-4 and V-5. |

## Gaps (informative)

1. **[capability] The command lists apps for the active org only.** There is no `--org` flag. `build_client` knows the env var triple or the active org (tvc/src/client.rs:48-64, tvc/src/client.rs:104-106). To list another configured org's apps, the user re-runs `tvc login` or exports three env vars. Sibling `keys backup-operator-key` selects any configured profile with `--org` / `TVC_ORG` (tvc/src/commands/keys/backup_operator_key.rs:28-29 and :51-61).

2. **[capability] Manifest set, share set, and timestamps vanish after creation.** The list drops `manifest_set`, `share_set`, `created_at`, `updated_at` (tvc/src/commands/app/list.rs:88-92). `app create` prints manifest set IDs only at creation time (tvc/src/commands/app/create.rs:287-313). No `app get` style command exists, although `fetch_tvc_app` already wraps the single-app endpoint (tvc/src/client.rs:67-80). After creation, no CLI path shows an app's share set or age.

3. **[capability] The name substring is the only filter.** `filter_by_name` is a case-sensitive `contains` (tvc/src/commands/app/list.rs:50-54). Every sibling command addresses apps by `--app-id` / `TVC_APP_ID` (tvc/src/commands/app/status.rs:27, tvc/src/commands/app/delete.rs:21). The list offers no id filter and no exact-match mode, so no filter can isolate `app` from `app-2`.

4. **[consistency] An ambient `TVC_APP_NAME` silently narrows JSON output.** The filter has an env tier (tvc/src/commands/app/list.rs:24). A leftover `TVC_APP_NAME` in CI yields `{"reason":"apps_listed","apps":[]}` with exit code 0. That output equals the output for an org with zero apps. The documented flag-then-env order permits this. No other command consumes `TVC_APP_NAME`, so nothing warns that the variable is load bearing here.

5. **[docs] The help text omits the match semantics.** The doc comment `/// Filter by app name.` (tvc/src/commands/app/list.rs:23) reads as an exact match. The tests pin case-sensitive substring semantics (tvc/src/commands/app/list.rs:186-194). One added word ("substring") in the doc comment would fix it.

6. **[capability] No pagination, and the limit is upstream.** The API offers no page controls. `GetTvcAppsRequest` carries only `organization_id`, and the response is the bare full list (client/src/generated/services.coordinator.public.v1.rs:1112-1124). The CLI has nothing to expose: one call fetches the whole org. Recorded here so nobody files this against the CLI.
