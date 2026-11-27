# tvc app create

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the app create command.*

## Purpose (informative)

`tvc app create` submits one `createTvcApp` activity built from a command config, normally the output of `tvc app init`.
On success the command records the new app ID and manifest set operator IDs in the tvc config.
Later commands (`tvc deploy init`, `tvc deploy create`) read those values as defaults.
Run it once per app, after `tvc login` or with CI env var auth.

## Acceptance scenario (normative)

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc login` and save the profile `acme-dev`. | The tvc config lists `acme-dev` as the active org with a stored API key. |
| 2 | Run `tvc app init --output app.json`. | `app.json` holds the placeholder template (tvc/src/commands/app/init.rs:56). |
| 3 | Edit `app.json`: set `name` to `payments-app`, `manifestSetParams.name` to `payments-set`, and the operator public key to a valid qos composite hex key. | `app.json` validates with zero placeholders (tvc/src/config/app.rs:160-211). |
| 4 | Run `tvc app create --config-file app.json`. | stdout prints `Creating app 'payments-app'...`, then the `App created successfully!` block with App ID, Name, Manifest Set ID, Manifest Set Operator IDs, and Config path; exit code 0 (tvc/src/commands/app/create.rs:252, 295-325). |
| 5 | Open the tvc config. | `last_created_app_id` and `last_operator_ids` for `acme-dev` hold the returned IDs (tvc/src/commands/app/create.rs:272-274). |

All steps pass in one run from a clean start against a live Turnkey org.

## Inputs (normative)

| Input | Flag | Env var | Command config key | Default | Prompted |
|---|---|---|---|---|---|
| command config path | `-c`, `--config-file` | `TVC_APP_CONFIG` | (none) | none; the argument is required (tvc/src/commands/app/create.rs:31-33) | no |
| operator reuse opt-out | `--no-operator-reuse` | `TVC_NO_OPERATOR_REUSE` | (none) | false | no |
| debug-mode deployments | `--dangerous-enable-debug-mode-deployments` | `TVC_DANGEROUS_ENABLE_DEBUG_MODE_DEPLOYMENTS` | `dangerousEnableDebugModeDeployments` | false | no |
| app name | (none) | (none) | `name` | placeholder `<FILL_IN_APP_NAME>` | yes, on placeholder |
| quorum public key | (none) | (none) | `quorumPublicKey` | `KNOWN_QUORUM_KEY`, the well-known insecure dev key (tvc/src/config/app.rs:65-66) | never |
| enable egress | (none) | (none) | `enableEgress` | false | never |
| manifest set ID | (none) | (none) | `manifestSetId` | null | never |
| manifest set params | (none) | (none) | `manifestSetParams` | template: threshold 1, one new operator with the saved or placeholder key (tvc/src/config/app.rs:76-86) | name and operator keys, on placeholder |
| share set ID | (none) | (none) | `shareSetId` | null | never |
| share set params | (none) | (none) | `shareSetParams` | null; the intent build substitutes `dev-known-share-set`, threshold 2, two well-known keys (tvc/src/config/app.rs:213-223) | name and keys, on placeholder |
| auth | (none) | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE`, all or none | (none) | the active org's stored API key (tvc/src/client.rs:38-64) | no |

The three flags MUST follow the Part 00 value resolution order.
The debug-mode flag MUST follow the Part 00 debug-mode exception (tvc/src/commands/app/create.rs:345-351).
The flag or its env var only turns the field on.
A command config value of `true` survives an absent flag.
Env var auth takes precedence over the active org's stored API key (tvc/src/client.rs:38-64).
The command MUST resolve every other input from the command config alone (tvc/src/commands/app/create.rs:30-56).
No flag or env var layer exists for them.
`tvc app init` serializes every field this command reads (tvc/src/config/app.rs:70-91, tvc/src/commands/app/init.rs:56).
The template therefore covers every input.

## Interactive behavior (normative)

The command checks non-interactive mode alone at the mode split (tvc/src/commands/app/create.rs:60).
The reuse selection prompt adds a stdin TTY check (tvc/src/commands/app/create.rs:83).

In interactive mode the command MUST run these steps in this order.

1. Read the command config from `--config-file` (tvc/src/commands/app/create.rs:182).
   The command MUST fall back to a fresh placeholder template on any read failure (tvc/src/commands/app/create.rs:182-185; Gap 5).
   This includes a missing file.
   A parse failure MUST end the command with an error (tvc/src/commands/app/create.rs:183, 226-229).
2. Validate the command config (tvc/src/config/app.rs:160-211).
   An error other than a placeholder MUST end the command (tvc/src/commands/app/create.rs:191-192).
   Placeholder errors start the fill walk.
3. Fill placeholders in prompt order: App name, Manifest set name, `Operator '<name>' public key` per operator, Share set name, share keys (tvc/src/config/app.rs:114-141).
   The operator key prompt offers a default: the active org's default operator public key (tvc/src/operator.rs:282-304).
   That default read is best effort (tvc/src/commands/app/create.rs:196-197).
   The command MUST prompt only for values that start with `<FILL_IN`.
   The command MUST NOT prompt for `quorumPublicKey`, `enableEgress`, thresholds, `manifestSetId`, `shareSetId`, or debug mode.
4. When any prompt filled a value, ask `Save filled config to <path>?`, default yes (tvc/src/commands/app/create.rs:202-204, 235-244).
   A yes answer writes the pretty-printed command config back to the same path.
5. Decide operator reuse after the fill and before the API call (tvc/src/commands/app/create.rs:71-96).
   Candidates come from the tvc config: hosted operator records with proven keys, plus saved bare IDs without keys (tvc/src/operator.rs:448-475).
   Reuse applies only when the command config requests exactly one new operator and pins no existing IDs (tvc/src/commands/app/create.rs:127-141).
   `--no-operator-reuse` disables reuse entirely (tvc/src/commands/app/create.rs:123-125).
   A sole candidate whose proven key matches wins automatic reuse; the command announces it (tvc/src/commands/app/create.rs:151, 164-167).
   Several matching candidates prompt `Select operator to reuse` (tvc/src/commands/app/create.rs:93).
   With piped stdin the command MUST fail with the remediation message (tvc/src/commands/app/create.rs:83-91).

In non-interactive mode the command config MUST exist, parse, and validate with zero placeholders (tvc/src/commands/app/create.rs:209-218).
Multiple matching reuse candidates MUST end the command with the remediation message (tvc/src/commands/app/create.rs:85-91).
The message names both escape hatches: set `manifestSetParams.existingOperatorIds`, or pass `--no-operator-reuse`.
A sole matching candidate still triggers reuse; JSON mode suppresses the announcement (tvc/src/output.rs:242-254).

## Outputs (normative)

In human mode the command MUST print, in this order:

1. `Wrote <path>`, when the save-back ran (tvc/src/commands/app/create.rs:241).
2. `Reusing operator <candidate> (pass --no-operator-reuse to create a new one)`, when reuse fired (tvc/src/commands/app/create.rs:164-167).
   A candidate renders as `<name> (<id>)`, or as the bare ID when it has no registry name (tvc/src/operator.rs:225-232).
3. `Creating app '<name>'...` (tvc/src/commands/app/create.rs:252).
4. The terminal block (tvc/src/commands/app/create.rs:295-325):

   ```
   App created successfully!

   App ID: <appId>
   Name: <name>
   Manifest Set ID: <manifestSetId>
   Manifest Set Operator IDs: <id>, <id>
   Config: <configPath>

   Use one of the Manifest Set Operator IDs above with `tvc deploy approve --operator-id`
   ```

   The block starts with one blank line.
   The Manifest Set Operator IDs line MUST vanish when the list is empty (tvc/src/commands/app/create.rs:308-314).

In JSON mode a successful run MUST emit exactly one NDJSON object with `reason` = `app_created` (tvc/src/outcome.rs:30, 53).
The object carries `appId`, `name`, `manifestSetId`, `manifestSetOperatorIds`, and `configPath` (tvc/src/commands/app/create.rs:285-293).
JSON mode MUST suppress every progress line above, including the reuse announcement (tvc/src/output.rs:242-254).
A failed run follows INV-G3 and the Part 00 error taxonomy.

## Side effects (normative)

- The command reads the command config and MAY rewrite it through the interactive save-back (tvc/src/commands/app/create.rs:235-244).
- Dispatch loads the tvc config and creates a default file when absent (INV-G4).
- After creation the command MUST write `last_created_app_id` and `last_operator_ids` for the active org (tvc/src/commands/app/create.rs:272-274).
  Both setters need an active org (tvc/src/config/turnkey.rs:699-724).
- The prompt default read is best effort and touches no device (tvc/src/operator.rs:282-304).
  It reads the local key file, the hosted record's stored keys, or the YubiKey registry's cached key.
- The command submits exactly one `create_tvc_app` activity (tvc/src/commands/app/create.rs:263-267).
  Auth uses the env var triple or the active org's stored API key (tvc/src/client.rs:38-64).
- The command performs no YubiKey device interaction.

## Failure modes (normative)

| Failure | Observation |
|---|---|
| `--config-file` absent and `TVC_APP_CONFIG` unset | clap usage error; exit code 2; `code` = `usage_error` in JSON mode (tvc/src/commands/app/create.rs:31-33, tvc/src/cli.rs:154-176). |
| unreadable or unparseable command config, non-interactive mode | error `failed to read config file: <path>` or `failed to parse config file: <path>`; `code` = `command_error`; exit code 1 (tvc/src/commands/app/create.rs:209-229). |
| command config with a validation error other than a placeholder, both modes | error `invalid config file: <path>: <errors>`; `code` = `command_error`; exit code 1 (tvc/src/commands/app/create.rs:231-233). |
| multiple matching reuse candidates without a usable prompt | error `multiple operator IDs use the requested manifest operator key; ...`; `code` = `command_error`; exit code 1 (tvc/src/commands/app/create.rs:85-91). |
| no active org at bookkeeping time | error `no active organization set` after the app exists; `code` = `command_error`; exit code 1 (tvc/src/config/turnkey.rs:699-707; Gap 1). |
| API failure | `code` = `unauthorized`, `not_found`, `api_error`, `network_error`, or `client_version_too_old` per the typed chain (tvc/src/errors.rs:93-103). |

The command MUST NOT emit `missing_required_input` or `invalid_input` (tvc/src/errors.rs:54-56; Gap 6).
Validation failures classify as `command_error` today; Gap 6 proposes `invalid_input`.

## Test vectors (normative)

Every vector runs in a fresh `HOME` with only the stated fixtures.
Vector comparison excludes the Part 00 nondeterministic fields: here `appId`, `manifestSetId`, and the `manifestSetOperatorIds` values.

| Fixture | Definition |
|---|---|
| valid `app.json` | `name` = `test-app`, `quorumPublicKey` = `KNOWN_QUORUM_KEY`, `manifestSetParams`: name `manifest-set`, threshold 1, one new operator `operator-1` (tvc/tests/app_create.rs:89-107). |
| two-candidate tvc config | Active org `hosted-org`, hosted operator records `hosted-op` (`11111111-1111-4111-8111-111111111111`) and `hosted-op-2` (`33333333-3333-4333-8333-333333333333`) sharing one proven key, plus saved bare ID `66666666-6666-4666-8666-666666666666` (tvc/tests/app_create.rs:19-77). |
| one-candidate tvc config | The same config with the `hosted-op-2` operator record removed. |

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Active org with a stored API key; valid `app.json` | `tvc app create --config-file app.json` | stdout: `Creating app 'test-app'...`, then the `App created successfully!` block; exit code 0 (tvc/src/commands/app/create.rs:252, 295-325). |
| V-2 | Same as V-1 | `tvc app create --config-file app.json --message-format json` | stdout: one NDJSON object, `reason` = `app_created`, keys `appId`, `name`, `manifestSetId`, `manifestSetOperatorIds`, `configPath`; no progress lines; exit code 0 (tvc/src/outcome.rs:53, tvc/src/output.rs:242-254). |
| V-3 | No fixtures; `TVC_APP_CONFIG` unset | `tvc app create --message-format json` | stdout: one JSON object with `code` = `usage_error`; exit code 2 (tvc/src/commands/app/create.rs:31-33, tvc/src/cli.rs:154-176). |
| V-4 | No file at `/tmp/absent-app.json` | `tvc app create --config-file /tmp/absent-app.json --non-interactive` | error contains `failed to read config file: /tmp/absent-app.json`; `code` = `command_error`; exit code 1 (tvc/src/commands/app/create.rs:209-211, 220-224). |
| V-5 | `app.json` is the untouched `tvc app init` template | `tvc app create --config-file app.json --non-interactive` | error contains `invalid config file:` and `name contains placeholder value <FILL_IN_APP_NAME>`; exit code 1 (tvc/src/commands/app/create.rs:213-215, tvc/src/config/app.rs:163-167, 255). |
| V-6 | valid `app.json` plus `manifestSetId` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | `tvc app create --config-file app.json` (either mode) | error contains `Cannot specify both manifestSetId and manifestSetParams`; exit code 1 (tvc/src/config/app.rs:178-183, 266-270; tvc/src/commands/app/create.rs:191-192). |
| V-7 | two-candidate tvc config; `app.json` requests one new operator with the shared key; piped stdin | `tvc app create --config-file app.json` | stderr contains `multiple operator IDs use the requested manifest operator key`; exit code 1 (tvc/src/commands/app/create.rs:85-91; tvc/tests/app_create.rs:83-123). |
| V-8 | two-candidate tvc config; `app.json` pins `existingOperatorIds` = `["33333333-3333-4333-8333-333333333333"]`; piped stdin | `tvc app create --config-file app.json` | stdout contains `Creating app 'test-app'`; stderr lacks the multiple-candidates message (tvc/src/commands/app/create.rs:132-134; tvc/tests/app_create.rs:131-170). |
| V-9 | two-candidate tvc config; `app.json` as in V-7; piped stdin | `tvc app create --config-file app.json --no-operator-reuse` | stdout contains `Creating app 'test-app'` and no `Reusing operator` line; stderr lacks the multiple-candidates message (tvc/src/commands/app/create.rs:123-125; tvc/tests/app_create.rs:176-219). |
| V-10 | two-candidate tvc config; `app.json` requests key `07` repeated 130 times, matching no candidate | `tvc app create --config-file app.json` | stdout has no `Reusing operator` line; `newOperators` reaches the intent unchanged (tvc/src/commands/app/create.rs:143-150; tvc/tests/app_create.rs:225-269). |
| V-11 | one-candidate tvc config; `app.json` requests one new operator with `hosted-op`'s proven key | `tvc app create --config-file app.json` | stdout contains `Reusing operator hosted-op (11111111-1111-4111-8111-111111111111) (pass --no-operator-reuse to create a new one)`; the intent carries that ID in `existingOperatorIds` and zero `newOperators` (tvc/src/commands/app/create.rs:149-151, 157-175; unit tests tvc/src/commands/app/create.rs:657-664, 684-692). |
| V-12 | valid `app.json` with `dangerousEnableDebugModeDeployments` = `true`; flag and env var absent | `tvc app create --config-file app.json` | the intent carries `enableDebugModeDeployments` = `true` (tvc/src/commands/app/create.rs:345-351; unit test tvc/src/commands/app/create.rs:480-489). |

## Invariants (normative)

The Part 00 global invariants apply.
This table adds per-command invariants.

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The debug-mode override MUST only turn the field on; a command config `true` survives an absent flag. | `apply_overrides` writes only when the flag is true (tvc/src/commands/app/create.rs:345-351). Behavioral check: tvc/src/commands/app/create.rs:480-489. |
| INV-2 | Reuse MUST fire only for a candidate whose proven key equals the sole requested new operator key. | Key equality filter in `decide_operator_reuse` (tvc/src/commands/app/create.rs:143-147); saved bare IDs carry no key (tvc/src/operator.rs:219-222). Behavioral check: tvc/tests/app_create.rs:225-269. |
| INV-3 | An explicit `manifestSetParams.existingOperatorIds` MUST pass to the intent unchanged. | Early return in `decide_operator_reuse` (tvc/src/commands/app/create.rs:132-134). Behavioral check: tvc/tests/app_create.rs:131-170. |
| INV-4 | The reuse selection prompt MUST NOT run without a TTY on stdin or in non-interactive mode. | The prompt fence bails first (tvc/src/commands/app/create.rs:83-91); INV-G1 covers JSON mode. Behavioral check: tvc/tests/app_create.rs:83-123. |
| INV-5 | With `shareSetId` and `shareSetParams` both null, the intent MUST carry the `dev-known-share-set` params. | `AppConfig::effective_share_set_params` substitutes them (tvc/src/config/app.rs:213-223). Behavioral check: tvc/src/commands/app/create.rs:399-407. |

## Gaps (informative)

1. **[bug?]** Under pure env var CI auth, the command creates the app, then reports failure and never emits the app ID.
   `build_client` accepts auth from `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` alone, without `tvc login` (tvc/src/client.rs:38-64; tvc/src/cli.rs:34-38).
   After a successful `create_tvc_app` call, `set_last_app_id` fails when no active org exists (tvc/src/commands/app/create.rs:272, tvc/src/config/turnkey.rs:699-707).
   The failure message is `no active organization set` (tvc/src/config/turnkey.rs:703).
   The default tvc config that dispatch writes on first run has no active org (tvc/src/cli.rs:219-223).
   The run exits 1 with no `app_created` outcome, and the caller loses the app ID.
   Any post-create `config.save()` failure masks success the same way.
   `tvc/tests/auth_env.rs` covers only the read-only `app status` under env auth.

2. **[capability]** Operator reuse matches hosted operator records only; registered local and YubiKey identities never become candidates.
   `known_operator_candidates` returns hosted registry records with proven keys, plus bare last-app-create IDs that deliberately carry `public_key: None` (tvc/src/operator.rs:448-475, 219-222).
   `decide_operator_reuse` matches only candidates with a proven key (tvc/src/commands/app/create.rs:143-147).
   Bare IDs therefore stay inert, and reuse fires exclusively for hosted operator records.
   A local operator record can carry an `operator_id` and a loadable key file (tvc/src/config/turnkey.rs:340-347).
   The YubiKey registry caches its key (tvc/src/operator.rs:298-302).
   Neither becomes a candidate.
   The common local flow that reruns `app create` with the same key silently mints a duplicate server-side operator, unless the user hand-edits `existingOperatorIds`.
   The tests acknowledge the limitation (tvc/tests/app_create.rs:221-223).
   Registry state constrains which operator identity the user's explicit key can resolve to.

3. **[capability]** Neither `app init` nor the interactive fill offers a secure quorum key choice; the well-known insecure dev key applies silently.
   The template hardcodes `KNOWN_QUORUM_KEY`, meant for applications that need no secure quorum key (tvc/src/config/app.rs:65-66, 73).
   `fill_interactively` never prompts for `quorumPublicKey` (tvc/src/config/app.rs:114-141).
   Both `app init --interactive` and the `app create` fill walk therefore create the app on the shared dev quorum key without surfacing the choice.
   The outputs of `keys create-quorum-key` and `keys generate-local-quorum-key` reach the command config only through hand-edited JSON.
   The same silent-default pattern covers the hardcoded `dev-known-share-set` that substitutes for a null `shareSetParams` (tvc/src/config/app.rs:213-223).
   Human mode output never says that the run used the well-known share keys.

4. **[consistency]** Only debug mode gets a flag and env var override; `deploy create` overrides every field and makes its file optional.
   `deploy create` pairs each config field with a flag and env var, for example `--app-id` with `TVC_APP_ID` (tvc/src/commands/deploy/create.rs:90-152).
   It also supports flag-only creation with `--config-file` omitted (tvc/src/commands/deploy/create.rs:73-75).
   `app create` requires the file and offers no `--name`, `--quorum-public-key`, `--enable-egress`, `--manifest-set-id`, `--share-set-id`, or threshold overrides (tvc/src/commands/app/create.rs:30-56).
   CI pipelines therefore template JSON to vary anything except debug mode.
   A minor adjunct: `app init` defaults its output to `app.json` (tvc/src/commands/app/init.rs:23), and `app create --config-file` has no default.
   The advertised init to create pairing still requires retyping the path.

5. **[bug?]** Interactive mode swallows every command config read error, well beyond a missing file.
   The read fallback treats every read failure as the start of a fresh template walk (tvc/src/commands/app/create.rs:182-185).
   Permission denied and is-a-directory failures on an existing file get the same treatment as a missing file.
   The command then offers to save over the same path.
   A mistyped or unreadable `--config-file` silently becomes a from-scratch app.
   `deploy create` propagates read errors and falls back only when the user gave no path (tvc/src/commands/deploy/create.rs:251-261).

6. **[docs]** The documented `invalid_input` code is unreachable, and this command's validation failures emit `command_error`.
   The global LONG_ABOUT defines `invalid_input` as semantic validation failure in the command (tvc/src/cli.rs:56).
   Yet `ErrorCode::InvalidInput` carries `#[allow(dead_code)]` (tvc/src/errors.rs:54-56), and `invalid_app_config_error` builds a bare `anyhow!` (tvc/src/commands/app/create.rs:231-233).
   Command config validation failures therefore classify as the `command_error` fallback.
   The non-interactive multiple-candidates bail (tvc/src/commands/app/create.rs:85-91) has a missing-input shape and also classifies as `command_error`.
   JSON mode consumers cannot distinguish these from generic failures.
