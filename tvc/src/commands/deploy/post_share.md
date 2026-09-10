# tvc deploy post-share

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy post-share command.*

## Purpose (informative)

`tvc deploy post-share` posts a re-encrypted quorum key share to Turnkey through the
`post_tvc_quorum_key_share` activity. `tvc keys re-encrypt-local-share` produces the share
file offline. The command attributes the posted share to a share set operator UUID. It is
the last online step of local and YubiKey operator provisioning: `provisioning-details`,
then `re-encrypt-local-share`, then `post-share`. Hosted operators skip this flow. For
them `tvc deploy provision` re-encrypts and posts server side in one step.

## Acceptance scenario (normative)

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc login` and select org `acme-prod`. | The tvc config stores `acme-prod` as the active org; exit code 0. |
| 2 | Run `tvc deploy provisioning-details --deploy-id 2b7e1516-28ae-4d2a-abf7-158809cf4f3c --provision-bundle-out provision-bundle.json`. | The command writes `provision-bundle.json`; exit code 0. |
| 3 | On the operator machine, run `tvc keys re-encrypt-local-share --quorum-key-metadata quorum-key-metadata.json --provision-bundle provision-bundle.json --re-encrypted-out re-encrypted-share.json`. | The command writes `re-encrypted-share.json` with `deploymentId`, `ephemeralPublicKeyHex`, `reEncryptedShare`, and `shareApproval`; exit code 0. |
| 4 | Run `tvc deploy post-share --re-encrypted-share re-encrypted-share.json --share-operator-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`. | stdout shows one line `Provisioning Share ID: <id>`; exit code 0. |

All steps MUST pass in one run from a clean start, with a quorum key metadata file from
`tvc keys generate-local-quorum-key` and a deployment that awaits provisioning.

## Inputs (normative)

| Input | Flag | Env | Config key | Default | Prompted |
|---|---|---|---|---|---|
| Re-encrypted share file | `--re-encrypted-share <PATH>` | `TVC_RE_ENCRYPTED_SHARE` | none | none (required) | never |
| Share set operator UUID | `--share-operator-id <UUID>` | `TVC_SHARE_OPERATOR_ID` | none | none (required) | never |
| Auth (org ID, API keys) | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE`, `TVC_API_BASE_URL` | active org in the tvc config | none | never |

Neither command input has a tvc config key or a built-in default. The command MUST
resolve each input from the flag first and the environment variable second
(`tvc/src/commands/deploy/post_share.rs:23,27`). This matches the Part 00 resolution
order with the config file and built-in default tiers empty.

The share file supplies the deployment ID, the ephemeral public key, the share
ciphertext, and the approval signature (`post_share.rs:76-81`). The command MUST NOT
accept a `--deploy-id` flag (`tvc/tests/deploy_post_share.rs:29`).

When `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` all hold values, the
command MUST authenticate with them (`tvc/src/client.rs:48-64`). When only some of the
three hold values, the command MUST fail before any network call (`client.rs:226-234`).
When none hold values, the command MUST authenticate with the active org's stored
credentials (`client.rs:103-125`).

## Interactive behavior (normative)

The command MUST NOT prompt in any mode. The run function ignores the interactive
context (`post_share.rs:33`). Clap requires both command inputs. A missing input MUST
fail as a usage error with exit code 2, in interactive mode and in non-interactive mode
(`tvc/tests/deploy_post_share.rs:33-46`). JSON mode adds no further behavior change
(INV-G1 applies). For contrast, `tvc deploy approve` prompts to select an approving
operator (`tvc/src/commands/deploy/approve.rs:340-346`).

## Outputs (normative)

In human mode the command MUST print exactly one line:
`Provisioning Share ID: <provisioning share ID>` (`post_share.rs:65-69`).

In JSON mode the command MUST emit exactly one NDJSON object:
`{"reason":"quorum_key_share_posted","provisioningShareId":"<provisioning share ID>"}`
(`tvc/src/outcome.rs:44`; `post_share.rs:59-63`).

The provisioning share ID is an API assigned identifier. Vector comparison excludes it
(Part 00, global vectors).

## Side effects (normative)

- The command MUST read the share file and parse it as JSON (`post_share.rs:34-35`;
  `tvc/src/util.rs:18-28`).
- The command loads the tvc config before dispatch and creates it when absent (INV-G4;
  `tvc/src/cli.rs:215-223`). When the auth env vars hold no values, the command also
  reads the active org's API key file (`client.rs:115-117`).
- The command MUST submit exactly one activity: `post_tvc_quorum_key_share`
  (`post_share.rs:48-52`). The intent carries `deployment_id`,
  `ephemeral_public_key_hex`, and a `share_approval_bundle` with `operator_id`,
  `re_encrypted_share_hex`, and `signature` (`post_share.rs:71-84`).
- The command MUST hex encode the file's approval signature (`post_share.rs:81`). It
  MUST omit the file's `shareApproval.member`, alias and public key, from the intent
  (`post_share.rs:78-82`; test `post_share.rs:110-124`).
- The command MUST NOT write any file and MUST NOT access a YubiKey device.

## Failure modes (normative)

| Failure | Observation | Mechanism |
|---|---|---|
| Missing `--re-encrypted-share` or `--share-operator-id` | Usage error naming the missing flags; exit code 2. | clap required arguments; `tvc/tests/deploy_post_share.rs:33-46` |
| `--share-operator-id` value is not a UUID | Usage error; exit code 2. | `Uuid` value parser (`post_share.rs:28`) |
| Any usage error in JSON mode | One NDJSON object with `reason` = `command_error`, `code` = `usage_error`; exit code 2. | `handle_parse_error` (`tvc/src/cli.rs:154-182`) |
| Share file unreadable | Error chain `failed to read re-encrypted share output: <path>`; `code` = `command_error`; exit code 1. | `read_json_file` (`tvc/src/util.rs:22-24`) |
| Share file invalid JSON, or a required field absent | Error chain `failed to parse re-encrypted share output: <path>`; `code` = `command_error`; exit code 1. | `read_json_file` (`util.rs:26-27`); test `post_share.rs:127-142` |
| No active org and no env auth | Error chain ``No active organization. Run `tvc login` first.``; `code` = `command_error`; exit code 1. | `client.rs:104-106` |
| Active org has no API key | Error chain ``No API key found for org '<alias>'. Run `tvc login` first.``; `code` = `command_error`; exit code 1. | `client.rs:115-117` |
| Partial env auth | Error chain `partial env var auth: missing <names>. Set all three (TVC_ORG_ID, TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE) env vars or none.`; `code` = `command_error`; exit code 1. | `client.rs:226-234` |
| API failure | `code` per the Part 00 error taxonomy: `unauthorized`, `not_found`, `api_error`, `approval_required`, or `network_error`; exit code 1. | `classify` (`tvc/src/errors.rs:93-103,212-249`) |

The `.context("failed to post quorum key share")` wrapper keeps the typed
`TurnkeyClientError` in the chain (`post_share.rs:48-52`), so `classify` maps API
failures to their taxonomy codes (`errors.rs:93-103`).

The command runs no local check on the bundle before submission. It does not confirm
that the hex fields decode, that the signature verifies, or that the operator UUID
matches the file's member. Such mismatches surface only as server side rejections
(Gap 3).

## Test vectors (normative)

All vectors share one worked example. The file `re-encrypted-share.json` holds:

```json
{
  "deploymentId": "2b7e1516-28ae-4d2a-abf7-158809cf4f3c",
  "ephemeralPublicKeyHex": "04abcd",
  "reEncryptedShare": "010203",
  "shareApproval": {
    "signature": "deadbeef",
    "member": { "alias": "operator-1", "pubKey": "aabbcc" }
  }
}
```

In the table, `BASE` stands for `tvc deploy post-share --re-encrypted-share
re-encrypted-share.json --share-operator-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`.
Vector comparison excludes the Part 00 global nondeterministic fields. For this command
the one excluded field is the provisioning share ID, an API assigned identifier.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | The worked example file; a logged-in active org; the API accepts the activity. | `BASE` | stdout: `Provisioning Share ID: <id>`; exit code 0 (`post_share.rs:65-69`). |
| V-2 | Same as V-1, with `TVC_RE_ENCRYPTED_SHARE=re-encrypted-share.json` and `TVC_SHARE_OPERATOR_ID=6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` set; no input flags. | `tvc deploy post-share --message-format json` | stdout: `{"reason":"quorum_key_share_posted","provisioningShareId":"<id>"}`; exit code 0 (`post_share.rs:23,27,59-63`; `outcome.rs:44`). |
| V-3 | No input flags, no input env vars. | `tvc deploy post-share` | stderr names `--re-encrypted-share <PATH>` and `--share-operator-id <SHARE_OPERATOR_ID>` as missing; exit code 2 (`tvc/tests/deploy_post_share.rs:33-46`). |
| V-4 | No input flags, no input env vars. | `tvc deploy post-share --message-format json` | stdout: one NDJSON object, `reason` = `command_error`, `code` = `usage_error`; exit code 2 (`cli.rs:160-176`; `tvc/src/output.rs:345-352`). |
| V-5 | The worked example file. | `tvc deploy post-share --re-encrypted-share re-encrypted-share.json --share-operator-id not-a-uuid` | clap value error for `--share-operator-id`; exit code 2 (`post_share.rs:28`). |
| V-6 | The worked example file. | `BASE --deploy-id 2b7e1516-28ae-4d2a-abf7-158809cf4f3c` | Unexpected argument error; exit code 2 (`tvc/tests/deploy_post_share.rs:29` pins the flag's absence). |
| V-7 | No file exists at `missing.json`. | `tvc deploy post-share --re-encrypted-share missing.json --share-operator-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | Error chain `failed to read re-encrypted share output: missing.json`; `code` = `command_error`; exit code 1 (`util.rs:22-24`). |
| V-8 | The worked example file without its `deploymentId` field. | `BASE` | Error chain `failed to parse re-encrypted share output: re-encrypted-share.json`, naming `deploymentId`; `code` = `command_error`; exit code 1 (`util.rs:26-27`; `post_share.rs:127-142`). |
| V-9 | Only `TVC_ORG_ID` set among the three required auth env vars. | `BASE` | Error chain `partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE. Set all three (TVC_ORG_ID, TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE) env vars or none.`; `code` = `command_error`; exit code 1 (`client.rs:226-234`). |
| V-10 | A fresh tvc config with no orgs; no auth env vars. | `BASE` | Error chain ``No active organization. Run `tvc login` first.``; `code` = `command_error`; exit code 1 (`client.rs:104-106`). |
| V-11 | Valid inputs; the API answers HTTP 401. | `BASE` | Error chain starts `failed to post quorum key share`; `code` = `unauthorized`; exit code 1 (`post_share.rs:48-52`; `errors.rs:219`). |
| V-12 | The worked example file and the operator UUID, at the unit level. | `build_post_tvc_quorum_key_share_intent` | Intent: `deployment_id` = `2b7e1516-28ae-4d2a-abf7-158809cf4f3c`, `ephemeral_public_key_hex` = `04abcd`, bundle `operator_id` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, `re_encrypted_share_hex` = `010203`, `signature` = `deadbeef` (`post_share.rs:110-124`). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT prompt in any mode. | The run body contains no prompt call and ignores the interactive context (`post_share.rs:33`); clap requires both inputs, so missing values fail at parse time (`tvc/tests/deploy_post_share.rs:33-46`). |
| INV-2 | The JSON `reason` MUST be `quorum_key_share_posted` and MUST NOT collide with another reason. | The `Outcome` variant name is the wire reason through internal serde tagging; a duplicate variant name fails to compile (`tvc/src/outcome.rs:29-44`); collision and snake_case tests (`outcome.rs:123-161`). |
| INV-3 | The intent MUST forward the file's ciphertext unchanged and the signature hex encoded, with the member identity omitted. | `build_post_tvc_quorum_key_share_intent` (`post_share.rs:71-84`); unit test `builds_expected_intent_shape` (`post_share.rs:110-124`). |
| INV-4 | An API failure MUST keep its typed `TurnkeyClientError` available for classification. | `.context` preserves the source chain (`post_share.rs:48-52`); `classify` downcasts through the chain (`tvc/src/errors.rs:93-103`). |

## Gaps (informative)

1. **[capability] The user supplies a raw operator UUID that tvc can resolve locally.**
   The file identifies the operator by public key (`shareApproval.member.pubKey`;
   `tvc/src/commands/keys/re_encrypt_local_share.rs:75-80`).
   The deployment's share set maps public key to operator UUID
   (`TvcDeployment.share_set`, `client/src/generated/external.data.v1.rs:721`;
   `TvcOperator.id` and `TvcOperator.public_key`, `external.data.v1.rs:794-797`). The
   file's `deploymentId` is enough to fetch that share set. The tvc config can store a
   local operator record's Turnkey ID (`LocalOperatorRecord.operator_id`,
   `tvc/src/config/turnkey.rs:344`). `deploy approve` derives exactly this value,
   `post_operator_id`, from the fetched deployment by matching the selected key
   (`tvc/src/commands/deploy/approve.rs:349-379`). post-share performs no such
   resolution (`post_share.rs:26-28`).

2. **[capability] No tvc command reveals the required UUID.** `app create` persists only
   `manifest_set_operator_ids` (`tvc/src/commands/app/create.rs:270-273`). The API
   result field `share_set_operator_ids` has zero references under `tvc/src`
   (`client/src/generated/immutable.activity.v1.rs:3594`). `deploy status` discards
   `share_set` (`tvc/src/commands/deploy/status.rs:68`). `provisioning-details` prints
   share set approvals by alias and public key and omits Turnkey operator UUIDs
   (`provisioning_details.rs:163-164,323`). The CLI demands this input and offers no way
   to obtain it.

3. **[capability] No local cross-check ties `--share-operator-id` to the posted share.**
   The intent carries the user's UUID next to the file's signature
   (`post_share.rs:78-82`). The command drops the file's member identity from the
   intent. A transposed or stale UUID therefore survives until a server side rejection,
   after signing and a network round trip. Sibling commands catch this class of mismatch
   locally. `provision` confirms the operator belongs to the manifest share set
   (`tvc/src/commands/deploy/provision.rs:142-151`). `approve` confirms the requested
   operator ID links to the selected key (`approve.rs:349-379,396-401`).

4. **[consistency] Env var naming drifts from the family's hand-off pattern.** The
   provisioning-details to re-encrypt hand-off pairs `TVC_PROVISION_BUNDLE_OUT`
   (`provisioning_details.rs:35`) with `TVC_PROVISION_BUNDLE`
   (`re_encrypt_local_share.rs:37`). The re-encrypt to post-share hand-off pairs
   `TVC_RE_ENCRYPTED_OUT` (`re_encrypt_local_share.rs:69`) with `TVC_RE_ENCRYPTED_SHARE`
   (`post_share.rs:23`). `provision` and `approve` both read `TVC_OPERATOR_ID`
   (`provision.rs:38`; `approve.rs:86`) while this command introduces
   `TVC_SHARE_OPERATOR_ID` (`post_share.rs:27`). The split arguably reflects the
   distinction between share set identity and manifest set identity. A pipeline that
   wires one operator through provision and post-share still sets two differently named
   vars.

5. **[consistency] The result passes through unvalidated, and the API error loses the
   enrichment that `provision` applies.** `provision` rejects an empty
   `provisioning_share_id` (`provision.rs:165-172`). It wraps client errors in
   `hosted_activity_error`, which names the activity that needs approvals
   (`provision.rs:107-109`; `tvc/src/operator/hosted.rs:355-364`). The post-share
   command prints whatever comes back (`post_share.rs:54-56`) and adds a bare `.context`
   (`post_share.rs:48-52`). Classification survives either way. The richer human message
   and the empty ID tripwire are absent here.

6. **[consistency] The command computes the timestamp inline.** `post_share.rs:43-46`
   duplicates `operator::timestamp_ms` (`tvc/src/operator.rs:478-483`), which
   `provision` uses (`provision.rs:107`). Trivial.

7. **[capability] The upstream command constrains which operator can produce the input
   file (producer side).** `keys re-encrypt-local-share` picks its backend from the
   default operator kind (`tvc/src/operator.rs:395-441`). A registered YubiKey
   operator record is reachable only when the default operator kind is `yubikey`
   (`re_encrypt_local_share.rs:125-128`). Multiple local operator records are a dead end
   (`SelectLocalOperatorError::MultipleLocalOperators`;
   `tvc/src/config/turnkey.rs:441-456`), and the `--serial` selector applies to YubiKey
   operator records only (`re_encrypt_local_share.rs:62`). A `hosted` default operator
   kind bails and redirects to `tvc deploy provision` (`operator.rs:435-439`). This
   canonical producer gap sits upstream in the hand-off and gates which shares can ever
   reach post-share.
