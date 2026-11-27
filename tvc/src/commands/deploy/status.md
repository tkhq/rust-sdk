# tvc deploy status

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy status command.*

## Purpose (informative)

`tvc deploy status` fetches the persisted record of one deployment from the Turnkey API and reports it.
The report covers the deployment, app, and manifest ids, the QOS version, the debug and deletion flags, the pivot container, and timestamps.
The command also validates the posted manifest approvals against the manifest set.
It reports a per-approval verdict and the quorum progress.
The report describes the stored deployment record.
Live cluster state (replicas, targeted deployment, last update) comes from `tvc deploy get-status`.

## Acceptance scenario (normative)

The scenario universe, shared with the test vectors: org `7b2f9d4e-1a3c-4b5d-8e6f-2c4a6b8d0e1f` holds deployment `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` for app `9f8e7d6c-5b4a-4321-8edc-ba9876543210` with manifest `3e5d7c9b-2a4f-4d6e-9b8a-7c6d5e4f3a2b`.
The manifest set holds `operator-alice` (`0a11ce00-0000-4000-8000-000000000001`) and `operator-bob` (`0b0b0000-0000-4000-8000-000000000002`) with threshold 2, and each posted a valid approval.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc login` and select org `7b2f9d4e-1a3c-4b5d-8e6f-2c4a6b8d0e1f`. | Exit code 0. The tvc config stores the org as the active org. |
| 2 | Run `tvc deploy status --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | stdout starts with `Deployment: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` and ends with `Manifest Approvals: 2/2 valid`, one `operator-alice (0a11ce00-0000-4000-8000-000000000001): valid` line, one `operator-bob (0b0b0000-0000-4000-8000-000000000002): valid` line, and `Quorum reached: yes`. Exit code 0. |
| 3 | Run `tvc deploy status --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json`. | stdout carries one NDJSON object: `reason` = `deployment_status`, `validCount` = 2, `threshold` = 2, `quorumReached` = true. Exit code 0. |

All steps pass in one run from a clean start against an org that holds the deployment.

## Inputs (normative)

| Input | Flag | Env | Tvc config source | Default | Prompted |
|---|---|---|---|---|---|
| deployment id | `-d`, `--deploy-id` (required, UUID) | `TVC_DEPLOY_ID` | none | none | never |
| auth (org plus API key) | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE` (all or none) | active org | none | never |
| API base URL | none | `TVC_API_BASE_URL` (env auth only) | per-org `api_base_url` | `https://api.turnkey.com` | never |

Resolution MUST follow the Part 00 order where a source exists.
The flag outranks `TVC_DEPLOY_ID` (tvc/src/commands/deploy/status.rs:29).
The auth environment variables outrank the active org (tvc/src/client.rs:51-61).
The deployment id has no tvc config source and no built-in default.
The command consumes no command config.
The three auth environment variables MUST arrive together or stay absent together; a partial set is an error (tvc/src/client.rs:226-234).
`TVC_API_BASE_URL` applies only with env auth.
The active org path reads the per-org `api_base_url` from the tvc config (tvc/src/client.rs:103-125, 196-198).
The inherited globals (`--non-interactive`, `--message-format`, `--color`) follow Part 00 with no command-specific handling.

## Interactive behavior (normative)

The command MUST NOT prompt in any mode.
Interactive mode and non-interactive mode runs are identical (tvc/src/commands/deploy/status.rs:35-121).
A missing or non-UUID `--deploy-id` MUST fail as a clap parse error in both modes (tvc/src/commands/deploy/status.rs:29-30).
That failure exits with code 2, never with `missing_required_input`.
The manifest-parse warning is the only mode-sensitive behavior: human mode alone prints it (see Outputs and Gap 5).

## Outputs (normative)

In human mode the command MUST print one report block to stdout (tvc/src/commands/deploy/status.rs:149-217, tvc/src/output.rs:86-96):

```
Deployment: <deployment id>
App ID: <app id>
Egress Enabled: <yes|no>
Manifest ID: <manifest id>
QOS Version: <version>
Marked for deletion: <yes|no>
Debug Mode: <yes|no>

Pivot Container:
  URL: <url>
  Path: <path>
  Args: ["<arg>", "<arg>"]

Created: <seconds>.<nanos>s
Updated: <seconds>.<nanos>s

Manifest Approvals: <valid count>/<threshold> valid
  <operator name> (<operator id>): <verdict>
Quorum reached: <yes|no>
```

The Pivot Container block MUST appear only when the deployment has a pivot container (tvc/src/commands/deploy/status.rs:169-182).
Its Args line MUST appear only when the args list holds at least one entry (status.rs:179-181).
The Created and Updated lines MUST appear only when the API returns the matching timestamp (status.rs:184-190). Gap 3 covers the nanos padding defect.
Human verdict strings are `valid`, `invalid signature`, `not in manifest set`, and `duplicate` (tvc/src/approvals.rs:129-140).
The test at tvc/src/commands/deploy/status.rs:239 pins the mixed-verdict rendering.
When the manifest bytes do not parse, the command MUST print `warning: failed to parse manifest; cannot validate approvals: <error>` to stderr (status.rs:88-90, tvc/src/output.rs:146-152).
The approvals section then MUST render `Manifest Approvals: <unknown>/<unknown> valid` with no approval lines and `Quorum reached: <unknown>` (status.rs:192-207).

In JSON mode the command MUST emit exactly one NDJSON terminal outcome with `reason` = `deployment_status` (tvc/src/outcome.rs:45).
The payload is camelCase (tvc/src/commands/deploy/status.rs:123-139):

| Field | Value |
|---|---|
| `deploymentId`, `appId`, `manifestId`, `qosVersion` | Strings from the API record. |
| `egressEnabled` | The app-level `enable_egress` flag (status.rs:107). |
| `markedForDeletion`, `debugMode` | Booleans from the API record. |
| `pivotContainer` | `{url, path, args}`, or null when absent (status.rs:112-116, 141-147). |
| `createdAt`, `updatedAt` | `{seconds, nanos}` string pairs, or null when absent (tvc/src/commands/app_status.rs:19-24). |
| `manifestApprovals` | Flattened `{approvals, threshold, validCount, quorumReached}`, or null when the manifest bytes did not parse (status.rs:136-138, tvc/src/approvals.rs:173-179). |

Each `approvals` element MUST carry `{id, operatorId, operatorName, signature, createdAt, verdict}` (tvc/src/approvals.rs:34-48, 149-154).
`signature` is lowercase hex (tvc/src/approvals.rs:50-53).
The operator public key MUST NOT serialize (tvc/src/approvals.rs:43-44).
JSON verdict values are kebab-case: `valid`, `invalid-signature`, `not-in-manifest-set`, `duplicate` (tvc/src/approvals.rs:129-130).
The test at tvc/src/approvals.rs:620 pins the full JSON shape.
Both modes MUST list approvals in ascending creation time order, with the approval id as the tie break (tvc/src/approvals.rs:285-299).
Approvals with a missing or unparseable timestamp sort last.

## Side effects (normative)

The command is read-only.
It MUST send exactly two API queries.
`get_tvc_deployment` fetches the record (tvc/src/commands/deploy/status.rs:39-48); `get_tvc_app` supplies the app-level `enable_egress` field (status.rs:79, tvc/src/client.rs:67-80).
It MUST NOT submit an activity, write a file, touch a YubiKey, or change the tvc config.
One global exception applies: dispatch creates a default tvc config when the file is absent (INV-G4, tvc/src/cli.rs:219-223).

## Failure modes (normative)

Every failure MUST follow the Part 00 error taxonomy.
In JSON mode the observation is the single error object.
In human mode the same failure prints an `error:` line to stderr (tvc/src/output.rs:161-182).

| Condition | `reason` | `code` | Exit code | Citation |
|---|---|---|---|---|
| Absent or non-UUID `--deploy-id` | `command_error` | `usage_error` | 2 | tvc/src/cli.rs:154-182, tvc/src/output.rs:344-352 |
| No active org, or no stored API key for it | `command_error` | `command_error` | 1 | tvc/src/client.rs:104-117 |
| Partial auth environment variables | `command_error` | `command_error` | 1 | tvc/src/client.rs:226-234 |
| HTTP 401 or 403 | `command_error` | `unauthorized`, with `httpStatus` | 1 | tvc/src/errors.rs:212-226 |
| HTTP 404 | `command_error` | `not_found`, with `httpStatus` | 1 | tvc/src/errors.rs:218-220 |
| Other non-success HTTP status | `command_error` | `api_error`, with `httpStatus` | 1 | tvc/src/errors.rs:221 |
| Connect, timeout, or DNS failure | `command_error` | `network_error` | 1 | tvc/src/errors.rs:227-235 |
| Success response with an empty deployment or manifest payload | `command_error` | `not_found`, no `httpStatus` | 1 | tvc/src/commands/deploy/status.rs:50-52, 72-77, tvc/src/errors.rs:93-96 |
| A posted approval with a missing operator | `command_error` | `not_found` | 1 | tvc/src/commands/deploy/status.rs:81-84, tvc/src/approvals.rs:86-88 |
| A posted approval with a non-UUID id or a malformed public key | `command_error` | `command_error` | 1 | tvc/src/commands/deploy/status.rs:81-84, tvc/src/approvals.rs:90-116 |
| A manifest set with an unparseable member key or an oversized threshold | `command_error` | `command_error` | 1 | tvc/src/commands/deploy/status.rs:98-101, tvc/src/approvals.rs:242-263 |

One failed posted approval MUST abort the whole command (tvc/src/commands/deploy/status.rs:81-84). Gap 4 proposes a change.
Unparseable manifest bytes are a degraded success: the command warns in human mode, nulls the approvals report, and exits 0 (status.rs:86-96).

## Test vectors (normative)

All vectors use the acceptance scenario universe.
Vector comparison excludes the Part 00 global nondeterministic fields.
For failure vectors, comparison covers `reason`, `code`, `httpStatus`, and the exit code; the `message` text carries OS and server wording.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Active org logged in; both approvals valid. | `tvc deploy status --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | stdout report block per Outputs, ending `Manifest Approvals: 2/2 valid`, two `: valid` approval lines, `Quorum reached: yes`; exit 0 (status.rs:149-217; test status.rs:239). |
| V-2 | Same as V-1. | Same as V-1 plus `--message-format json` | One NDJSON object: `reason` = `deployment_status`, `validCount` = 2, `threshold` = 2, `quorumReached` = true, verdicts `valid`; exit 0 (tvc/src/outcome.rs:45; JSON shape test tvc/src/approvals.rs:620). |
| V-3 | The deployment's manifest bytes do not parse as a `VersionedManifest`. | Same as V-1 | stderr line `warning: failed to parse manifest; cannot validate approvals: <error>`; stdout renders `Manifest Approvals: <unknown>/<unknown> valid`, no approval lines, `Quorum reached: <unknown>`; exit 0 (status.rs:86-96, 192-207). |
| V-4 | Same as V-3. | Same as V-2 | One NDJSON object with `reason` = `deployment_status` and `manifestApprovals` = null; no warning object; exit 0 (status.rs:88-96, tvc/src/output.rs:146-152). |
| V-5 | Any environment. | `tvc deploy status --message-format json` | One NDJSON object: `reason` = `command_error`, `code` = `usage_error`, `message` carries clap's usage text; exit 2 (tvc/src/cli.rs:160-176, tvc/src/output.rs:344-352). |
| V-6 | The API answers HTTP 404 for the deployment fetch. | `tvc deploy status --deploy-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --message-format json` | `reason` = `command_error`, `code` = `not_found`, `httpStatus` = 404; exit 1 (tvc/src/errors.rs:218-220, 225). |
| V-7 | The API answers success with `tvc_deployment` absent. | Same as V-6 | `reason` = `command_error`, `code` = `not_found`, no `httpStatus`, `message` contains `deployment not found: 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`; exit 1 (status.rs:50-52, tvc/src/errors.rs:24-29, 93-96). |
| V-8 | No auth environment variables; the tvc config has no active org. | Same as V-6 | `reason` = `command_error`, `code` = `command_error`, `message` contains ``No active organization. Run `tvc login` first.``; exit 1 (tvc/src/client.rs:104-106). |
| V-9 | Only `TVC_ORG_ID` = `7b2f9d4e-1a3c-4b5d-8e6f-2c4a6b8d0e1f` is set. | Same as V-6 | `reason` = `command_error`, `code` = `command_error`, `message` names `TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE` as missing; exit 1 (tvc/src/client.rs:226-234). |
| V-10 | One posted approval has `operator` = null. | Same as V-6 | `reason` = `command_error`, `code` = `not_found`, `message` contains `approval operator is missing`; exit 1 (status.rs:81-84, tvc/src/approvals.rs:58-59, 86-88; test tvc/src/approvals.rs:667). |
| V-11 | operator-bob's stored public key is `not-hex`. | Same as V-6 | `reason` = `command_error`, `code` = `command_error`, `message` contains `operator 0b0b0000-0000-4000-8000-000000000002 public key is not valid hex`; exit 1 (status.rs:81-84, tvc/src/approvals.rs:64-68, 109-113; test tvc/src/approvals.rs:679). |
| V-12 | The manifest set holds member `borked` with public key `aabbcc`. | Same as V-6 | `reason` = `command_error`, `code` = `command_error`, `message` contains `manifest set contains an invalid public key for member borked`; exit 1 (status.rs:98-101, tvc/src/approvals.rs:222-227, 250-261; test tvc/src/approvals.rs:600). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT prompt in any mode. | clap requires `--deploy-id` before the command body runs (tvc/src/commands/deploy/status.rs:29-30); `run` contains no prompt call and never reads `Ctx::is_non_interactive` (status.rs:35-121). |
| INV-2 | Approval verdicts MUST NOT depend on the API's response order. | `ValidatedManifest::validate` sorts approvals by creation time, then approval id, before classification (tvc/src/approvals.rs:290-299). Behavioral check: tvc/src/approvals.rs:558. |
| INV-3 | One manifest set member MUST count toward the threshold at most once. | The `counted` set keys verdicts by member key bytes; a repeat classifies as `duplicate` (tvc/src/approvals.rs:302-328). Behavioral check: tvc/src/approvals.rs:538. |
| INV-4 | `validCount` and `quorumReached` MUST agree with the serialized verdict list. | `with_meta` computes both from the validation at packaging time; no caller sets them (tvc/src/approvals.rs:181-204). Behavioral check: tvc/src/approvals.rs:620. |
| INV-5 | A manifest parse failure MUST NOT change the exit code. | `run` discards the parse error with `.ok()` and degrades the report field to null (tvc/src/commands/deploy/status.rs:86-102). |

## Gaps (informative)

1. **[consistency][docs] `status` and `get-status` naming is incoherent across the CLI, and the help text does not disambiguate.**

   `app status` reports live cluster status (tvc/src/cli.rs:444-445).
   `deploy status` reports the persisted record, and the live variant is `deploy get-status` (tvc/src/cli.rs:396-397, 404-405).
   Plain `status` therefore means opposite things in the two command groups.
   The about text is "Get the status of a deployment" (tvc/src/cli.rs:404, status.rs:24).
   It gives a user no way to know which of the two they want.
   Neither command has a long_about (`long_about = None`, status.rs:26, get_status.rs:25), and neither cross-references the other.

2. **[capability] The command offers no way to discover or pick a deployment id.**

   `--deploy-id` is a required clap UUID with no prompt and no tvc config fallback (status.rs:29-30).
   No `deploy list` command exists (tvc/src/cli.rs:392-422).
   Discovery requires `app status`'s live listing (tvc/src/commands/app/status.rs), which only shows deployments present in cluster state.
   An interactive user who inspects approval progress pastes a UUID obtained elsewhere.

3. **[bug?] Created and Updated nanos render with space padding where zero padding is the intent.**

   `TimestampPayload.nanos` is a String (tvc/src/commands/app_status.rs:21-24), and the `0` flag in `{:09}` is a no-op for strings.
   `format!("{:09}", "5")` yields `"5        "` (verified with a standalone rustc test).
   The output therefore prints `Created: 1723473600.5        s` in place of the intended `.000000005s`.
   Lines status.rs:185 and 189 and get_status.rs:117-119 share the flaw; app/status.rs:136 already uses the correct `{:0>9}`.

4. **[consistency] One malformed approval row hard-fails the whole status report, which contradicts the command's own degrade-gracefully design.**

   Approval parsing collects into `Result` and propagates the first error (status.rs:81-84).
   A single bad public-key hex on any posted approval therefore hides the entire deployment record.
   The very next step treats unparseable manifest bytes as a warning plus degraded output (status.rs:86-102).
   The approvals module itself states that validation classifies every approval in place of failing fast (tvc/src/approvals.rs:6-8).
   The degraded manifest path also drops successfully parsed approvals from the output entirely (the report field is None, status.rs:98-104).
   The user then cannot see who has approved.

5. **[capability] JSON mode carries no signal for the degraded manifest-parse path.**

   The warning goes through `ctx.shell().human().warn(...)`, which is a no-op in JSON mode (status.rs:88-90, tvc/src/output.rs:146-152).
   A machine consumer sees only `"manifestApprovals": null` with no reason field or warning message that names the manifest parse failure.
   The parse error text is recoverable only through `RUST_LOG` debug logging.

6. **[consistency] The command re-implements the deployment fetch inline in place of the shared helper.**

   status.rs:39-52 duplicates `client::fetch_tvc_deployment` (tvc/src/client.rs:83-100) verbatim: same request, same context string, same `MissingResource`.
   `deploy get-status` uses the shared helper (get_status.rs:44).
   Behavior is identical today, and the two copies can drift.
