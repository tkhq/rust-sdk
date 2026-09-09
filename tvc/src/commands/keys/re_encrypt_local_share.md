# tvc keys re-encrypt-local-share

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the keys re-encrypt-local-share command.*

*Citation shorthand: `rs:` cites `tvc/src/commands/keys/re_encrypt_local_share.rs`; `tests:` cites `tvc/tests/keys_re_encrypt_local_share.rs`. Another file appears with its full path first, then its basename within the same section.*

## Purpose (informative)

The command decrypts one quorum key share with a local or YubiKey operator key.
The share comes from `keys generate-local-quorum-key` metadata.
The command re-encrypts the share to a deployment's attested ephemeral key and signs a share approval.
It is the offline middle step of the manual provisioning flow: `deploy provisioning-details`, then `keys re-encrypt-local-share`, then `deploy post-share`.
It runs fully offline: attestation and manifest approval verification run locally (tvc/src/provisioning.rs:160-241), and no Turnkey API call happens.

## Acceptance scenario (normative)

The scenario uses the fixture universe defined under Test vectors.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Write operator O's 64-char hex master seed to `operator_seed.txt`. | The file holds the seed for the operator public key named in `quorum_key_metadata.json`. |
| 2 | Run `tvc keys re-encrypt-local-share --quorum-key-metadata quorum_key_metadata.json --provision-bundle provision_bundle.json --operator-seed-path operator_seed.txt --dangerous-skip-verification --re-encrypted-out re_encrypted_share.json`. | Exit code 0. stderr carries the WARNING line and `Re-encrypted share written to: re_encrypted_share.json`. stdout is empty. |
| 3 | Parse `re_encrypted_share.json`. | One JSON object with `deploymentId` = `deploy-123`, `ephemeralPublicKeyHex` equal to the bundle value, `reEncryptedShare` (hex), and `shareApproval` (member plus signature). |
| 4 | Decrypt `reEncryptedShare` with ephemeral key E's private half. | The plaintext equals the original share bytes. |
| 5 | Verify `shareApproval.signature` with the member public key over the manifest hash. | The signature verifies. |

All steps pass in one run from a clean start (tests:209-308).

## Inputs (normative)

| Input | Flag | Env | Config key | Default | Prompted |
|---|---|---|---|---|---|
| Quorum key metadata file | `--quorum-key-metadata <PATH>` | `TVC_QUORUM_KEY_METADATA` | none | none (required) | never |
| Provision bundle file | `--provision-bundle <PATH>` | `TVC_PROVISION_BUNDLE` | none | none (required) | never |
| Operator seed (hex) | `--operator-seed <HEX_SEED>` | `TVC_OPERATOR_SEED` | none | none | never |
| Operator seed file (raw hex) | `--operator-seed-path <PATH>` | `TVC_OPERATOR_SEED_PATH` | none | none | never |
| YubiKey serial | `--serial <SERIAL>` | none | none | sole YubiKey operator record | only when several YubiKey operator records exist |
| Skip verification | `--dangerous-skip-verification` | `TVC_DANGEROUS_SKIP_VERIFICATION` | none | false | never |
| Output path | `--re-encrypted-out <PATH>` | `TVC_RE_ENCRYPTED_OUT` | none | none (inline output) | never |
| Decrypting operator | none | none | `orgs.<alias>.default_operator_kind` plus the sole operator record of that kind | active org default | PIN always on the YubiKey path; record only when several YubiKey operator records exist |

The flag and environment variable definitions live at rs:28-71.

The decrypting operator deviates from the Part 00 value resolution order.
It has no flag and no environment variable, apart from the raw seed escape hatch.
When both seed flags are absent, the active org's persisted default operator kind alone selects the operator kind (tvc/src/operator.rs:411-440).

The command MUST reject an invocation that carries both `--operator-seed` and `--operator-seed-path` (tvc/src/local_operator_key.rs:23-27).
When one seed flag is present, the command MUST use that seed.
It MUST then ignore `--serial` and the tvc config (rs:127-129; tvc/src/operator.rs:400-402).
This precedence is silent: no conflict error names the ignored `--serial` (Gap 4).

## Interactive behavior (normative)

Prompts exist only on the YubiKey path: both seed flags are absent and the active org's default operator kind is `yubikey` (rs:125-129).
On that path, in interactive mode:

1. When several YubiKey operator records exist and `--serial` is absent, the command MUST show the `Select YubiKey operator` select prompt (rs:133-154).
2. The command MUST collect the PIN with the `YubiKey PIV PIN (touch the device each time it blinks)` password prompt (rs:171-173).

The command MUST NOT read the PIN from the tvc config or the environment (rs:164-169).
Both prompts MUST settle before the command reads the input files, and device I/O MUST wait until both files parse (rs:179-188).

In non-interactive mode (JSON mode included, per INV-G1):

- When several YubiKey operator records exist and `--serial` is absent, the command MUST fail with code `missing_required_input` for `--serial` (rs:155-158).
- Every run whose default operator kind is `yubikey` MUST fail before file I/O with the PIN error (rs:164-169; tests:157-181).
- Non-interactive runs therefore succeed only with a `local` default operator kind or a seed flag.

Runs on the local path MUST NOT prompt in any mode: the prompt block is YubiKey-only (rs:125-129).

## Outputs (normative)

- Human mode without `--re-encrypted-out`: stdout MUST carry the share payload as pretty printed JSON (rs:95-103).
- Human mode with `--re-encrypted-out`: stderr MUST carry `Re-encrypted share written to: <path>` and stdout MUST stay empty (rs:104-107, 290, 735-738).
- JSON mode: stdout MUST carry one NDJSON object with `reason` = `re_encrypted_share_generated`. Without `--re-encrypted-out` the object MUST carry the flattened fields `deploymentId`, `ephemeralPublicKeyHex`, `reEncryptedShare`, and `shareApproval` (unit test rs:697-718). With `--re-encrypted-out` it MUST carry `writtenTo` (unit test rs:720-733).
- With `--dangerous-skip-verification`, in human mode the command MUST print one WARNING line on stderr before other output (rs:117-122).
- JSON mode MUST suppress the WARNING line and the written-to narration: both print through the human-only writer (tvc/src/output.rs:154-159, 272-276).

## Side effects (normative)

- Reads: the quorum key metadata JSON and the provision bundle JSON (rs:183-186). It also reads the registered operator key file or the raw seed file, and the YubiKey registry entries in the tvc config.
- YubiKey device (YubiKey path only): the command opens the device by serial and verifies the PIN (rs:188). It requires one touch for share decryption plus one touch for the approval signature (rs:217-231).
- Writes: only the `--re-encrypted-out` file, and only when the flag is present (rs:286-290).
- The command MUST NOT call the Turnkey API (INV-1).
- Aside from the INV-G4 creation of an absent file, the command MUST NOT write the tvc config. It reads the tvc config to select the operator kind and the sole operator record of that kind.

## Failure modes (normative)

Every row without a `code` value classifies as `command_error` with exit code 1 (Part 00 error taxonomy).

| Condition | Observation | Mechanism |
|---|---|---|
| `--quorum-key-metadata` or `--provision-bundle` absent | clap usage error naming the missing flags; exit code 2; `usage_error` in JSON mode | clap required args; tests:108-120 |
| Both seed flags given | `--operator-seed and --operator-seed-path are mutually exclusive, please provide only one` | tvc/src/local_operator_key.rs:23-27 |
| No seed flag and no active org | `No active organization. Run 'tvc login' first or provide --operator-seed or --operator-seed-path.` | tvc/src/operator.rs:404-409 |
| Default operator kind `hosted` | error names the local key need and redirects to `tvc deploy provision`; fires before file reads | tvc/src/operator.rs:433-439; tests:126-151 |
| Default kind `local`, zero local operator records | `no local operator is configured`, with `org '<alias>'` context | tvc/src/config/turnkey.rs:388, 441-457 |
| Default kind `local`, several local operator records | `multiple local operators are configured`; no disambiguator exists on this command (Gap 1) | tvc/src/config/turnkey.rs:390-391, 452-456 |
| Default kind `yubikey`, zero YubiKey operator records | `no YubiKey operator is configured` | tvc/src/config/turnkey.rs:426-427, 513-515 |
| `--serial` names no YubiKey operator record | `no YubiKey operator has serial <serial>`; refused before file I/O | tvc/src/config/turnkey.rs:509-512; tests:183-207 |
| Several YubiKey operator records, non-interactive mode, no `--serial` | code `missing_required_input` for `--serial` | rs:155-158 |
| Default kind `yubikey` in non-interactive mode | `a YubiKey operator needs its PIN typed at an interactive prompt; the PIN is never read from config or the environment` | rs:164-169; tests:157-181 |
| Metadata quorum key differs from manifest quorum key | `quorum key metadata quorumKeyPublic (<hex>) does not match provision bundle manifest quorumKey (<hex>)`; `--dangerous-skip-verification` does not bypass this check | rs:242-259; unit tests rs:526-541, 648-670 |
| Operator public key absent from metadata shares | `operator (<hex>) not found in quorum key metadata shares` | tvc/src/quorum_key_metadata.rs:46-48 |
| Operator absent from manifest share set | `operator (<hex>) not part of share set` | rs:261-279 |
| Attestation or manifest approval verification failure | context-wrapped error from the verification chain | tvc/src/provisioning.rs:160-241 |

## Test vectors (normative)

The vectors and the acceptance scenario share one fixture universe.

| Fixture | Contents |
|---|---|
| Operator O | A P-256 pair whose 64-char hex master seed is in `operator_seed.txt`. |
| Ephemeral key E | A P-256 pair; its public half is the bundle's ephemeral key. |
| `quorum_key_metadata.json` | `quorumKeyPublic` (hex), `threshold` = 1, one share encrypted to O's public key. |
| `provision_bundle.json` | `deploymentId` = `deploy-123`, `ephemeralPublicKeyHex` = E's public hex, `fetchedAtUnixMs` = 1712345678901, a placeholder attestation document, and a manifest envelope whose quorum key matches the metadata and whose share set contains O. |
| hosted-org config | tvc config: active org `hosted-org` (id `88888888-8888-4888-8888-888888888888`), default operator kind `hosted`, one hosted operator record (tvc/tests/common.rs:71-106). |
| yubikey-org config | tvc config: active org `yubikey-org` (id `99999999-9999-4999-8999-999999999999`), default operator kind `yubikey`, one YubiKey operator record with serial `01c95c1f` (tvc/tests/common.rs:124-173). |

Share-transform vectors pass `--dangerous-skip-verification` because the fixture bundle carries a placeholder attestation document.
Vector comparison excludes `reEncryptedShare` and `shareApproval.signature` in addition to the Part 00 global exclusions: encryption randomness makes them nondeterministic.
V-1 checks both fields cryptographically: decrypt with E, verify with O's public key.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Fixture files; `operator_seed.txt` | `tvc keys re-encrypt-local-share --quorum-key-metadata quorum_key_metadata.json --provision-bundle provision_bundle.json --operator-seed-path operator_seed.txt --dangerous-skip-verification --re-encrypted-out re_encrypted_share.json` | Exit 0; stdout empty; stderr carries the WARNING and `Re-encrypted share written to: re_encrypted_share.json`; the file passes acceptance steps 3 to 5 (tests:209-308). |
| V-2 | Same fixtures | V-1 invocation without `--re-encrypted-out` | Exit 0; stdout carries the payload as pretty printed JSON (rs:95-103). |
| V-3 | Same fixtures | V-2 invocation plus `--message-format json` | Exit 0; one NDJSON object with `reason` = `re_encrypted_share_generated`, `deploymentId` = `deploy-123`, `ephemeralPublicKeyHex`, `reEncryptedShare`, `shareApproval` (unit test rs:697-718). |
| V-4 | Same fixtures | V-1 invocation plus `--message-format json` | Exit 0; one NDJSON object `{"reason":"re_encrypted_share_generated","writtenTo":"re_encrypted_share.json"}`; stderr carries no WARNING line (unit test rs:720-733; tvc/src/output.rs:154-159). |
| V-5 | Any state | `tvc keys re-encrypt-local-share` | clap error naming `--quorum-key-metadata <PATH>` and `--provision-bundle <PATH>`; exit 2 (tests:108-120). |
| V-6 | Same fixtures | V-1 invocation plus `--operator-seed abababababababababababababababababababababababababababababababab` | `--operator-seed and --operator-seed-path are mutually exclusive, please provide only one`; exit 1; code `command_error` (tvc/src/local_operator_key.rs:23-27; unit test local_operator_key.rs:138-142). |
| V-7 | hosted-org config; no seed flag; nonexistent input files | `tvc keys re-encrypt-local-share --quorum-key-metadata does_not_exist_metadata.json --provision-bundle does_not_exist_bundle.json --dangerous-skip-verification` | stderr contains `needs the local operator key the share was encrypted to` and `tvc deploy provision`; exit 1; the input files stay unread (tests:126-151). |
| V-8 | yubikey-org config; stdin closed; nonexistent input files | V-7 invocation | stderr contains `a YubiKey operator needs its PIN typed at an interactive prompt`; exit 1; no device access (tests:157-181). |
| V-9 | yubikey-org config (registered serial `01c95c1f`); nonexistent input files | V-7 invocation plus `--serial deadbeef` | stderr contains `no YubiKey operator has serial deadbeef`; the metadata file stays unread; exit 1 (tests:183-207). |
| V-10 | yubikey-org config with two YubiKey operator records; stdin closed | V-7 invocation | Error with code `missing_required_input` naming `--serial`; exit 1 (rs:155-158). |
| V-11 | tvc config with no active org; no seed flag | V-7 invocation | `No active organization. Run 'tvc login' first or provide --operator-seed or --operator-seed-path.`; exit 1 (tvc/src/operator.rs:404-409). |
| V-12 | Fixtures where `quorumKeyPublic` differs from the manifest quorum key | V-1 invocation | Error contains `does not match`; exit 1; the skip flag does not bypass the check (unit tests rs:526-541, 648-670). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST run fully offline: no Turnkey API call. | `run` constructs no API client, and the share transform consumes only parsed local inputs (rs:113-199, 201-239). Behavioral check: the integration tests run with no server (tvc/tests/keys_re_encrypt_local_share.rs). |
| INV-2 | Operator selection and every prompt MUST settle before the command reads the input files, and device I/O MUST wait until both files parse. | Statement order in `run`: prompts (rs:125-177), operator selection (rs:181), file reads (rs:183-186), device resolution (rs:188). Behavioral checks: tests:126-151, 157-181, 183-207 pass nonexistent input files. |
| INV-3 | The YubiKey PIN MUST reach the command only through the interactive password prompt. | The sole `Pin` construction wraps `prompts::password` (rs:171-173); the promptless path fails first (rs:164-169). Behavioral check: tests:157-181. |
| INV-4 | The emitted ciphertext MUST decrypt only with E's private half, and the plaintext share MUST NOT appear in any output. | The plaintext lives only inside the re-encryption block scope (rs:217-226); the output holds hex ciphertext (rs:234-239). Behavioral check: round-trip decryption with E (tests:295-297). |

## Gaps (informative)

1. **[capability] A registered local operator record is unusable unless the org's persisted default operator kind is `local`**. The user cannot pick the operator at all. This is the verified form of Richard's observation. With no seed flag, the operator kind comes solely from `org.default_operator_kind` (tvc/src/operator.rs:411-440). No flag, environment variable, or prompt selects a kind or an operator record on this command (Args, rs:28-71). That default is org-level persisted state. `operator create --make-default` flips it to `hosted` (tvc/src/commands/operator/create.rs:230-232) or `yubikey` (create.rs:399-401). An interactive confirm prompt offers the same flip (create.rs:320-325). Org creation sets the initial value (tvc/src/config/turnkey.rs:643-651). Nothing ever sets it back to `local` except hand editing the tvc config (no `operator set-default` command exists). The precise condition is therefore "the default operator kind is `local`": in practice, whichever operator most recently became the default. That condition differs from strict last use, yet the effect Richard described holds. A configured local operator record becomes unreachable once a YubiKey or hosted operator becomes the default. The escape hatch is worse than it looks. The registered key file holds `StoredQosOperatorKey` JSON, while `--operator-seed-path` parses raw hex (tvc/src/local_operator_key.rs:80-99). The user therefore has to extract `private_key` from the JSON by hand. Target UX per Richard: pick any configured operator record: local, YubiKey, and today hosted. Hosted could later leave this path. That possibility stays informative context; the normative sections above pin only current behavior.

2. **[consistency] Sibling commands let the user choose an operator; this command hard-codes the choice**. `deploy approve` enumerates every registered operator record of every kind as a candidate and filters by manifest set membership (tvc/src/commands/deploy/approve.rs:242-346). It accepts a mutually exclusive selector group: `--operator-id`, `--serial`, `--operator-seed`, `--operator-seed-path` (approve.rs:52-57). It prompts `Select approving operator` when the choice is ambiguous (approve.rs:345). Config defaults are "intentionally absent from this boundary" (tvc/src/operator.rs:306-307). `deploy provision` requires an explicit `--operator-id` (tvc/src/commands/deploy/provision.rs:37-39). Even `keys backup-operator-key` ignores the default operator kind and reaches the local operator record directly (tvc/src/commands/keys/backup_operator_key.rs:63-79). `keys re-encrypt-local-share` is the only command in the share flow where persisted default state constrains an explicit choice. The data for approve-style matching already exists. The metadata names each share's operator by public key (tvc/src/quorum_key_metadata.rs:29-49). It can hold shares for several operators (tvc/src/commands/keys/generate_local_quorum_key.rs:112-150). A selector could therefore filter candidates to the operator records that actually hold a share.

3. **[capability] A hosted default operator kind blocks the command outright even when a registered local or YubiKey operator holds a share**. The hosted arm fails with a redirect to `tvc deploy provision` (tvc/src/operator.rs:433-439). It consults neither the other registered operator records nor the metadata's share list. The user has to flip `default_operator_kind` in the tvc config to proceed. The legitimate core: a hosted operator cannot decrypt locally, and re-encryption needs decrypt capability (rs:204-206), so hosted itself belongs on the `deploy provision` path. The gap is that mere hosted defaultness blocks the registered local and YubiKey operator records.

4. **[consistency] `--serial` has effect only when the default operator kind is `yubikey` and no seed flag is present; otherwise the command silently ignores it**. The whole YubiKey selection block sits behind `operator_seed_source.is_none() && org.default_operator_kind == OperatorKind::Yubikey` (rs:127-129). `--serial` with a `local` or `hosted` default, or alongside `--operator-seed`, therefore does nothing, and no conflict error appears. `deploy approve` places `--serial` in an exclusive ArgGroup (tvc/src/commands/deploy/approve.rs:52-57). It validates the serial against the org's operator records up front (approve.rs:155-163).

5. **[consistency] `--serial` has no environment variable**. Every other input on this command has one (rs:32-70); `--serial` is flag only (rs:59-62). `deploy approve --serial` shares the same omission (tvc/src/commands/deploy/approve.rs:91-97). The inconsistency is with this command's own flag set; the sibling agrees with the omission.

6. **[docs] The seed flag help text misstates the fallback**. The help says `If no seed flag is provided, uses the operator key from the logged-in org config` (rs:40-41). It omits three facts: the org's default operator kind decides, the command refuses a hosted default entirely, and a YubiKey default needs an interactive PIN. `long_about = None` (rs:27) leaves nowhere else to learn this before the errors hit.
