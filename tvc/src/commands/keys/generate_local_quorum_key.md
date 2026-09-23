# tvc keys generate-local-quorum-key

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the keys generate-local-quorum-key command.*

## Purpose (informative)

`tvc keys generate-local-quorum-key` generates a fresh quorum key pair in local memory.
It Shamir-splits the master seed into `shares` pieces with reconstruction threshold `threshold`.
It encrypts each share to one operator public key from a command config.
It writes one quorum key metadata JSON file: the quorum public key plus the encrypted shares.
The command runs fully offline, with no Turnkey API call and no device access.
`tvc keys init-local-quorum-key` produces the command config template.
`tvc keys re-encrypt-local-share` later consumes the metadata file.
Plaintext key material lives only in memory; the `Zeroizing` wrapper clears it on drop (generate_local_quorum_key.rs:41-42).

## Acceptance scenario (normative)

The scenario starts in an empty directory.
OPK1, OPK2, and OPK3 name the composite public keys of three freshly generated operator key pairs.
Each key is 260 characters of lowercase hex.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Write `quorum_key.json` with `shares` = 3, `threshold` = 2, and `operatorPublicKeys` = [uppercase OPK1, OPK2, OPK3]. | The command config exists with three 260-character hex keys. |
| 2 | Run `tvc keys generate-local-quorum-key --config-file quorum_key.json --quorum-key-metadata-out quorum_key_metadata.json`. | Exit code 0. stdout holds three lines; the last reads `Threshold: 2` (tests/keys_generate_local_quorum_key.rs:62-64). |
| 3 | Parse `quorum_key_metadata.json`. | `threshold` = 2; `shares` holds 3 entries; the `operatorPublicKey` values equal [OPK1, OPK2, OPK3] in lowercase, in config order (tests/keys_generate_local_quorum_key.rs:66-79). |
| 4 | Decrypt the first two shares with the matching operator private keys, then reconstruct the master seed with `shares_reconstruct`. | The reconstructed pair's public key equals `quorumKeyPublic` (tests/keys_generate_local_quorum_key.rs:81-98). |

All steps pass in one run from a clean start.

## Inputs (normative)

| Input | Flag | Environment variable | Command config key | Default | Prompted |
|---|---|---|---|---|---|
| command config path | `-c`, `--config-file` | `TVC_QUORUM_KEY_CONFIG` | (none) | none, required (generate_local_quorum_key.rs:23-24) | no |
| metadata output path | `--quorum-key-metadata-out` | `TVC_QUORUM_KEY_METADATA_OUT` | (none) | `quorum_key_metadata.json` (generate_local_quorum_key.rs:27-33) | no |
| shares | (none) | (none) | `shares` | none | no |
| threshold | (none) | (none) | `threshold` | none | no |
| operator public keys | (none) | (none) | `operatorPublicKeys` | none | no |

The two path inputs MUST follow ranks 1, 2, and 4 of the Part 00 value resolution order (generate_local_quorum_key.rs:21-34).
The `shares`, `threshold`, and `operatorPublicKeys` values exist only at rank 3.
The command config supplies them, and no flag or environment variable can (config/quorum_key.rs:21-25).

Each operator public key MUST be a 130-byte qos composite key in bare hex (quorum_key_metadata.rs:52-57).
The composite is a 65-byte encrypt key then a 65-byte sign key.
The decoder trims whitespace and accepts either hex case (quorum_key_metadata.rs:52-57).
The command normalizes every key to lowercase hex in the output (generate_local_quorum_key.rs:105).

The command MUST reject a command config that breaks any rule in this table (config/quorum_key.rs:52-94).

| Rule | Mechanism |
|---|---|
| No value starts with `<FILL_IN`. | config/quorum_key.rs:53-55 |
| `1 <= shares <= 255` | config/quorum_key.rs:56-61 |
| `2 <= threshold <= shares` | config/quorum_key.rs:62-74 |
| `operatorPublicKeys` length equals `shares`. | config/quorum_key.rs:76-82 |
| Every key parses as a composite key. | config/quorum_key.rs:84-87 |
| No two keys normalize to the same value. | config/quorum_key.rs:88-90 |

Dispatch withholds the loaded tvc config from this command (cli.rs:292-294), so orgs and operator records never influence behavior.

## Interactive behavior (normative)

The command MUST NOT prompt in any mode.
The `ctx` parameter stays unused (generate_local_quorum_key.rs:45).
`--non-interactive` and JSON mode change nothing: every mode runs the same path (tests/keys_generate_local_quorum_key.rs:130-172).

## Outputs (normative)

In human mode the command MUST print exactly three stdout lines (generate_local_quorum_key.rs:85-95; tests/keys_generate_local_quorum_key.rs:62-64):

```
Quorum key metadata written to: <metadata path>
Quorum Public Key: <260 hex characters>
Threshold: <threshold>
```

In JSON mode the command MUST emit exactly one NDJSON object with these fields (tests/keys_generate_local_quorum_key.rs:163-171).

| Field | Value |
|---|---|
| `reason` | `quorum_key_generated` (outcome.rs:64) |
| `quorumKeyPublic` | 260-character lowercase hex quorum public key (generate_local_quorum_key.rs:126) |
| `threshold` | the command config `threshold` (generate_local_quorum_key.rs:70-74) |
| `metadataPath` | the resolved metadata output path (generate_local_quorum_key.rs:70-74) |

## Side effects (normative)

- Before dispatch the CLI loads the tvc config and creates the file when it is absent (INV-G4; cli.rs:215-223). This command never reads the loaded value (cli.rs:292-294).
- The command reads the command config once (generate_local_quorum_key.rs:46-47; util.rs:18-28).
- The command MUST refuse to overwrite an existing file at the metadata output path (generate_local_quorum_key.rs:50-55).
- On success the command MUST write one pretty-printed JSON file to the metadata output path (generate_local_quorum_key.rs:61-68). The table below gives its shape (quorum_key_metadata.rs:7-20).
- The write uses `fs::write` with default file permissions (generate_local_quorum_key.rs:63). The file holds only public keys and encrypted shares, so the permissions expose no plaintext secret.
- The command makes no Turnkey API call and touches no YubiKey (INV-4).

| Metadata field | Value |
|---|---|
| `quorumKeyPublic` | 260-character lowercase hex composite quorum public key (generate_local_quorum_key.rs:126) |
| `threshold` | the command config `threshold` (generate_local_quorum_key.rs:152-156) |
| `shares[i].operatorPublicKey` | the normalized form of command config key `i` (generate_local_quorum_key.rs:145-146) |
| `shares[i].share` | hex of the share encrypted to key `i` (generate_local_quorum_key.rs:139-147) |

## Failure modes (normative)

Every runtime failure MUST exit with code 1 and, in JSON mode, MUST emit `code` = `command_error` (errors.rs:93-103; INV-G2, INV-G3).
The command produces only untyped `anyhow` errors, so every failure lands on the `classify` fallback (errors.rs:102).
The table lists the failure checks in execution order (generate_local_quorum_key.rs:45-68; config/quorum_key.rs:52-94).

| Order | Trigger | Error message | Mechanism |
|---|---|---|---|
| 1 | `HOME` unset (pre-dispatch) | `HOME environment variable not set` | cli.rs:215 |
| 2 | Malformed tvc config (pre-dispatch) | `failed to parse config file: <path>` | cli.rs:225-230 |
| 3 | Command config unreadable | `failed to read quorum key config file: <path>` | util.rs:22-24 |
| 4 | Command config JSON does not match the schema | `failed to parse quorum key config file: <path>` | util.rs:26-27 |
| 5 | Placeholder operator keys | `config contains placeholder operator public keys` | config/quorum_key.rs:53-55 |
| 6 | `shares` out of bounds | `shares must be between 1 and 255, got <n>` | config/quorum_key.rs:56-61 |
| 7 | `threshold` below 2 | `threshold must be >= 2, got <n>` | config/quorum_key.rs:62-67 |
| 8 | `threshold` above `shares` | `threshold (<t>) cannot exceed shares (<s>)` | config/quorum_key.rs:68-74 |
| 9 | Key count differs from `shares` | `operatorPublicKeys length (<n>) must equal shares (<s>)` | config/quorum_key.rs:76-82 |
| 10 | A key fails to parse | `invalid operator public key at index <i>` | config/quorum_key.rs:84-87 |
| 11 | Two keys normalize equal | `duplicate operator public key <hex> at index <i>` | config/quorum_key.rs:88-90 |
| 12 | Metadata output file exists | `quorum key metadata file already exists: <path>` | generate_local_quorum_key.rs:50-55 |
| 13 | Share generation or encryption fails | `failed to generate quorum key shares: ...`, `failed to encrypt quorum key share: ...` | generate_local_quorum_key.rs:124-143 |
| 14 | Metadata serialization or write fails | `failed to serialize quorum key metadata`, `failed to write file: <path>` | generate_local_quorum_key.rs:61-68 |

The command config path has no default (generate_local_quorum_key.rs:23-24).
With no flag and no environment variable, argument parsing MUST fail: `code` = `usage_error`, exit code 2 (Part 00 exit codes).

`run` re-parses the keys and re-checks the key count after validation (generate_local_quorum_key.rs:57, 117-122).
The command config validation already rejects both defect classes, so the CLI cannot reach those paths.

## Test vectors (normative)

The vectors reuse OPK1, OPK2, and OPK3 from the acceptance scenario.
`cfg.json` names the command config in the working directory; `out.json` names the metadata output path.
Vector comparison excludes the Part 00 global fields plus `quorumKeyPublic` and every `share` value.
Key generation and share encryption draw fresh randomness on each run (generate_local_quorum_key.rs:124-143).

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | `cfg.json`: `shares` 3, `threshold` 2, keys [uppercase OPK1, OPK2, OPK3] | `tvc keys generate-local-quorum-key --config-file cfg.json --quorum-key-metadata-out out.json` | Exit 0. stdout ends `Threshold: 2`. `out.json` pairs [OPK1, OPK2, OPK3] (lowercase, config order) with encrypted shares (tests/keys_generate_local_quorum_key.rs:26-99). |
| V-2 | `cfg.json`: `shares` 2, `threshold` 2, keys [OPK1, OPK2] | `tvc --message-format json keys generate-local-quorum-key --config-file cfg.json --quorum-key-metadata-out out.json` | Exit 0. Exactly one NDJSON object: `reason` = `quorum_key_generated`, `threshold` = 2, `metadataPath` = the `out.json` path, `quorumKeyPublic` a string (tests/keys_generate_local_quorum_key.rs:130-172). |
| V-3 | `TVC_QUORUM_KEY_CONFIG` unset | `tvc --message-format json keys generate-local-quorum-key` | One JSON object with `code` = `usage_error`; exit 2 (generate_local_quorum_key.rs:23-24; tvc/src/cli.rs:145). |
| V-4 | `cfg.json`: `shares` 2, `threshold` 2, keys [`not-hex`, `also-not-hex`] | `tvc keys generate-local-quorum-key --config-file cfg.json` | Failure; stderr contains `invalid operator public key at index 0`; exit 1 (tests/keys_generate_local_quorum_key.rs:101-127). |
| V-5 | Same `cfg.json` as V-4 | `tvc --message-format json keys generate-local-quorum-key --config-file cfg.json` | One NDJSON object: `reason` = `command_error`, `code` = `command_error`, `message` contains `invalid operator public key at index 0`; exit 1 (errors.rs:93-103; tvc/src/cli.rs:130). |
| V-6 | `cfg.json`: `shares` 2, `threshold` 2, keys [`<FILL_IN_OPERATOR_PUBLIC_KEY_1>`, `<FILL_IN_OPERATOR_PUBLIC_KEY_2>`] | `tvc keys generate-local-quorum-key --config-file cfg.json` | Error `config contains placeholder operator public keys`; exit 1 (config/quorum_key.rs:53-55; test config/quorum_key.rs:109-120). |
| V-7 | `cfg.json`: `shares` 0, `threshold` 2, keys `[]` | `tvc keys generate-local-quorum-key --config-file cfg.json` | Error `shares must be between 1 and 255, got 0`; exit 1 (config/quorum_key.rs:56-61; test config/quorum_key.rs:159-169). |
| V-8 | `cfg.json`: `shares` 2, `threshold` 1, keys [OPK1, OPK2] | `tvc keys generate-local-quorum-key --config-file cfg.json` | Error `threshold must be >= 2, got 1`; exit 1 (config/quorum_key.rs:62-67; test config/quorum_key.rs:171-179). |
| V-9 | `cfg.json`: `shares` 2, `threshold` 3, keys [OPK1, OPK2] | `tvc keys generate-local-quorum-key --config-file cfg.json` | Error `threshold (3) cannot exceed shares (2)`; exit 1 (config/quorum_key.rs:68-74; test config/quorum_key.rs:181-189). |
| V-10 | `cfg.json`: `shares` 3, `threshold` 2, keys [OPK1, OPK2] | `tvc keys generate-local-quorum-key --config-file cfg.json` | Error `operatorPublicKeys length (2) must equal shares (3)`; exit 1 (config/quorum_key.rs:76-82; test config/quorum_key.rs:122-134). |
| V-11 | `cfg.json`: `shares` 2, `threshold` 2, keys [OPK1, uppercase OPK1] | `tvc keys generate-local-quorum-key --config-file cfg.json` | Error `duplicate operator public key <OPK1> at index 1`; exit 1 (config/quorum_key.rs:88-90; test config/quorum_key.rs:136-157). |
| V-12 | Valid `cfg.json` as in V-2; `quorum_key_metadata.json` already exists | `tvc keys generate-local-quorum-key --config-file cfg.json` | Error `quorum key metadata file already exists: quorum_key_metadata.json`; exit 1; the existing file keeps its content (generate_local_quorum_key.rs:50-55). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT write the quorum master seed or any plaintext share to disk. | Plaintext shares exist only inside `generate_and_encrypt_shares`; `Zeroizing` clears each share on drop (generate_local_quorum_key.rs:41-42, 128-134). The only write serializes public keys and encrypted shares (generate_local_quorum_key.rs:61-68). |
| INV-2 | The command MUST NOT overwrite an existing file at the metadata output path. | Existence check before any generation (generate_local_quorum_key.rs:50-55). Behavioral check: V-12. |
| INV-3 | Metadata entry `i` MUST pair the normalized form of command config key `i` with the share encrypted to that exact key. | Index-aligned zip of parsed keys and generated shares (generate_local_quorum_key.rs:136-150); normalization re-encodes the parsed point (generate_local_quorum_key.rs:105). Behavioral check: decrypt round-trip (tests/keys_generate_local_quorum_key.rs:71-98). |
| INV-4 | The command MUST NOT contact the Turnkey API or any other network host. | `run` takes only `ctx` and `args`; dispatch withholds the tvc config, and the module imports no API client (cli.rs:292-294; generate_local_quorum_key.rs:1-17, 45). |

## Gaps (informative)

1. **[capability] Operator keys enter only as hand-copied raw hex: no registry resolution, no flags, no prompt**.
   The sibling `create-quorum-key` resolves `--operator-ids` (UUIDs) against the org's operator records and also accepts inline `--operator-encrypt-keys` (create_quorum_key.rs:33-67, 164-182).
   This command accepts only `operatorPublicKeys` in the command config.
   Dispatch withholds the loaded tvc config (cli.rs:292-294), so registry resolution stays structurally impossible today.
   `init-local-quorum-key` prefills exactly one key: the sole record of the active org's default operator kind (operator.rs:282-304).
   The user copies every other configured key (other operator kinds, extra YubiKeys) out of the tvc config, key files, or per-key commands (`backup-operator-key`, `refresh-yubikey`).
   No command lists all operator public keys; the `operator` group has only `create` (cli.rs:379-384).

2. **[capability] This command's output forms the upstream half of the re-encrypt operator-selection gap**.
   The metadata fixes forever which operator keys can re-encrypt: `re-encrypt-local-share` looks each share up by the resolved operator's public key (quorum_key_metadata.rs:29-49).
   That command resolves only the org's default operator kind, or an explicit `--operator-seed`/`--operator-seed-path` (operator.rs:395-441).
   Encrypting a share to an operator outside the default operator kind produces a stranded share.
   Examples: a second local key, or a YubiKey while the default operator kind is `local`.
   Recovery then needs the raw seed flags or a hand edit of the default operator kind.

3. **[bug?] Shares encrypted to hosted operator keys are dead ends, and `init` prefills one when the org's default operator kind is `hosted`**.
   `default_operator_public_key` returns the hosted composite key (operator.rs:291-297) and the template pastes it in (init_local_quorum_key.rs:34-36).
   This command then encrypts to it without complaint.
   `re-encrypt-local-share` refuses hosted operators outright (operator.rs:435-439).
   `deploy provision` provisions only Turnkey-side shares of hosted quorum keys by operator UUID (provision.rs:32-44); it never reads local metadata.
   No warning appears at generate time, and no tvc command can recover the share.

4. **[consistency] `shares` and `threshold` have no flag or environment variable, unlike the hosted sibling**.
   `create-quorum-key` exposes `--threshold`/`TVC_QUORUM_KEY_THRESHOLD` and `--operator-encrypt-keys`/`TVC_OPERATOR_ENCRYPT_KEYS` (create_quorum_key.rs:40-67).
   Here every substantive input lives in the command config alone (config/quorum_key.rs:21-25).
   The two quorum key creation commands take disjoint input mechanisms.
   Neither honors the full value resolution order that LONG_ABOUT documents (cli.rs:19-23).

5. **[docs] Help text omits the operator key format, and the format differs between siblings**.
   This command needs the 130-byte qos composite key (qos_p256 `P256Public::from_bytes` accepts exactly 130 bytes).
   `create-quorum-key` needs the 65-byte encrypt-only key (operator.rs:72-75 rejects anything else).
   Both help texts say only "operator ... keys" (`long_about = None`, generate_local_quorum_key.rs:20; cli.rs:466).
   A wrong-format paste yields `invalid QOS P-256 key` or `must be a 65-byte uncompressed P-256 public key` with no hint about which command wants which format.

6. **[consistency] Every semantic validation failure emits `command_error`, though the documented taxonomy assigns `invalid_input`**.
   LONG_ABOUT defines `invalid_input` as semantic validation failure in the command (cli.rs:56).
   `classify` recognizes only `MissingResource` and `TurnkeyClientError` (errors.rs:93-103), and nothing in the crate constructs `ErrorCode::InvalidInput` (sole mention: errors.rs:56).
   This command performs almost pure semantic validation, so every failure lands on the fallback.

7. **[consistency] The `init` to `generate` defaults do not chain, and no overwrite flag exists**.
   `init-local-quorum-key` writes its template to `quorum_key.json` by default (init_local_quorum_key.rs:18-25).
   Its outcome text tells the user to pass `--config-file <path>` (init_local_quorum_key.rs:63-64).
   `--config-file` here has no default; a default of `quorum_key.json` would make the documented two-step flow flagless.
   Separately, an existing metadata file always aborts the run (generate_local_quorum_key.rs:50-55).
   A rerun in the same directory first needs a manual delete (same pattern in `init`, init_local_quorum_key.rs:30-32).
