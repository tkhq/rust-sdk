# tvc deploy provision

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy provision command.*

## Purpose (informative)

`tvc deploy provision` provisions one hosted quorum-key share for a deployment.
The command fetches the deployment's provisioning details (attestation document plus manifest envelope) and verifies them.
It checks that the chosen hosted operator record belongs to the manifest share set.
It then submits a `ReEncryptTvcQuorumKeyShare` activity, and Turnkey re-encrypts that operator's share to the enclave.
It is the hosted counterpart of the local three-step flow (`deploy provisioning-details`, then `keys re-encrypt-local-share`, then `deploy post-share`).
Run it once per hosted share-set operator after a deployment's manifest approvals reach quorum.
Dispatch: tvc/src/cli.rs:251-253. Implementation: tvc/src/commands/deploy/provision.rs.

## Acceptance scenario (normative)

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc login` and select org `acme` (org id `7d2e9f4b-1a2c-4d3e-9f8a-0b1c2d3e4f5a`). | The tvc config stores `acme` as the active org with a saved API key file. |
| 2 | Run `tvc operator create --name ops-alice`. | The active org gains a hosted operator record `ops-alice` with operator ID `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. |
| 3 | Create deployment `9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` with `ops-alice` in its manifest share set, then collect manifest approvals to quorum. | `tvc deploy provisioning-details -d 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d` verifies attestation and approvals and exits with code 0. |
| 4 | Run `tvc deploy provision -d 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d --operator-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | stdout carries exactly one line, `Provisioning Share ID: <id>`, and the exit code is 0. |

All steps pass in one run from a clean start against a Turnkey environment that hosts the approved deployment.

## Inputs (normative)

| Input | Flag | Env | Config source | Default | Prompted |
|---|---|---|---|---|---|
| deployment ID | `-d, --deploy-id <UUID>` | `TVC_DEPLOY_ID` | none | none (required) | never |
| hosted operator ID | `--operator-id <UUID>` | `TVC_OPERATOR_ID` | none | none (required) | never |
| skip verification | `--dangerous-skip-verification` | `TVC_DANGEROUS_SKIP_VERIFICATION` | none | `false` | never |
| API credentials | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE` (all or none); `TVC_API_BASE_URL` optional | active org in the tvc config | `https://api.turnkey.com` base URL (client.rs:24) | never |
| operator identity and keys | none | none | hosted operator record in the active org | none | never |

Flag and env resolution MUST follow the Part 00 value resolution order (provision.rs:34-43).
Neither UUID has a tvc config source.
The command MUST resolve `--operator-id` to a hosted operator record in the active org (hosted.rs:265-276).
With env credentials, `TVC_ORG_ID` MUST equal the resolved record's org id (provision.rs:81, operator.rs:149-158).
The operator key material comes only from the tvc config, so env-credential runs (the CI path, client.rs:48-64) still require the tvc config.

## Interactive behavior (normative)

The command MUST NOT prompt on any path.
Interactive mode and non-interactive mode behave identically.
Both UUIDs are hard clap requirements, so a missing or malformed value MUST fail at parse time as `usage_error` (cli.rs:154-182).
The process exits with code 2.
No code path emits `missing_required_input`.
With `--dangerous-skip-verification` the command MUST print a warning line to stderr in human mode (provision.rs:72-77).
JSON mode suppresses that warning line (output.rs:271-276, 154-159).

## Outputs (normative)

- Human mode: the command MUST print exactly one stdout line, `Provisioning Share ID: <id>` (provision.rs:53-61).
- JSON mode: the command MUST emit one NDJSON object, `{"reason": "provisioning_share_created", "provisioningShareId": "<id>"}` (outcome.rs:29-43; pinned at provision.rs:475-482).
- A runtime error MUST surface per INV-G3 with reason `command_error` and a `code` from the Part 00 error taxonomy.

## Side effects (normative)

- The command reads the tvc config (cli.rs:215-240) and, without env credentials, the active org's API key file (client.rs:103-125). It MUST NOT write any file and MUST NOT mutate the tvc config.
- The command makes three API calls in order (provision.rs:83-109): `get_tvc_deployment_provisioning_details` (provisioning.rs:81-85), `get_tvc_deployment` (client.rs:83-100), then the `re_encrypt_tvc_quorum_key_share` activity. Only the activity changes server state.
- Before submission the command MUST verify manifest-set approvals, the Nitro attestation chain, and the PCR and manifest-hash binding (provisioning.rs:206-240).
- The deployment manifest MUST byte-equal the manifest in the provisioning envelope (provision.rs:137-140).
- The operator's composite key MUST be a manifest share-set member (provision.rs:141-151). Both checks run in both verification modes (INV-1).
- `--dangerous-skip-verification` skips only the approvals and attestation checks. The command MUST still parse the attestation document (provision.rs:123-126).
- The command performs no YubiKey or device interaction.

## Failure modes (normative)

Each failure MUST exit per INV-G2 and, in JSON mode, MUST emit one NDJSON error object per INV-G3.

| # | Failure | Observable behavior | `code` | Exit |
|---|---|---|---|---|
| F-1 | Missing or malformed `--deploy-id` or `--operator-id`, or an invalid `TVC_DANGEROUS_SKIP_VERIFICATION` literal | clap usage text; one NDJSON `usage_error` object in JSON mode (cli.rs:154-182) | `usage_error` | 2 |
| F-2 | No active org | `No active organization. Run 'tvc login' first.` (hosted.rs:271) | `command_error` | 1 |
| F-3 | Operator record resolution fails: ID absent (hosted.rs:273-275), duplicate IDs (hosted.rs:260), or malformed record keys (hosted.rs:196-203) | error chain names the operator ID and org alias | `command_error` | 1 |
| F-4 | Credential acquisition fails: partial env credentials (client.rs:226-234) or missing API key file (client.rs:115-117) | `partial env var auth: missing ...` or `No API key found for org '<alias>'. Run 'tvc login' first.` | `command_error` | 1 |
| F-5 | Env-credential org differs from the record's org | `authenticated organization (<env org>) does not match configured organization (<record org>)` (operator.rs:149-158) | `command_error` | 1 |
| F-6 | Deployment lookup returns no deployment | `MissingResource` (client.rs:97-99) | `not_found` | 1 |
| F-7 | Deployment present with the manifest missing or empty | `deployment response missing manifest` or `deployment response contained an empty manifest` (provision.rs:86-97; see gap 6) | `command_error` | 1 |
| F-8 | Verification fails: approvals (provisioning.rs:211-213), attestation (provisioning.rs:215-221; test provision.rs:342-372), or PCR binding (provisioning.rs:227-237) | `failed to verify manifest approvals` or `failed to parse and verify attestation document` or `attestation document did not match manifest expectations` | `command_error` | 1 |
| F-9 | Deployment manifest differs from the envelope manifest | `deployment manifest does not match provisioning manifest envelope` (provision.rs:137-140) | `command_error` | 1 |
| F-10 | Operator outside the manifest share set | `hosted operator '<name>' (<id>) is not part of the manifest share set` (provision.rs:142-151) | `command_error` | 1 |
| F-11 | Activity fails | typed `TurnkeyClientError` preserved through `hosted_activity_error` (provision.rs:109, hosted.rs:355-364), with `httpStatus` when applicable | `api_error`, `unauthorized`, `not_found`, `approval_required`, or `network_error` | 1 |
| F-12 | Activity result carries a blank provisioning share ID | `re-encrypt TVC quorum-key share response contained an empty provisioning share ID` (provision.rs:169-172) | `command_error` | 1 |

## Test vectors (normative)

Vector comparison excludes the nondeterministic fields Part 00 names.
The `provisioningShareId` value is an API assigned identifier; vectors compare only its presence.
Given states build on the acceptance scenario universe: active org `acme`, operator record `ops-alice` (`6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`), deployment `9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d`.
`PROVISION` abbreviates `tvc deploy provision -d 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d --operator-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Acceptance scenario steps 1-3 complete | `PROVISION` | stdout: `Provisioning Share ID: <id>`; exit 0 (provision.rs:53-61; test provision.rs:460-483) |
| V-2 | Same as V-1 | `PROVISION --message-format json` | One NDJSON object `{"reason": "provisioning_share_created", "provisioningShareId": "<id>"}`; exit 0 (pinned at provision.rs:475-482) |
| V-3 | Same as V-1 | `tvc deploy provision -d 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d --message-format json` | One NDJSON object with `code` = `usage_error`; exit 2 (cli.rs:154-176) |
| V-4 | `TVC_DANGEROUS_SKIP_VERIFICATION=1` exported | `PROVISION` | clap `usage_error`; exit 2 (provision.rs:42-43; clap default bool parser accepts only `true`/`false`; see gap 5) |
| V-5 | tvc config with no active org, no env credentials | `PROVISION --message-format json` | Error chain contains `No active organization. Run 'tvc login' first.`; `code` = `command_error`; exit 1 (hosted.rs:271; test hosted.rs:459-469) |
| V-6 | Active org `acme` with no record for the given ID | `tvc deploy provision -d 9b8a7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d --operator-id aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa` | Error `hosted operator ID 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa' was not found in org 'acme'`; `code` = `command_error`; exit 1 (hosted.rs:273-275; test hosted.rs:473-486) |
| V-7 | Only `TVC_ORG_ID` exported, no other auth env vars | `PROVISION` | Error `partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE. Set all three (TVC_ORG_ID, TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE) env vars or none.`; `code` = `command_error`; exit 1 (client.rs:226-234) |
| V-8 | Env credentials with `TVC_ORG_ID=00000000-0000-4000-8000-000000000000`; record org `7d2e9f4b-1a2c-4d3e-9f8a-0b1c2d3e4f5a` | `PROVISION` | Error `authenticated organization (00000000-0000-4000-8000-000000000000) does not match configured organization (7d2e9f4b-1a2c-4d3e-9f8a-0b1c2d3e4f5a)`; `code` = `command_error`; exit 1 (provision.rs:81, operator.rs:149-158) |
| V-9 | Deployment lookup returns an OK response with no deployment | `PROVISION --message-format json` | One NDJSON object with `code` = `not_found`; exit 1 (client.rs:97-99) |
| V-10 | Provisioning envelope with zero manifest-set approvals | `PROVISION` | Error `failed to verify manifest approvals`; `code` = `command_error`; exit 1 (provisioning.rs:211-213; test provision.rs:316-339) |
| V-11 | Same unapproved envelope as V-10 | `PROVISION --dangerous-skip-verification` | stderr warning `WARNING: Skipping attestation, PCR, and manifest approval verification! ...`; success outcome; exit 0 (provision.rs:72-77; test provision.rs:375-396) |
| V-12 | Manifest share set without the `ops-alice` composite key | `PROVISION --dangerous-skip-verification` | Error `hosted operator 'ops-alice' (6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b) is not part of the manifest share set`; `code` = `command_error`; exit 1 (provision.rs:142-151; test provision.rs:399-431) |
| V-13 | Deployment manifest bytes that differ from the envelope manifest | `PROVISION --dangerous-skip-verification` | Error `deployment manifest does not match provisioning manifest envelope`; `code` = `command_error`; exit 1 (provision.rs:137-140; test provision.rs:434-457) |
| V-14 | Activity result with `provisioning_share_id` = `" "` | `PROVISION` | Error `re-encrypt TVC quorum-key share response contained an empty provisioning share ID`; `code` = `command_error`; exit 1 (provision.rs:169-172; test provision.rs:486-496) |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST enforce manifest equality and share-set membership in both verification modes. | Unconditional `ensure!` checks after the mode branch in `build_re_encrypt_intent` (provision.rs:137-151). Behavioral checks with skip enabled: provision.rs:399-431, 434-457. |
| INV-2 | With `--dangerous-skip-verification` the command MUST still parse the attestation document before it builds the intent. | The skip branch calls `unsafe_attestation_doc_from_der` (provision.rs:123-126). |
| INV-3 | The command MUST verify the authenticated org against the operator record's org before any API request. | `ensure_authenticated_org` runs before both fetches and the activity (provision.rs:81-85, operator.rs:149-158). |
| INV-4 | The submitted intent MUST carry the fetched deployment manifest bytes unchanged. | `build_re_encrypt_intent` base64 encodes the fetched bytes with no re-serialization (provision.rs:157). Behavioral check: provision.rs:276-313. |
| INV-5 | The command MUST NOT report success for a blank provisioning share ID. | `validate_result` rejects blank IDs before the outcome exists (provision.rs:165-177). Behavioral check: provision.rs:486-496. |

## Gaps (informative)

1. **[capability] The command accepts the hosted operator only as a raw UUID: no default, no name selection, no prompt**.
   Even when the org's default operator kind is `hosted` with exactly one hosted operator record, provision refuses to run without `--operator-id` (provision.rs:38-39).
   `select_hosted_operator` (tvc/src/config/turnkey.rs:473-483) resolves that state as the org default elsewhere (operator.rs:291-296).
   The canonical mirror: `keys re-encrypt-local-share` with a hosted default redirects users to this command (operator.rs:435-440) with no UUID in sight.
   `deploy approve` shows the expected shape: enumerate operator records, filter to manifest members, auto-pick a sole candidate, prompt on multiple (approve.rs:318-346).
   Operator records have registry names (hosted.rs:217-219) that this command ignores.

2. **[capability] Only hosted operator records in this machine's tvc config can provision; no escape hatch exists**.
   `resolve_hosted_operator` errors with "was not found in org" for anything absent from the tvc config (hosted.rs:273-275).
   No flags accept the encrypt and sign public keys directly, and no API lookup fallback exists.
   Yet `tvc operator create` creates the operator server side, and the intent needs only its two public keys (provision.rs:158-159).
   A teammate on a second machine (env-credential auth works, client.rs:48-64) first hand-copies the hosted operator record into the local tvc config.
   The `--operator-id` help text omits this requirement (provision.rs:37).

3. **[consistency] The same domain role has different flag and env names across the two share-posting paths, and `TVC_OPERATOR_ID` means different things across commands**.
   Provision's share-set operator is `--operator-id` / `TVC_OPERATOR_ID` (provision.rs:38).
   `deploy post-share` names the same role `--share-operator-id` / `TVC_SHARE_OPERATOR_ID` (post_share.rs:26-28).
   `deploy approve` also reads `TVC_OPERATOR_ID` for its manifest-set operator selector (approve.rs:83-90).
   An env var exported for a provision pipeline silently becomes approve's operator selection in the same shell.

4. **[consistency] A verification-skipped provision is indistinguishable in JSON mode**.
   The only trace of `--dangerous-skip-verification` is a human mode stderr line (provision.rs:72-77) that JSON mode suppresses (output.rs:271-276, 154-159).
   `ProvisioningShareCreated` carries no verification field (provision.rs:49-51).
   Sibling `deploy provisioning-details` records `"verification": "skipped attestation, ..."` in its machine outcome (provisioning_details.rs:93-97, 176).
   CI logs cannot prove which mode produced a share.

5. **[consistency] `TVC_DANGEROUS_SKIP_VERIFICATION` accepts only the literals `true` and `false`, unlike the global bool envs**.
   The global `--non-interactive` uses `BoolishValueParser` (`1`, `yes`, `on`, and more; cli.rs:71-78).
   Provision's bool flag declares `env` with clap's default bool parser (provision.rs:42-43), so `TVC_DANGEROUS_SKIP_VERIFICATION=1` is a parse error where `TVC_NON_INTERACTIVE=1` works.
   The same trait appears family wide in provisioning_details.rs:31-32 and re_encrypt_local_share.rs:65-66.

6. **[consistency] A deployment response missing its manifest classifies as `command_error`**.
   The repo contract routes this "OK response with an empty resource" case to `not_found` through `MissingResource`.
   `fetch_tvc_deployment` does exactly that one step earlier for the missing deployment (client.rs:97-99).
   The bare `.context("deployment response missing manifest")` at provision.rs:93 sends machine consumers the generic fallback code (errors.rs:93-103).
   Related: the `invalid_input` code advertised in `LONG_ABOUT` (cli.rs:55-56) is `#[allow(dead_code)]`; none of this command's semantic validation failures ever produce it (errors.rs:54-56).

7. **[capability] No dry-run or preview exists: the command submits the activity immediately**.
   `deploy approve` offers `--dry-run` (approve.rs:119-120), and the local flow externalizes every artifact for inspection (`--provision-bundle-out`, `--re-encrypted-out`).
   Provision has no way to run its full verification and share-set membership check for one operator without also submitting the re-encrypt activity (provision.rs:98-109).
