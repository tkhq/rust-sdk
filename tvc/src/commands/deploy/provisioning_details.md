# tvc deploy provisioning-details

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy provisioning-details command.*

## Purpose (informative)

The command fetches provisioning details for one deployment from the Turnkey API: the attestation document and the manifest envelope.
It verifies the Nitro attestation chain, the PCR values against the manifest, and the manifest set approvals.
It then prints an attestation summary.
With `--provision-bundle-out` it also writes a `ProvisionBundle` JSON file that `tvc keys re-encrypt-local-share` consumes.
Run it to inspect what an enclave attested to before provisioning, or to capture the bundle input for local share re-encryption.

Entry point: `run` (tvc/src/commands/deploy/provisioning_details.rs:65), dispatched from tvc/src/cli.rs:248-250.
The shared fetch and verify flow lives in tvc/src/provisioning.rs.

## Acceptance scenario (normative)

The scenario starts from a tvc config with active org `acme` and a valid stored API key.
Deployment `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` holds a valid attestation document and manifest envelope.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --provision-bundle-out ./provision-bundle.json`. | stdout starts with `Provision bundle written to: ./provision-bundle.json`, then one attestation summary block with `Verification: verified (attestation + approvals)`. Exit code 0. |
| 2 | Read `./provision-bundle.json`. | One JSON object with `attestationDocumentCoseSign1Base64`, `manifestEnvelope`, `fetchedAtUnixMs`, `deploymentId` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, and `ephemeralPublicKeyHex`. |
| 3 | Run step 1 again with `--message-format json`. | stdout carries one NDJSON object with `reason` = `provisioning_details` and `bundlePath` = `./provision-bundle.json`. Exit code 0. |

All steps pass in one run from a clean start against one live deployment.

## Inputs (normative)

| Input | Flag | Env | Config key | Default | Prompted |
|---|---|---|---|---|---|
| deployment id (UUID) | `-d`, `--deploy-id` | `TVC_DEPLOY_ID` | none | required | never |
| skip verification | `--dangerous-skip-verification` | `TVC_DANGEROUS_SKIP_VERIFICATION` | none | `false` | never |
| bundle output path | `--provision-bundle-out <PATH>` | `TVC_PROVISION_BUNDLE_OUT` | none | none (no bundle write) | never |
| auth | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE`, optional `TVC_API_BASE_URL` | active org in the tvc config | base URL `https://api.turnkey.com` | never |

The command MUST resolve each input in the Part 00 value resolution order (tvc/src/cli.rs:19).
The command MUST take the three auth environment variables together, or fall back to the active org (tvc/src/client.rs:48-64).
A partial auth environment set MUST fail with the names of the missing variables (tvc/src/client.rs:226-234).
No input reads a command config.
The sibling deploy commands take the same required `--deploy-id` and `TVC_DEPLOY_ID` pair.

## Interactive behavior (normative)

The command MUST NOT prompt in any mode.
`run` binds the context parameter as `_ctx` and never reads it (provisioning_details.rs:65).
The command behaves the same in interactive mode, non-interactive mode, and JSON mode.
A missing deployment id MUST raise a clap usage error with exit code 2 in every mode.

## Outputs (normative)

In human mode the command MUST print one text block (golden tests: provisioning_details.rs:447-472 and 475-495).
When the command writes a bundle, the block MUST start with `Provision bundle written to: <path>` and one blank line (provisioning_details.rs:256-258).
In JSON mode the command MUST emit one NDJSON object with `reason` = `provisioning_details` (tvc/src/outcome.rs:30,42).
The JSON object uses camelCase field names and hex encodes every byte field (provisioning_details.rs:218-231).

| Human label | JSON field | Value |
|---|---|---|
| `Deployment` | `deploymentId` | The deployment id string. |
| `Verification` | `verification` | The verification status literal (see below). |
| `Ephemeral Key` | `ephemeralKey` | Hex of the attested ephemeral public key. |
| `Module ID` | `moduleId` | The attestation module id. |
| `Digest` | `digest` | The Debug rendering of the NSM digest (provisioning_details.rs:220). |
| `Timestamp (ms)` | `timestampMs` | The attestation timestamp in milliseconds. |
| `User Data` | `userData` | Hex; human mode prints `(none)` when absent, JSON carries `null`. |
| `Nonce` | `nonce` | Hex; human mode prints `(none)` when absent, JSON carries `null`. |
| `PCRs` | `pcrs` | Index and hex value pairs. Human mode annotates index 16 `(setup manifest/key commitment)` and index 17 `(live manifest/key commitment)` (provisioning_details.rs:284-291). |
| `Certificate Length` | `certificateLength` | The leaf certificate byte count, rendered `<n> bytes` in human mode. |
| `CA Bundle Certificates` | `caBundleCertificates` | The CA bundle certificate count. |
| `Manifest Set Approvals` | `manifestSetApprovals`, `manifestSetThreshold` | Human mode prints `<count>/<threshold>`, then one `<alias>: <pubkey hex>` line per approval. |
| `Share Set Approvals` | `shareSetApprovals` | Human mode prints the count, then `<alias>: <pubkey hex>` lines; `(none)` when empty (provisioning_details.rs:306-315). |
| `Provision bundle written to` | `bundlePath` | The bundle path. The JSON key MUST appear only when the command wrote a bundle (provisioning_details.rs:189-191). |

The `verification` value MUST be the literal `verified (attestation + approvals)` or the literal `skipped attestation, PCR, and approval verification (--dangerous-skip-verification)` (provisioning_details.rs:93-97).
Both modes MUST include only PCR indices at or below 17 (`SUMMARY_PCR_MAX_INDEX`, provisioning_details.rs:61,143-147).

## Side effects (normative)

Dispatch loads the tvc config and creates the file with defaults when absent (tvc/src/cli.rs:219-224, INV-G4).
The command MUST NOT modify the tvc config.
The command MUST make exactly one API call, the read endpoint `get_tvc_deployment_provisioning_details` (provisioning.rs:81-85).
The command MUST NOT submit an activity.
The command MUST NOT access a YubiKey.

With `--provision-bundle-out` the command MUST write one `ProvisionBundle` JSON file (provisioning.rs:121-146).
The bundle carries the base64 attestation document, the manifest envelope, `fetchedAtUnixMs`, the deployment id, and the hex ephemeral public key.
The write goes through `write_file`, plain `tokio::fs::write` (tvc/src/util.rs:31-35).
The write MUST overwrite an existing file without confirmation, and MUST fail when the parent directory does not exist.
The command MUST build the summary before the bundle write (provisioning_details.rs:69-91).
When verification runs and fails, the command MUST NOT write the bundle.
With `--dangerous-skip-verification` the command writes the bundle without verification.

## Failure modes (normative)

A failed run MUST classify through the Part 00 error taxonomy and exit through the Part 00 exit codes.
The bundle write failure occurs after the API call succeeded.

| Failure | `code` | Exit | Mechanism |
|---|---|---|---|
| Missing or malformed `--deploy-id`, or a bad flag value | `usage_error` | 2 | clap parse; the JSON path routes through `handle_parse_error` (tvc/src/cli.rs:154-182) |
| No active org, or no stored API key | `command_error` | 1 | errors that direct the user to `tvc login` (tvc/src/client.rs:106,117) |
| A partial auth environment set | `command_error` | 1 | `partial env var auth: missing ...` (tvc/src/client.rs:226-234) |
| HTTP 401 or 403 | `unauthorized` | 1 | tvc/src/errors.rs:219 |
| HTTP 404 | `not_found` | 1 | tvc/src/errors.rs:220 |
| Another non-success HTTP status | `api_error` | 1 | tvc/src/errors.rs:221 |
| Transport failure (connect, timeout, DNS) | `network_error` | 1 | tvc/src/errors.rs:230-235 |
| Missing or empty attestation document or manifest envelope, or an undecodable envelope | `command_error` | 1 | bail (provisioning.rs:101-111); tests provisioning.rs:374-417 |
| Verification failure: manifest set approvals invalid or below threshold, attestation chain invalid or expired, PCR0-3 or manifest hash mismatch | `command_error` | 1 | `verify_provisioning_details` (provisioning.rs:206-240) |
| Bundle write failure | `command_error` | 1 | `failed to write file: <path>` (tvc/src/util.rs:31-35) |

## Test vectors (normative)

Vector comparison excludes the Part 00 global nondeterministic fields.
It also excludes the attestation content fields: `ephemeralKey`, `moduleId`, `digest`, `timestampMs`, `userData`, `nonce`, PCR values, `certificateLength`, `caBundleCertificates`, approval public keys, and the bundle's `fetchedAtUnixMs`.
Each vector runs against the acceptance scenario universe: active org `acme` and deployment `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` with a valid attestation document.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Valid auth and deployment. | `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | Human block in the golden shape (provisioning_details.rs:447-472) with `Verification: verified (attestation + approvals)` (provisioning_details.rs:93-97); PCR16 and PCR17 lines carry the commitment annotations (provisioning_details.rs:284-291); exit code 0. |
| V-2 | Valid auth and deployment. | `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | One NDJSON object with `reason` = `provisioning_details` (tvc/src/outcome.rs:30,42) and no `bundlePath` key (provisioning_details.rs:189-191); exit code 0. |
| V-3 | Valid auth and deployment; `./provision-bundle.json` absent. | `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --provision-bundle-out ./provision-bundle.json` | stdout header `Provision bundle written to: ./provision-bundle.json` (provisioning_details.rs:256-258); the file holds the five camelCase bundle fields (test provisioning.rs:437-460); exit code 0. |
| V-4 | Valid auth and deployment. | `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --dangerous-skip-verification` | `Verification: skipped attestation, PCR, and approval verification (--dangerous-skip-verification)` (provisioning_details.rs:93-97); no stderr warning; the attestation document parses without verification (provisioning_details.rs:122-124); exit code 0. |
| V-5 | Valid auth; `TVC_DEPLOY_ID` unset. | `tvc deploy provisioning-details --message-format json` | One JSON object with `code` = `usage_error` (tvc/src/cli.rs:154-182); exit code 2. |
| V-6 | Valid auth. | `tvc deploy provisioning-details -d not-a-uuid` | clap usage error from the `Uuid` value parser (provisioning_details.rs:27-28); exit code 2. |
| V-7 | tvc config without an active org; no auth environment variables. | `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | One JSON object; `message` starts with `No active organization.` (tvc/src/client.rs:106); `code` = `command_error`; exit code 1. |
| V-8 | `TVC_ORG_ID` set; `TVC_API_KEY_PUBLIC` and `TVC_API_KEY_PRIVATE` unset. | `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | `message` starts with `partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE` (tvc/src/client.rs:226-234); `code` = `command_error`; exit code 1. |
| V-9 | Valid auth; deployment id unknown to the API. | `tvc deploy provisioning-details -d 00000000-0000-4000-8000-000000000000 --message-format json` | `code` = `not_found` on HTTP 404 (tvc/src/errors.rs:220); exit code 1. |
| V-10 | API response without an attestation document. | `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | `message` contains `attestation document missing in provisioning details response` (provisioning.rs:101-104; test provisioning.rs:374-400); `code` = `command_error`; exit code 1. |
| V-11 | Manifest envelope with zero manifest set approvals. | `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | `message` contains `failed to verify manifest approvals` (provisioning.rs:211-213; test provisioning_details.rs:376-397); `code` = `command_error`; exit code 1. |
| V-12 | Valid auth and deployment; directory `/nonexistent` absent. | `tvc deploy provisioning-details -d 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --provision-bundle-out /nonexistent/provision-bundle.json` | `message` contains `failed to write file: /nonexistent/provision-bundle.json` (tvc/src/util.rs:31-35); `code` = `command_error`; exit code 1. |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT prompt in any mode. | `run` binds the context as `_ctx` with no prompt path (provisioning_details.rs:65). INV-G1 covers JSON mode. |
| INV-2 | When verification runs and fails, the command MUST NOT write a bundle file. | `run` builds the summary, propagating verification errors, before the bundle branch (provisioning_details.rs:69-91). |
| INV-3 | Both output modes MUST carry only PCR indices at or below 17. | One shared summary filters on `SUMMARY_PCR_MAX_INDEX` before either rendering (provisioning_details.rs:61,143-147). |
| INV-4 | The command MUST NOT submit an activity. | The flow calls one read endpoint, `get_tvc_deployment_provisioning_details`, and nothing else (provisioning.rs:81-85). |
| INV-5 | The JSON `reason` MUST be `provisioning_details`. | serde internal tagging on the `Outcome` variant name (tvc/src/outcome.rs:30,42). Behavioral check: tvc/src/outcome.rs:140. |

## Gaps (informative)

1. **[consistency] No stderr warning when `--dangerous-skip-verification` is set, unlike both sibling commands in the same flow**.
   `deploy provision` and `keys re-encrypt-local-share` print `WARNING: Skipping attestation, PCR, and manifest approval verification! ...` on stderr (tvc/src/commands/deploy/provision.rs:72-77, tvc/src/commands/keys/re_encrypt_local_share.rs:117-122).
   This command only embeds the skipped status in the outcome's `verification` field (provisioning_details.rs:93-97).
   A consumer that pipes the summary can miss it.

2. **[capability] `--provision-bundle-out` silently overwrites an existing file with no `--overwrite` gate, confirmation, or bail**.
   `write_file` is plain `tokio::fs::write` (tvc/src/util.rs:31-35, call at provisioning_details.rs:109-113).
   Siblings guard writes: `deploy init` and `app init` bail on an existing file (tvc/src/commands/deploy/init.rs:77-78, tvc/src/commands/app/init.rs:47-49).
   `keys backup-operator-key` requires `--overwrite` or an interactive confirmation (tvc/src/commands/keys/backup_operator_key.rs:93-96).

3. **[capability] The bundle records `fetchedAtUnixMs` but nothing can validate a bundle as of fetch time, so bundles silently expire with the attestation cert chain**.
   Verification at consumption always uses `SystemTime::now()` (provisioning.rs:261-268).
   The `validation_time_override` parameter is `None` on every production path; only tests set it (provisioning.rs:160-165, provisioning_details.rs:73, provision.rs:103).
   After the Nitro cert chain expires (hours), a bundle can pass `keys re-encrypt-local-share` only with `--dangerous-skip-verification` there.
   That flag discards all verification; a check pinned to the recorded fetch time would keep the rest.

4. **[docs] The status line `verified (attestation + approvals)` overstates the check: the output displays share set approvals without verifying them**.
   `check_approvals` validates only manifest set approvals: signatures, membership, uniqueness, and threshold (qos_core 0.14.1 boot.rs:663-704, called from provisioning.rs:211-213).
   The output lists share set approvals directly under that banner (provisioning_details.rs:306-315).
   Their signatures could be garbage and the command still reports `verified`.
   Also, manifest set approvals render as `count/threshold` while share set approvals render count only, though the manifest carries the share set threshold.

5. **[consistency] `TVC_DANGEROUS_SKIP_VERIFICATION` uses clap's strict bool env parsing while the global `TVC_NON_INTERACTIVE` accepts boolish values**.
   The global flag opts into `BoolishValueParser` (tvc/src/cli.rs:72-79), which accepts `1`, `yes`, and `on`.
   The dangerous flag is a bare bool with `env` (provisioning_details.rs:30-32), so `TVC_DANGEROUS_SKIP_VERIFICATION=1` fails value validation where `TVC_NON_INTERACTIVE=1` works.
   The same defect exists in provision.rs:41-43 and re_encrypt_local_share.rs:64-66.

6. **[consistency] A bundle produced under `--dangerous-skip-verification` is indistinguishable from a verified one**.
   `ProvisionBundle` has no field that records skipped verification (provisioning.rs:121-129).
   The ephemeral key it pins came from an unverified attestation document.
   Mitigation: `keys re-encrypt-local-share` re-verifies by default (provisioning.rs:167-203), so this only bites when both ends skip.
   In that case no artifact records that verification never happened.

7. **[docs] The PCR cutoff at index 17 appears in no help text and applies in JSON mode too**.
   `SUMMARY_PCR_MAX_INDEX = 17` drops every higher indexed PCR from both outputs (provisioning_details.rs:61,143-147).
   No flag includes them and no help text mentions the cutoff.
   A machine consumer of the JSON has no way to know about the filter.
