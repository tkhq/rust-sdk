# tvc keys re-encrypt-local-share

## Purpose

Decrypts one quorum-key share (from `keys generate-local-quorum-key` metadata) with a
local or YubiKey operator key, re-encrypts it to a deployment's attested ephemeral key,
and signs a share approval — the offline middle step of the manual provisioning flow
(`deploy provisioning-details` -> `keys re-encrypt-local-share` -> `deploy post-share`).
Fully offline: no Turnkey API calls; attestation and manifest-approval verification run
locally (provisioning.rs:206-240).

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| quorum key metadata file | `--quorum-key-metadata <PATH>` | `TVC_QUORUM_KEY_METADATA` | none | none (required) | never |
| provision bundle file | `--provision-bundle <PATH>` | `TVC_PROVISION_BUNDLE` | none | none (required) | never |
| operator seed (hex) | `--operator-seed <HEX_SEED>` | `TVC_OPERATOR_SEED` | none | none | never |
| operator seed file (raw hex) | `--operator-seed-path <PATH>` | `TVC_OPERATOR_SEED_PATH` | none | none | never |
| YubiKey serial | `--serial <SERIAL>` | none | none | sole registered YubiKey | yes (multi-YubiKey case only) |
| skip verification | `--dangerous-skip-verification` | `TVC_DANGEROUS_SKIP_VERIFICATION` | none | false | never |
| output path | `--re-encrypted-out <PATH>` | `TVC_RE_ENCRYPTED_OUT` | none | none (inline to stdout) | never |
| decrypting operator (backend + record) | none | none | `orgs.<alias>.default_operator_kind` + sole operator record of that kind | org default | YubiKey PIN always; record only in the multi-YubiKey case |

Deviation from flag > env > config > default: the operator itself has no flag or env at
all (other than the raw-seed escape hatch) — the persisted org config alone decides which
backend is used (operator.rs:411-440). `--operator-seed`/`--operator-seed-path` are
mutually exclusive (local_operator_key.rs:21-33) and, when given, silently win over
`--serial` and the config (re_encrypt_local_share.rs:127-128, operator.rs:400-402).

## Interactive behavior

Prompts exist only on the YubiKey path, i.e. when no seed flag is given AND the active
org's `default_operator_kind == yubikey` (re_encrypt_local_share.rs:125-128):

1. Several YubiKey operators registered and no `--serial`: `Select YubiKey operator`
   select prompt (re_encrypt_local_share.rs:135-153).
2. Always: `YubiKey PIV PIN` password prompt (re_encrypt_local_share.rs:171-173). The PIN
   is never read from config or the environment, by design.

Both prompts settle before the input files are read; device I/O waits until the files
parse (re_encrypt_local_share.rs:179-186).

Non-interactive / JSON mode:
- Multi-YubiKey without `--serial` -> `missing_required_input` for `--serial`
  (re_encrypt_local_share.rs:155-158).
- Any YubiKey-default run -> hard error, "a YubiKey operator needs its PIN typed at an
  interactive prompt" (re_encrypt_local_share.rs:164-169; pinned by
  tests/keys_re_encrypt_local_share.rs:158). So non-interactive runs work only with a
  local default or the raw-seed flags.
- Local-default runs never prompt in either mode.

## Outputs

- Human, no `--re-encrypted-out`: pretty-printed share JSON on stdout
  (re_encrypt_local_share.rs:95-110). With `--re-encrypted-out`: `Re-encrypted share
  written to: <path>` narration on stderr, nothing on stdout (rs:104-107, 290).
- JSON: one `reason: "re_encrypted_share_generated"` message, either with the flattened
  payload (`deploymentId`, `ephemeralPublicKeyHex`, `reEncryptedShare`, `shareApproval`)
  or with `writtenTo` (tests rs:697-733).
- `--dangerous-skip-verification` prints a WARNING to stderr up front (rs:117-122).

## Side effects

- Reads: quorum key metadata JSON, provision bundle JSON (rs:183-186); the registered
  operator key file or raw seed file; the YubiKey registry cache.
- YubiKey device: opened by serial; PIN verified; one touch for share decryption
  (key-agreement) plus one for the approval signature (rs:188, 217-231).
- Writes: only the `--re-encrypted-out` file, when given (rs:286-290).
- No Turnkey API calls; no config mutation. (The config is *read* to pick the backend and
  the sole record of that kind.)

## Failure modes

- Missing `--quorum-key-metadata`/`--provision-bundle`: clap usage error, exit 2.
- Both seed flags: "mutually exclusive" error (local_operator_key.rs:23-27), exit 1,
  code `command_error`.
- No seed and no active org: "No active organization..." (operator.rs:404-409).
- Hosted default: hard bail redirecting to `tvc deploy provision` (operator.rs:433-439;
  tests/keys_re_encrypt_local_share.rs:127).
- Local default with zero or multiple local records: `no local operator is configured` /
  `multiple local operators are configured` (config/turnkey/mod.rs:441-457) — the
  multiple case has no disambiguator on this command.
- `--serial` not registered: `no YubiKey operator has serial ...`
  (config/turnkey/mod.rs:509-512), refused before file I/O (tests rs:184).
- Multi-YubiKey, non-interactive, no `--serial`: `missing_required_input`.
- Quorum key metadata vs manifest mismatch (rs:242-259), operator not in metadata shares
  (quorum_key_metadata.rs:46-48), operator not in share set (rs:261-279), attestation /
  approval verification failures (provisioning.rs:160-240): all `command_error`, exit 1.

## Gaps

1. **[capability] A registered local operator is unusable unless the org's persisted
   default backend is `local` — the user cannot pick the operator at all.** Verified form
   of Richard's observation: with no seed flag, the backend comes solely from
   `org.default_operator_kind` (operator.rs:411-440); no flag, env, or prompt selects a
   kind or record on this command (Args, re_encrypt_local_share.rs:28-71). That default
   is org-level persisted state, flipped to `hosted`/`yubikey` by `operator create
   --make-default` (create.rs:230-232, 399-401; interactively prompted at create.rs:320-325)
   and set at org creation (config/turnkey/mod.rs:643-651); nothing ever sets it back to
   `local` except hand-editing tvc.config.toml (no `operator set-default` exists). So the
   precise condition is "default kind is local" — in practice, whichever operator was
   most recently made default — not strictly "last-used", but the effect Richard
   described holds: a configured local operator is unreachable once a YubiKey or hosted
   operator became the default. The escape hatch is worse than it looks: the registered
   key file is `StoredQosOperatorKey` JSON, but `--operator-seed-path` parses raw hex
   (local_operator_key.rs:79-99), so the user must manually extract `private_key` from
   the JSON. Target UX per Richard: pick ANY configured operator — local, YubiKey, and
   today hosted (hosted may be dropped from this path later; noted as context, not
   specced away).

2. **[consistency] Sibling commands let the user choose an operator; this one hard-codes
   the choice.** `deploy approve` enumerates every registered operator of every kind as a
   candidate, filters by manifest-set membership, accepts a mutually exclusive selector
   group (`--operator-id`/`--serial`/`--operator-seed`/`--operator-seed-path`,
   approve.rs:52-57) and prompts `Select approving operator` when ambiguous
   (approve.rs:242-346); config defaults are "intentionally absent from this boundary"
   (operator.rs:306-307). `deploy provision` requires an explicit `--operator-id`
   (provision.rs:37-39). Even `keys backup-operator-key` ignores the default kind and
   reaches the local record directly (backup_operator_key.rs:63-79). re-encrypt is the
   only command in the share flow where persisted default state constrains an explicit
   choice. It also already has the data for approve-style matching: the metadata names
   each share's operator by public key (quorum_key_metadata.rs:29-49) and can hold shares
   for several operators (generate_local_quorum_key.rs:113-150), so candidates could be
   filtered to operators that actually hold a share.

3. **[capability] A hosted default blocks the command outright even when a local or
   YubiKey operator with a share is also registered.** The hosted arm bails with a
   redirect to `tvc deploy provision` (operator.rs:433-439) without consulting the other
   registered records or the metadata's share list — the org must flip
   `default_operator_kind` in the TOML to proceed. (Legitimate core: a hosted operator
   cannot decrypt locally — re-encryption needs decrypt capability,
   re_encrypt_local_share.rs:204-206 — so hosted itself belongs on the `deploy provision`
   path; the gap is that its mere *defaultness* blocks the local/YubiKey operators.)

4. **[consistency] `--serial` is silently ignored unless the default kind is Yubikey and
   no seed flag is given.** The whole YubiKey selection block is gated on
   `operator_seed_source.is_none() && org.default_operator_kind == OperatorKind::Yubikey`
   (re_encrypt_local_share.rs:126-128), so `--serial` with a local/hosted default, or
   alongside `--operator-seed`, does nothing — no conflict error. `deploy approve` places
   `--serial` in an exclusive ArgGroup (approve.rs:52-57) and validates it against the
   org config up front (approve.rs:155-163).

5. **[consistency] `--serial` has no env var.** Every other input on this command has one
   (re_encrypt_local_share.rs:32-70); `--serial` is flag-only (rs:59-62). Same omission
   as `deploy approve --serial` (approve.rs:91-97) — inconsistent with this command's own
   flags rather than with siblings.

6. **[docs] The seed-flag help text misstates the fallback.** "If no seed flag is
   provided, uses the operator key from the logged-in org config"
   (re_encrypt_local_share.rs:40-41) — omits that the org's default backend *kind*
   decides, that a hosted default is refused entirely, and that a YubiKey default needs
   an interactive PIN. `long_about = None` (rs:27) leaves nowhere else to learn this
   before hitting the errors.
