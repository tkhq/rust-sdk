# tvc keys generate-local-quorum-key

## Purpose
Generates a fresh quorum key pair locally, Shamir-splits its master seed into `shares`
pieces with reconstruction threshold `threshold`, encrypts each share to one operator
public key from a JSON config file, and writes the result (quorum public key + encrypted
shares) as a quorum-key-metadata JSON file. Fully offline: no Turnkey API calls, no
device access. Run after `tvc keys init-local-quorum-key`; the metadata is later consumed
by `tvc keys re-encrypt-local-share`. The plaintext quorum key and shares exist only in
memory (zeroized on drop, generate_local_quorum_key.rs:41-42) and are never written out.

## Inputs
| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| quorum key config path | `-c`/`--config-file` | `TVC_QUORUM_KEY_CONFIG` | — | none (required) | no |
| metadata output path | `--quorum-key-metadata-out` | `TVC_QUORUM_KEY_METADATA_OUT` | — | `quorum_key_metadata.json` | no |
| shares | — | — | `shares` (config JSON) | none | no |
| threshold | — | — | `threshold` (config JSON) | none | no |
| operator public keys | — | — | `operatorPublicKeys` (config JSON) | none | no |

The substantive inputs (shares, threshold, operator keys) live only at layer 3 of the
flag > env > config > default ladder — there is no flag or env equivalent for any of
them (generate_local_quorum_key.rs:19-34). Operator keys must be 130-byte qos composite
public keys (encrypt||sign) in bare hex, case-insensitive, whitespace-trimmed
(quorum_key_metadata.rs:52-57); they are normalized to lowercase in the output. The
config requires `operatorPublicKeys.len() == shares`, `1 <= shares <= 255`,
`2 <= threshold <= shares`, no duplicates, no `<FILL_IN` placeholders
(config/quorum_key.rs:52-94).

Global `--non-interactive` is a no-op here: the command never prompts. Dispatch does not
pass the loaded TVC config to this command (cli.rs:292-294), so nothing in
`~/.config/turnkey/config.toml` (org, operator registry) influences behavior.

## Interactive behavior
None. No prompts in either mode; behavior is identical under `--non-interactive` and
`--message-format json`. `ctx` is unused (generate_local_quorum_key.rs:45).

## Outputs
Human mode (stdout):
```
Quorum key metadata written to: <path>
Quorum Public Key: <hex>
Threshold: <n>
```
JSON mode: one NDJSON line, reason `quorum_key_generated`, fields `quorumKeyPublic`,
`threshold`, `metadataPath` (outcome.rs:64, verified by
tests/keys_generate_local_quorum_key.rs:167-171).

## Side effects
- Reads the quorum key config JSON (`--config-file`).
- Writes the metadata JSON to `--quorum-key-metadata-out` (refuses to overwrite,
  generate_local_quorum_key.rs:50-55). Written with plain `fs::write` (default perms) —
  acceptable, since it holds only public keys and encrypted shares.
- Pre-dispatch, like every command except `yubikey create-certs`: loads
  `~/.config/turnkey/config.toml` and **creates it if missing** (cli.rs:215-240) even
  though this command never uses it.
- No Turnkey API calls, no YubiKey interaction.

## Failure modes
All runtime failures are untyped `anyhow` errors → JSON `code: command_error`, exit 1
(errors.rs:93-103):
- config file unreadable/unparsable ("failed to read/parse quorum key config file", util.rs:18-28)
- placeholder keys, share/threshold bounds, duplicate keys, key-count mismatch (config/quorum_key.rs:52-94)
- invalid operator key hex/point ("invalid operator public key at index N", generate_local_quorum_key.rs:97-110)
- output file already exists (generate_local_quorum_key.rs:50-55)
- share generation/encryption or metadata write failure (generate_local_quorum_key.rs:124-150, 61-68)
- pre-dispatch: `HOME` unset or malformed `~/.config/turnkey/config.toml` (cli.rs:215-230)

Missing `--config-file` (no flag, no env) is a clap error → `usage_error`, exit 2.

## Gaps
1. **[capability] Operators can only be supplied as hand-copied raw hex — no registry
   resolution, no flags, no prompt.** Sibling `create-quorum-key` resolves
   `--operator-ids` (UUIDs) against the org's operator registry and also accepts inline
   `--operator-encrypt-keys` (create_quorum_key.rs:33-67, 164-182); this command accepts
   only `operatorPublicKeys` in a JSON file, and dispatch doesn't even pass the loaded
   `Config` (cli.rs:292-294), so registry resolution is structurally impossible today.
   `init-local-quorum-key` prefills exactly one key — the active org's *default*
   backend's sole record (operator.rs:282-304) — leaving every other configured operator
   (other kinds, multiple YubiKeys) to be scraped from `config.toml`, key files, or
   per-key commands (`backup-operator-key`, `refresh-yubikey`); no command lists all
   operator public keys (`operator` has only `create`, cli.rs:379-384).

2. **[capability] This command's output is the upstream half of the canonical
   re-encrypt operator-selection gap.** The metadata fixes forever which operator keys
   can re-encrypt (share looked up by the resolved operator's public key,
   quorum_key_metadata.rs:29-49), while `re-encrypt-local-share` can only resolve the
   org's *default-kind* operator or an explicit `--operator-seed[-path]`
   (operator.rs:395-441). Encrypting a share here to any operator outside the default
   backend (e.g. a second local key, or a YubiKey when the default is local) produces a
   share usable only via raw-seed flags or by hand-editing `default_operator_kind`.

3. **[bug?] Shares encrypted to hosted operator keys are dead ends — and `init`
   prefills one when the org default is hosted.** `default_operator_public_key`
   happily returns the hosted composite (operator.rs:291-297), the template pastes it
   in (init_local_quorum_key.rs:34-36), and this command encrypts to it without
   complaint — but `re-encrypt-local-share` refuses hosted operators outright
   (operator.rs:435-439) and `deploy provision` provisions only Turnkey-side shares of
   *hosted* quorum keys by operator UUID (provision.rs:32-44), never local metadata.
   No warning at generate time; the share is unrecoverable by any tvc command.

4. **[consistency] shares/threshold have no flag or env equivalent, unlike the hosted
   sibling.** `create-quorum-key` exposes `--threshold`/`TVC_QUORUM_KEY_THRESHOLD` and
   `--operator-encrypt-keys`/`TVC_OPERATOR_ENCRYPT_KEYS` (create_quorum_key.rs:40-67);
   here everything is config-file-only (config/quorum_key.rs:21-25). The two quorum-key
   creation commands take entirely disjoint input mechanisms, and neither honors the
   full flag > env > config ladder LONG_ABOUT documents (cli.rs:19-23).

5. **[docs] The operator key format is undocumented and differs between siblings.**
   This command needs the 130-byte qos composite key (encrypt||sign;
   qos_p256 `P256Public::from_bytes` requires exactly 2×65 bytes), while
   `create-quorum-key` needs the 65-byte encrypt-only key (operator.rs:72-75 rejects
   anything else). Both help texts just say "operator ... keys" (`long_about = None`,
   generate_local_quorum_key.rs:20; cli.rs:466); pasting the wrong one yields
   "invalid QOS P-256 key"/"must be a 65-byte uncompressed P-256 public key" with no
   hint about which command wants which format.

6. **[consistency] All semantic validation failures emit `command_error`, though the
   documented taxonomy assigns them `invalid_input`.** LONG_ABOUT defines
   `invalid_input` as "semantic validation failed in the command" (cli.rs:58), but
   `classify` recognizes only `MissingResource`/`TurnkeyClientError` (errors.rs:93-103)
   and nothing in the crate ever constructs `ErrorCode::InvalidInput` (sole mention:
   errors.rs:56). This command is nearly all semantic validation, so every failure
   lands on the fallback.

7. **[consistency] The `init` → `generate` defaults don't chain, and there is no
   `--force`.** `init-local-quorum-key` defaults its output to `quorum_key.json`
   (init_local_quorum_key.rs:18-25) and its outcome text tells the user to pass
   `--config-file <path>` (init_local_quorum_key.rs:63-64), but `--config-file` here
   has no default — the natural default `quorum_key.json` would make the documented
   two-step flow flagless. Separately, an existing metadata file always aborts
   (generate_local_quorum_key.rs:50-55) with no `--force`/`--overwrite`, so a rerun in
   the same directory requires manual deletion (same pattern in `init`,
   init_local_quorum_key.rs:30-32).
