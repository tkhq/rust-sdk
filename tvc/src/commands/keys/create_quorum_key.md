# tvc keys create-quorum-key

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the keys create-quorum-key command.*

## Purpose (informative)

The command creates a hosted quorum key. It submits one `CreateTvcQuorumKey` activity.
Turnkey generates the quorum key server side, splits it with Shamir sharing, and encrypts each share to one supplied operator encryption public key.
Run it to set up an app whose quorum key and share custody live in Turnkey.
It is the hosted counterpart of `keys generate-local-quorum-key`.
Implementation: `tvc/src/commands/keys/create_quorum_key.rs` (dispatch `tvc/src/cli.rs:289-291`).

## Acceptance scenario (normative)

These worked-example values ground the scenario and the test vectors.

| Name | Value |
|---|---|
| K1 | `046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c2964fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5` (a valid 65-byte uncompressed P-256 point, bare hex) |
| K2 | `047cf27b188d034f7e8a52380304b51ac3c08969e277f21b35a60b48fc4766997807775510db8ed040293d9ac69f7430dbba7dade63ce982299e04b79d227873d1` (a second valid 65-byte uncompressed P-256 point) |
| K1U | K1 with its hex digits in uppercase (parses to the same point) |
| U1 | `11111111-1111-4111-8111-111111111111` (tvc/tests/keys_create_quorum_key.rs:6) |
| U2 | `22222222-2222-4222-8222-222222222222` (tvc/tests/keys_create_quorum_key.rs:7) |
| ORG | `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` |

| Step | Action | Expected observation |
|---|---|---|
| 1 | Export `TVC_ORG_ID=ORG`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` with valid credentials; point `HOME` at a clean directory. | `build_client` selects env auth (tvc/src/client.rs:48-64). |
| 2 | Run `tvc keys create-quorum-key --threshold 2 --operator-encrypt-keys K1,K2 --message-format json`. | Exit code 0. |
| 3 | Read stdout. | One NDJSON object: `reason` = `quorum_key_created`, non-empty `quorumKeyId` and `quorumPublicKey`, `shareIds` with exactly two entries. |
| 4 | Read `~/.config/turnkey/tvc.config.toml`. | The default tvc config from dispatch exists; the command added nothing to it. |

All steps pass in one run from a clean start with valid credentials for one org.

## Inputs (normative)

| Input | Flag | Env | Command config | Default | Prompted |
|---|---|---|---|---|---|
| threshold (shares needed to reconstruct) | `--threshold`, u8, minimum 2 through the range parser (create_quorum_key.rs:26-28,41-46) | `TVC_QUORUM_KEY_THRESHOLD` | none | none, required | never |
| operator encryption public keys (65-byte uncompressed P-256, bare hex, comma-separated) | `--operator-encrypt-keys` (create_quorum_key.rs:48-56) | `TVC_OPERATOR_ENCRYPT_KEYS` | none | none, one source required | never |
| hosted operator UUIDs (comma-separated) | `--operator-ids` (create_quorum_key.rs:58-67) | `TVC_OPERATOR_IDS` | none | none, one source required | never |
| auth | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE`, optional `TVC_API_BASE_URL` | none (active org in the tvc config) | none | never |

The two source flags form one required, mutually exclusive Clap group (`tvc/src/commands/keys/create_quorum_key.rs:33-38`).
The parser MUST accept exactly one of `--operator-encrypt-keys` and `--operator-ids`.

Each of the three inputs MUST follow the Part 00 value resolution order: flag above env.
This command consumes no command config; the local sibling takes one through `--config-file`.

Env auth is all or nothing (`tvc/src/client.rs:38-64`).
With all three auth variables present, `build_client` MUST ignore the tvc config credentials.
With only some of the three present, `build_client` MUST fail and name the missing variables.

The `--operator-ids` path MUST resolve each UUID against the active org's operator records, even when env supplies auth (`create_quorum_key.rs:164-182`).
Only operator records of operator kind `hosted` resolve (`tvc/src/operator/hosted.rs:280-286`, `tvc/src/config/turnkey.rs:461-468`).

## Interactive behavior (normative)

The command MUST NOT prompt in any mode.
Clap requires the threshold and one operator source at parse time (`create_quorum_key.rs:33-46`).
A missing required input MUST exit with code 2 as a usage error in every mode.
The runtime `missing_required_input` path cannot fire for this command.
Interactive mode, non-interactive mode, and JSON mode produce identical behavior for identical inputs.
For contrast, sibling commands prompt to select operators: `deploy approve` (`tvc/src/commands/deploy/approve.rs:345`) and `keys re-encrypt-local-share` (`tvc/src/commands/keys/re_encrypt_local_share.rs:151`).

## Outputs (normative)

In human mode the command MUST print exactly three lines to stdout (`create_quorum_key.rs:78-88`):

```
Quorum Key ID: <id>
Quorum Public Key: <hex>
Share IDs: <id>, <id>, ...
```

In JSON mode the command MUST emit one terminal NDJSON object with `reason` = `quorum_key_created` (`tvc/src/outcome.rs:63`; tag rule `tvc/src/outcome.rs:30`).
The payload MUST carry the camelCase fields `quorumKeyId`, `quorumPublicKey`, and `shareIds` (`create_quorum_key.rs:70-76`).
The unit test at `create_quorum_key.rs:444-452` pins the exact payload shape.

The output omits the threshold, and the command persists nothing locally.
To place the key in an app config, the user records `quorumPublicKey` by hand (`tvc/src/config/app.rs:16`).

## Side effects (normative)

- The command MUST submit exactly one activity, `create_tvc_quorum_key` (`create_quorum_key.rs:137-141`). The activity creates the quorum key and its encrypted shares server side.
- The command MUST NOT write files and MUST NOT modify the tvc config. Global dispatch creates a default tvc config file when the file is absent (`tvc/src/cli.rs:219-224`; INV-G4).
- The command MUST finish all local checks before network I/O: count, threshold, and duplicate detection after canonicalization (`create_quorum_key.rs:114-124`). Registry resolution for `--operator-ids` also precedes `build_client` (`create_quorum_key.rs:122-128`).
- The command performs no device interaction.

## Failure modes (normative)

A parse failure MUST exit with code 2.
In JSON mode a parse failure MUST surface as one usage error object with `code` = `usage_error` (`tvc/src/cli.rs:154-182`).
A runtime failure MUST exit with code 1.
Every local validation error classifies as `command_error`: the `ensure!` errors carry no typed cause (`tvc/src/errors.rs:93-103`; Gap 10).
An API failure MUST preserve the typed `TurnkeyClientError` (`tvc/src/operator/hosted.rs:355-364`), so its `code` follows the Part 00 error taxonomy.

| Condition | Stage | Exit code | JSON `code` |
|---|---|---|---|
| Missing threshold or operator source | parse | 2 | `usage_error` (tvc/tests/keys_create_quorum_key.rs:31-45) |
| Both operator sources on the command line | parse | 2 | `usage_error` (tvc/tests/keys_create_quorum_key.rs:94-112) |
| Threshold outside 2..=255 | parse | 2 | `usage_error` (create_quorum_key.rs:26-28; tvc/tests/keys_create_quorum_key.rs:115-133) |
| Malformed key: `must not be empty`, `must be bare hex encoded`, `must be a 65-byte uncompressed P-256 public key`, `is not a valid P-256 point` | parse | 2 | `usage_error` (tvc/src/operator.rs:63-81) |
| Malformed UUID: `must be a UUID` | parse | 2 | `usage_error` (create_quorum_key.rs:147-149) |
| Operator count of 255 or more | runtime | 1 | `command_error` (create_quorum_key.rs:207-213) |
| Threshold above the operator count | runtime | 1 | `command_error` (create_quorum_key.rs:215-221) |
| Duplicate operator ID | runtime | 1 | `command_error` (create_quorum_key.rs:184-193) |
| Duplicate key after canonicalization, including two IDs that resolve to one key | runtime | 1 | `command_error` (create_quorum_key.rs:195-205,395-423) |
| No active org on the `--operator-ids` path | runtime | 1 | `command_error` (create_quorum_key.rs:168-170) |
| Unknown operator ID in the active org | runtime | 1 | `command_error` (tvc/src/operator/hosted.rs:273-275) |
| Invalid stored hosted operator record | runtime | 1 | `command_error` (tvc/src/operator/hosted.rs:187-203) |
| Authenticated org differs from the configured org (`--operator-ids` path only) | runtime | 1 | `command_error` (create_quorum_key.rs:129-131; tvc/src/operator.rs:149-158) |
| API rejection or transport failure | API call | 1 | per taxonomy: `unauthorized`, `not_found`, `api_error`, `approval_required`, `network_error` (tvc/src/operator/hosted.rs:355-364) |
| Response with an empty quorum key ID, empty public key, empty share ID, or a share count mismatch | response | 1 | `command_error` (create_quorum_key.rs:233-259) |

## Test vectors (normative)

Vector comparison excludes the Part 00 nondeterministic fields.
For this command the API assigns `quorumKeyId`, `quorumPublicKey`, and every `shareIds` entry; compare their presence and count only.
Rows write K1, K2, K1U, U1, U2, and ORG for the worked-example values from the acceptance scenario.
Vectors V-1 and V-2 need valid credentials and a reachable API; all other vectors run offline.
No offline vector can drive the API failure or response validation paths; the Failure modes citations pin those.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Env auth for ORG; clean `HOME`. | `tvc keys create-quorum-key --threshold 2 --operator-encrypt-keys K1,K2 --message-format json` | stdout: one NDJSON object, `reason` = `quorum_key_created`, `shareIds` length 2; exit 0 (create_quorum_key.rs:70-76,144; payload shape create_quorum_key.rs:444-452). |
| V-2 | Env auth for ORG; clean `HOME`. | `tvc keys create-quorum-key --threshold 2 --operator-encrypt-keys K1,K2` | stdout: three lines `Quorum Key ID: <id>`, `Quorum Public Key: <hex>`, `Share IDs: <id>, <id>`; exit 0 (create_quorum_key.rs:78-88,437-440). |
| V-3 | Empty environment. | `tvc keys create-quorum-key` | stderr names `--threshold <THRESHOLD>` and `<--operator-encrypt-keys <HEX>\|--operator-ids <UUID>>` as missing; exit 2 (tvc/tests/keys_create_quorum_key.rs:31-45). |
| V-4 | Empty environment. | `tvc keys create-quorum-key --threshold 2 --operator-encrypt-keys K1,K2 --operator-ids U1,U2` | stderr contains `cannot be used with`; exit 2 (tvc/tests/keys_create_quorum_key.rs:94-112). |
| V-5 | Empty environment. | `tvc keys create-quorum-key --threshold 256 --operator-encrypt-keys K1,K2` | stderr contains `out of range integral type conversion attempted`; exit 2 (tvc/tests/keys_create_quorum_key.rs:115-133). |
| V-6 | Empty environment. | `tvc keys create-quorum-key --threshold 2 --operator-encrypt-keys 04abcd` | stderr contains `must be a 65-byte uncompressed P-256 public key`; exit 2 (tvc/src/operator.rs:73-75). |
| V-7 | Clean `HOME`; `TVC_QUORUM_KEY_THRESHOLD=3`; `TVC_OPERATOR_ENCRYPT_KEYS=K1,K2`. | `tvc keys create-quorum-key` | stderr contains `threshold (3) cannot exceed operator encryption public key count (2)`; exit 1 (create_quorum_key.rs:215-221; tvc/tests/keys_create_quorum_key.rs:48-70). |
| V-8 | Clean `HOME`. | `tvc keys create-quorum-key --threshold 2 --operator-encrypt-keys K1,K1U --message-format json` | stdout: one NDJSON object, `reason` = `command_error`, `code` = `command_error`, message contains `duplicate operator encryption public key at index 1`; exit 1 (create_quorum_key.rs:195-205,339-347; tvc/src/errors.rs:93-103). |
| V-9 | Clean `HOME`. | `tvc keys create-quorum-key --threshold 2 --operator-ids U1,U1` | stderr contains `duplicate operator ID at index 1: 11111111-1111-4111-8111-111111111111`; exit 1 (create_quorum_key.rs:184-193,350-360). |
| V-10 | Clean `HOME`; the dispatch-created tvc config has no active org. | `tvc keys create-quorum-key --threshold 2 --operator-ids U1,U2` | stderr contains `No active organization`; exit 1 (create_quorum_key.rs:168-170). |
| V-11 | tvc config: active org `acme` with one hosted operator record whose ID is U1. | `tvc keys create-quorum-key --threshold 2 --operator-ids U1,U2` | stderr contains `hosted operator ID '22222222-2222-4222-8222-222222222222' was not found in org 'acme'`; exit 1 (tvc/src/operator/hosted.rs:273-275). |
| V-12 | Clean `HOME`; `TVC_QUORUM_KEY_THRESHOLD=2`; `TVC_OPERATOR_ENCRYPT_KEYS` holds 255 distinct valid keys. | `tvc keys create-quorum-key` | stderr contains `operator encryption public key count (255) must be less than 255`; exit 1 (create_quorum_key.rs:207-213,372-378). |

## Invariants (normative)

The Part 00 global invariants INV-G1 through INV-G4 apply; this section adds per-command invariants only.

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | All local validation MUST finish before any network I/O. | Statement order in `run`: source selection, count, threshold, registry resolution, and normalization precede `build_client` (create_quorum_key.rs:114-128). Behavioral check: tvc/tests/keys_create_quorum_key.rs:48-91 reach these errors with no credentials. |
| INV-2 | On the `--operator-ids` path the authenticated org MUST equal the org whose operator records supplied the keys. | `run` calls `ensure_authenticated_org` before submitting the activity (create_quorum_key.rs:129-131; tvc/src/operator.rs:149-158). |
| INV-3 | One invocation MUST submit at most one activity and MUST NOT persist anything locally. | `run` contains a single `create_tvc_quorum_key` call, takes the tvc config by value, and never calls `Config::save` (create_quorum_key.rs:106-145). |
| INV-4 | Keys sent to the API MUST be canonical lowercase hex with no duplicates. | `normalize_operator_encrypt_keys` renders each key through the `OperatorPublicKey` `Display` impl and rejects duplicates (create_quorum_key.rs:195-205; tvc/src/operator.rs:83-87). Behavioral check: create_quorum_key.rs:325-347. |
| INV-5 | The command MUST refuse a response whose share count differs from the submitted key count. | `validate_result` compares against the intent's key count, captured before submission (create_quorum_key.rs:127,245-249). |

## Gaps (informative)

1. **[capability]** `--operator-ids` resolves hosted operator records only; registered local and YubiKey operator records cannot participate.
   Resolution filters the registry to operator kind `hosted` (`tvc/src/operator/hosted.rs:280-286`, `tvc/src/config/turnkey.rs:461-468`).
   Local key files and the YubiKey key cache both hold composite public keys whose first 65 bytes are the encrypt key (`tvc/src/operator.rs:282-304`).
   To include such an operator, the user extracts the hex by hand and switches entirely to `--operator-encrypt-keys`.
   This is the canonical shape: registry state constrains what the user can pick explicitly.
2. **[capability]** Operator sources cannot mix.
   The Clap group is `multiple(false)` (`create_quorum_key.rs:33-38`; test `tvc/tests/keys_create_quorum_key.rs:94-112`).
   A set of two registered hosted operators plus one external partner key forces manual hex extraction for every key.
   The registry becomes unusable the moment one non-registry key participates.
3. **[capability]** No interactive path exists.
   Sibling commands prompt to select operators (`tvc/src/commands/deploy/approve.rs:345`, `tvc/src/commands/keys/re_encrypt_local_share.rs:151`).
   The registry plus `known_operator_candidates` (`tvc/src/operator.rs:448-475`) holds everything a picker needs.
   Here the parse-time required group makes prompting structurally impossible.
   `generate-local-quorum-key` shares this shape and also never prompts.
4. **[capability]** The hosted flow has no file input and no `init` counterpart.
   The local flow pairs `keys init-local-quorum-key`, which writes a template (`tvc/src/commands/keys/init_local_quorum_key.rs:29-45`), with `generate-local-quorum-key --config-file`, which consumes it (`tvc/src/commands/keys/generate_local_quorum_key.rs:23`).
   Here a large operator set has to fit on the command line or in one env var; the command accepts up to 254 keys.
5. **[consistency]** The hosted maximum operator count is 254; the local flow allows 255.
   The hosted bound requires the count below 255 (`create_quorum_key.rs:24,207-213`), while its comment cites qos support for at most 255 shares.
   The local path allows exactly 255 (`tvc/src/config/quorum_key.rs:10,56-61`).
   No recorded rationale explains the stricter hosted bound; the bound or the comment looks wrong.
6. **[docs]** The command accepts arbitrary encrypt keys, yet only hosted-operator keys have downstream use.
   `deploy provision` re-encrypts shares exclusively through a hosted registry operator record (`tvc/src/commands/deploy/provision.rs:38-39,79`).
   No command fetches or decrypts a hosted share by `shareId`; share IDs appear nowhere else in the CLI.
   Shares encrypted to external keys through `--operator-encrypt-keys` currently have no reachable use in tvc.
   Neither the help text nor a long_about says so, and the command defines no long_about (`create_quorum_key.rs:32`).
7. **[bug?]** The source tie-break silently prefers `--operator-ids`, and env-against-env exclusivity has no test.
   The run function picks the IDs source whenever that vec is non-empty and relies on the Clap group (`create_quorum_key.rs:112-118`).
   The only mutual-exclusion test passes both sources as flags (`tvc/tests/keys_create_quorum_key.rs:94-112`).
   If Clap does not fire the group conflict for two env-supplied values, the command silently ignores the encrypt keys.
   That situation arises when CI sets both `TVC_OPERATOR_IDS` and `TVC_OPERATOR_ENCRYPT_KEYS`.
   The audit could not verify this statically; a test settles it either way.
8. **[consistency]** Repeated flag occurrences replace earlier values.
   Both list args use `ArgAction::Set` (`create_quorum_key.rs:53,64`).
   `--operator-encrypt-keys A --operator-encrypt-keys B` yields only `[B]`; earlier values vanish without a warning.
   The help text does not say that a list needs one comma-joined occurrence.
9. **[consistency][docs]** The key format differs from the local sibling, and neither side documents the difference.
   This command takes 65-byte encrypt-only keys (`tvc/src/operator.rs:72-75`).
   The local sibling's command config takes 130-byte qos composite keys (`tvc/src/commands/keys/generate_local_quorum_key.rs:97-110`), and its template carries a composite key (`tvc/src/commands/keys/init_local_quorum_key.rs:34`, `tvc/src/operator.rs:293-296`).
   A key copied from the local template into `--operator-encrypt-keys` fails with a length error.
   The error gives no pointer to the composite versus encrypt-half distinction.
10. **[docs]** Semantic validation errors emit `code` = `command_error`; the global help promises `invalid_input` for them.
    `LONG_ABOUT` documents `invalid_input` for semantic validation failures in the command (`tvc/src/cli.rs:56`).
    `classify` maps typed errors only (`tvc/src/errors.rs:93-103`), and every validation in this command is a plain `ensure!`.
    Each one lands on the `command_error` fallback.
    The gap is global; this command evidences it fully.
