# tvc deploy init

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy init command.*

## Purpose (informative)

`tvc deploy init` writes the command config that `tvc deploy create` consumes through `--config-file`.
The command offers three fill modes: a blank placeholder template (the default), a copy of an existing deployment (`--from-deployment`), and a prompt walk (`--interactive`).
Run it to start a new deployment or to clone the settings of an existing one.

Citation shorthand: `init.rs` is tvc/src/commands/deploy/init.rs, and `deploy.rs` is tvc/src/config/deploy.rs.
`create.rs`, `status.rs`, and `get_status.rs` are siblings of `init.rs`. Other citations carry their path under `tvc/` or `client/`.

## Acceptance scenario (normative)

The scenario starts clean: an empty working directory, no tvc config, and no `TVC_*` environment variables.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc deploy init --output deploy-demo.json`. | Exit code 0. The first stdout line is `Created deployment config template: deploy-demo.json` (init.rs:156). |
| 2 | Read the rest of stdout. | The block `Edit the file to fill in your values, then run:` names `tvc deploy create --config-file deploy-demo.json`, then the shared port guidance follows (init.rs:180-190; tvc/src/commands/deploy.rs:17-23). |
| 3 | Read `deploy-demo.json`. | Pretty printed JSON with exactly the 11 fields and values in the blank template column of the Inputs section. No `replicas` key (deploy.rs:124-139, 49-50). |
| 4 | Check `~/.config/turnkey/tvc.config.toml`. | The tvc config exists with defaults (INV-G4; tvc/src/cli.rs:219-223). |
| 5 | Run the same invocation again. | Exit code 1. The error is `File already exists: deploy-demo.json` (init.rs:78). |

All steps pass in one run from a clean start.

## Inputs (normative)

The command MUST resolve each input in the Part 00 value resolution order.

| Input | Flag | Env | tvc config source | Default | Prompted |
|---|---|---|---|---|---|
| Output path | `-o, --output <PATH>` | `TVC_DEPLOY_CONFIG_OUT` | none | `deploy-<YYYY-MM-DD-HHMMSS>.json`, local time (init.rs:71-74) | never |
| Seed deployment id | `--from-deployment <DEPLOY_ID>` | `TVC_FROM_DEPLOYMENT` | none | none: blank template (init.rs:88-98) | never |
| Prompt walk | `--interactive` | none (init.rs:46-47) | none | off | n/a |
| App id prefill | none | none | `last_created_app_id` for the active org (tvc/src/config/turnkey.rs:710-712) | `<FILL_IN_APP_ID>` (deploy.rs:126) | with `--interactive` only; the saved id is the prompt default (deploy.rs:150-151) |
| Credentials (`--from-deployment` only) | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE`, optional `TVC_API_BASE_URL` | active org credentials from `tvc login` (tvc/src/client.rs:48-64) | none | never |

The app id prefill has no flag and no environment variable, so the tvc config layer is its only source (init.rs:85, 98).
No command line input overrides it (see Gap 2).
When all three credential environment variables are set, they outrank the active org credentials (tvc/src/client.rs:51-61).

The output file follows the `DeployConfig` schema (deploy.rs:29-51).
The command MUST write the 11 fields below in this order and MUST NOT write a `replicas` key (deploy.rs:49-50, 116, 137).

| Field | Blank template (deploy.rs:124-139) | With `--from-deployment` (deploy.rs:69-118) |
|---|---|---|
| `appId` | saved app id, else `<FILL_IN_APP_ID>` | source `app_id` |
| `qosVersion` | `0.12.1` (deploy.rs:24) | source `qos_version` |
| `pivotContainerImageUrl` | `<FILL_IN_PIVOT_CONTAINER_IMAGE_URL>` | source `container_url` |
| `pivotPath` | `<FILL_IN_PIVOT_PATH>` | source `path` |
| `pivotArgs` | `[]` | source `args` |
| `expectedPivotDigest` | `<FILL_IN_EXPECTED_PIVOT_DIGEST>` | bare hex of the manifest `pivot_hash()` (deploy.rs:109) |
| `dangerousDeployDebugMode` | `false` | manifest `debug_mode()` (deploy.rs:110) |
| `pivotContainerEncryptedPullSecret` | `<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>` (deploy.rs:21, 133) | the same sentinel when the source has a pull secret, else `null` (deploy.rs:111-112) |
| `healthCheckType` | `TVC_HEALTH_CHECK_TYPE_HTTP` (deploy.rs:134) | source value |
| `healthCheckPort` | `3000` | source port |
| `publicIngressPort` | `3000` | source port |

## Interactive behavior (normative)

Without `--interactive` the command MUST NOT prompt, in interactive mode and in non-interactive mode alike (INV-1).
It writes the output file and exits in both fill modes.

The `--interactive` flag requests the prompt walk and MUST pass a gate first (init.rs:53-59).

- In non-interactive mode the command MUST fail with `--interactive conflicts with --non-interactive or TVC_NON_INTERACTIVE=true` (tvc/src/prompts.rs:29-31).
- JSON mode implies non-interactive mode (INV-G1), so `--interactive` MUST fail there with the same error.
- In human mode without the non-interactive flag, stdin MUST be a TTY. A piped stdin fails with `--interactive requires a TTY` (tvc/src/prompts.rs:33-38).

The prompt walk MUST prompt only for fields that still hold a `<FILL_IN...>` placeholder (deploy.rs:145-165).
The prompt order is: App ID, QOS version, pivot container image URL, pivot path, expected pivot digest.
The App ID prompt offers the saved app id as its default (deploy.rs:150-151).
The digest prompt carries the hint `(sha256:...)` (deploy.rs:163-165).
When the pull secret sentinel is present, the command asks the confirm `Is the container image in a public registry?`, default yes (deploy.rs:167-168).
Either answer MUST clear the field (deploy.rs:169).
An answer of no also prints a note to pass `--pivot-pull-secret` to `tvc deploy create` (deploy.rs:170-176).
An empty answer to a text prompt fails the command with `<prompt> cannot be empty` (tvc/src/prompts.rs:42-48).

With `--from-deployment` every placeholder field arrives filled, so at most the pull secret confirm fires (deploy.rs:388-395).
The walk never asks about ports, `healthCheckType`, `pivotArgs`, debug mode, or `replicas` (see Gap 3).

## Outputs (normative)

Human mode (init.rs:151-192):

- The first line MUST be `Created deployment config template: <path>` without `--interactive`, and `Created deployment config: <path>` with it (init.rs:153-157).
- With `--from-deployment` the seeded guidance block MUST follow. It states that `expectedPivotDigest` and debug mode came from the source manifest, and that an image change requires a digest recompute (init.rs:128-133, 163-164).
- When the written file still holds the pull secret sentinel, a `--from-deployment` run MUST add the pull secret guidance (init.rs:135-138, 166-168). Blank template runs MUST NOT print this block, even though the sentinel is present (init.rs:163).
- The output MUST end with the next step and the shared port guidance (tvc/src/commands/deploy.rs:17-23). Runs with `--interactive` print `Run: tvc deploy create --config-file <path>`; template runs print `Edit the file to fill in your values, then run:` with the same command (init.rs:171-190).

JSON mode: the command MUST emit exactly one NDJSON object with `reason` = `deployment_config_created` (tvc/src/outcome.rs:30, 47).
The camelCase fields are `command` (`deploy init`), `path`, `template`, `interactive`, `fromDeployment`, and `needsPullSecret` (init.rs:140-149).
`template` MUST equal the negation of `interactive` (init.rs:121).
JSON mode forbids `--interactive` (see above), so `template` is always true there.
`needsPullSecret` MUST be true exactly when the written file holds the pull secret sentinel (init.rs:109; deploy.rs:214-216).

## Side effects (normative)

- The command MUST create the tvc config with defaults when it is absent (INV-G4; tvc/src/cli.rs:219-223). It reads the saved app id (tvc/src/config/turnkey.rs:710-712) and MUST NOT change any tvc config field. A malformed tvc config fails the command before dispatch (test tvc/tests/non_interactive.rs:376).
- The command MUST write the output file as pretty printed JSON (init.rs:111-116) and MUST NOT overwrite an existing file (INV-2).
- With `--from-deployment` the command builds an authenticated client (init.rs:93; tvc/src/client.rs:48-64). It sends one `get_tvc_deployment` request for the active org (init.rs:95; tvc/src/client.rs:83-100). It decodes the QOS manifest locally to recover the digest and debug mode (deploy.rs:82-85, 109-110).
- The command MUST NOT submit an activity and MUST NOT touch a YubiKey (INV-4).

## Failure modes (normative)

A failed run MUST exit with code 1 and, in JSON mode, MUST emit one error object per INV-G3.
Usage errors exit with code 2 (Part 00 exit codes).

| Condition | Error | `code` | Source |
|---|---|---|---|
| Output file exists | `File already exists: <path>` | `command_error` | init.rs:77-79; fallback classification tvc/src/errors.rs:102 |
| `--interactive` in non-interactive mode | `--interactive conflicts with --non-interactive or TVC_NON_INTERACTIVE=true` | `command_error` | tvc/src/prompts.rs:29-31; tests tvc/tests/non_interactive.rs:166, 184 |
| `--interactive`, human mode, piped stdin | `--interactive requires a TTY` | `command_error` | tvc/src/prompts.rs:33-38 |
| Empty answer to a required prompt | `<prompt> cannot be empty` | `command_error` | tvc/src/prompts.rs:42-48 |
| `--from-deployment`, no active org, no credential env vars | ``No active organization. Run `tvc login` first.`` | `command_error` | tvc/src/client.rs:106 |
| `--from-deployment`, partial credential env vars | `partial env var auth: missing <names>. Set all three (...) env vars or none.` | `command_error` | tvc/src/client.rs:226-233 |
| HTTP failure on the fetch | classified from `TurnkeyClientError` | `unauthorized`, `not_found`, `api_error`, or `network_error` | tvc/src/errors.rs:98-99 |
| Fetch succeeds with an empty deployment payload | `MissingResource` for `deployment` | `not_found` | tvc/src/client.rs:99; tvc/src/errors.rs:95-96 |
| Source deployment lacks a manifest or a container spec, or the manifest fails to decode | `deployment <id> has no manifest to seed from`, `failed to decode QOS manifest for deployment <id>`, or `deployment <id> has no pivot container spec` | `command_error` | deploy.rs:82-96; tests deploy.rs:426-460 |
| Source port exceeds `u16` | `deployment health check port does not fit in u16` or the ingress twin | `command_error` | deploy.rs:98-101; test deploy.rs:433-441 |
| Serialize or write failure | `failed to serialize config` or `failed to write file: <path>` | `command_error` | init.rs:111-116 |
| Unknown flag or malformed usage | clap usage error | `usage_error`, exit 2 | Part 00 exit codes; GV-1 |

## Test vectors (normative)

Vector comparison excludes the Part 00 global nondeterministic fields plus the timestamped default output filename (V-3).

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Empty directory, no tvc config | `tvc deploy init --output deploy.json` | Exit 0. stdout contains `Created deployment config template: deploy.json` and no `"reason"` key (test tvc/tests/message_format.rs:7-24). |
| V-2 | Empty directory, no tvc config | `tvc --message-format=json deploy init --output deploy.json` | Exit 0. Exactly one NDJSON object: `reason` = `deployment_config_created`, `command` = `deploy init`, `path` = `deploy.json`, `template` = `true`, `interactive` = `false`, `fromDeployment` = `false`, `needsPullSecret` = `true` (test tvc/tests/message_format.rs:27-57). |
| V-3 | Empty directory, no tvc config | `tvc deploy init` | Exit 0. A file `deploy-<YYYY-MM-DD-HHMMSS>.json` appears in the working directory; the timestamp is excluded from comparison (init.rs:71-74). |
| V-4 | tvc config with active org `acme` and `last_created_app_id.acme` = `651b573c-861b-4f10-a478-cbcfe0c226af` | `tvc deploy init --output deploy.json` | Exit 0. The file's `appId` is `651b573c-861b-4f10-a478-cbcfe0c226af`; every other field matches the blank template column (init.rs:85, 98; deploy.rs:126). |
| V-5 | `deploy.json` already exists | `tvc deploy init --output deploy.json` | Exit 1. Error `File already exists: deploy.json`; in JSON mode `code` = `command_error` (init.rs:78; tvc/src/errors.rs:102). |
| V-6 | Fresh HOME | `tvc deploy init --interactive --non-interactive` | Exit 1. Error `--interactive conflicts with --non-interactive or TVC_NON_INTERACTIVE=true` (test tvc/tests/non_interactive.rs:184; env variant at 166). |
| V-7 | Fresh HOME | `tvc --message-format=json deploy init --interactive` | Exit 1. One NDJSON error object with `code` = `command_error` and the same conflict message (INV-G1; tvc/src/prompts.rs:30). |
| V-8 | Fresh HOME, human mode | `tvc deploy init --interactive < /dev/null` | Exit 1. Error `--interactive requires a TTY` (init.rs:57; tvc/src/prompts.rs:33-38). |
| V-9 | No active org, no credential env vars | `tvc deploy init --from-deployment 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | Exit 1. Error ``No active organization. Run `tvc login` first.``; `code` = `command_error` (tvc/src/client.rs:106). |
| V-10 | Active org with valid credentials; deployment `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` has image `ghcr.io/x/y@sha256:img`, path `/bin/pivot`, args `["--flag", "value"]`, ports 8080 and 9090, a decodable manifest, no pull secret | `tvc deploy init --from-deployment 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --output seeded.json` | Exit 0. The file copies each source field; `expectedPivotDigest` is the bare hex `pivot_hash()`; `pivotContainerEncryptedPullSecret` is `null`; stdout shows the seeded guidance block (unit tests deploy.rs:375-395, 417-423; init.rs:128-133). |
| V-11 | Fetch returns success with no deployment payload | the V-10 invocation | Exit 1. `code` = `not_found` (tvc/src/client.rs:99; tvc/src/errors.rs:95-96). |
| V-12 | Source deployment manifest bytes fail to decode | the V-10 invocation | Exit 1. Error `failed to decode QOS manifest for deployment <id>`; `code` = `command_error` (deploy.rs:84-85; test deploy.rs:451-460). |

## Invariants (normative)

Global invariants INV-G1 through INV-G4 apply.

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | Without `--interactive` the command MUST NOT prompt, in any mode. | Every prompt call site sits behind the `args.interactive` gate (init.rs:53-59, 105-107). Behavioral check: tvc/tests/message_format.rs:7-24 runs with piped stdin. |
| INV-2 | The command MUST NOT overwrite an existing file at the output path. | The existence check bails before any fetch, prompt, or write (init.rs:77-79). |
| INV-3 | Blank template mode MUST NOT read credentials and MUST NOT touch the network. | Client construction and the fetch live only in the `Some(deploy_id)` match arm (init.rs:88-98). Behavioral check: tvc/tests/message_format.rs:7-57 passes with an empty HOME. |
| INV-4 | The command MUST NOT submit an activity. | The only API call is the `get_tvc_deployment` read (init.rs:93-96; tvc/src/client.rs:83-100). |

## Gaps (informative)

1. **[capability] `deploy create` honors `replicas` from the command config and `deploy init` never emits it, in any mode**. The field carries `skip_serializing_if = "Option::is_none"` (deploy.rs:49-50). Both `DeployConfig::template` (deploy.rs:137) and `TryFrom<TvcDeployment>` (deploy.rs:116) set `None`, so neither the blank template nor a seeded config ever contains the key. Meanwhile `create` reads it from the file and forwards it to the intent (create.rs:320-322, 382; test create.rs:748-752). A user who edits the generated file cannot discover the field. Every other `DeployConfig` field appears in the generated file.

2. **[consistency] Per-machine state silently prefills the blank template's `appId`, with no marker and no explicit override**. `init` has no `--app-id` flag; `create` has one (create.rs:106). The last created app id for the active org (tvc/src/config/turnkey.rs:710-712) replaces the `<FILL_IN_APP_ID>` placeholder (init.rs:85; deploy.rs:126). The last used app id can differ. The generated file then looks complete for possibly the wrong app. `create` accepts it: the checks are UUID shape and app existence only (deploy.rs:225; create.rs:412). Saved state beats explicit choice here, and no flag can make the explicit choice.

3. **[capability] The prompt walk fills only the five placeholder fields plus the pull secret question; ports, `healthCheckType`, `pivotArgs`, debug mode, and `replicas` stay unreachable**. `fill_interactively` (deploy.rs:145-179) never asks about them, and `init` has no flags for them. An interactive user still hand-edits the filled config for anything beyond the defaults. The template hard codes `healthCheckType` to HTTP (deploy.rs:134) even though `TVC_HEALTH_CHECK_TYPE_GRPC` exists (client/src/generated/immutable.common.v1.rs:1359). No command exposes a flag for it; hand-editing the JSON is the only path.

4. **[consistency] `--from-deployment` takes a raw `String` while every sibling deploy command parses the deployment id as `Uuid` at the CLI boundary**. Compare init.rs:42 with `deploy status` and `deploy get-status` (`pub deploy_id: Uuid`, status.rs:30; get_status.rs:29). A malformed id costs an auth round trip plus an API call and exits 1 as an API error. A `Uuid` field would reject it at parse time with exit 2.

5. **[consistency] The two fill paths produce different digest formats for the same field**. `--from-deployment` writes bare hex (`hex::encode(manifest.pivot_hash())`, deploy.rs:109). The interactive prompt hints `Expected pivot digest (sha256:...)` (deploy.rs:163-165). `create` never validates or normalizes the field, so one of the two conventions misleads.

6. **[consistency] `--interactive` combined with `--from-deployment` on a private-image source loses the needs-pull-secret signal in the final output**. The pull secret confirm clears the sentinel for either answer (deploy.rs:167-169). The fill runs before the `needs_pull_secret` check (init.rs:105-109), so the pull secret guidance never prints (init.rs:166-168) and JSON `needsPullSecret` reads false. The only trace is a transient note printed mid walk (deploy.rs:170-176).

7. **[docs] `LONG_ABOUT` claims that `--from-deployment` copies every field from that deployment (init.rs:22-25)**. The seeded file omits `replicas`. The API deployment type carries no desired replica count, so `TryFrom` leaves the field unset (deploy.rs:116; test deploy.rs:401-405). The help text names only the pull secret as unrecoverable.

8. **[docs] A stale comment says `--from-deployment` deliberately leaves the expected pivot digest blank; the code copies it**. Compare init.rs:101-102 with deploy.rs:109 and with `FROM_DEPLOYMENT_GUIDANCE` (init.rs:128-133). The guidance correctly says the digest and debug mode come from the source manifest.

9. **[consistency] No `--force` or overwrite escape hatch exists, and sibling init defaults diverge**. An existing output file bails as a generic `command_error` (init.rs:77-79), the same as `app init` (tvc/src/commands/app/init.rs:47-50). Deleting the stale file is the only non-interactive way to regenerate it. `deploy init` defaults to a timestamped name while `app init` defaults to a fixed `app.json` (tvc/src/commands/app/init.rs:23). The collision behavior between the two siblings differs for no documented reason.
