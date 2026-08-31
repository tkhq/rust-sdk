# tvc keys create-quorum-key

## Purpose
Creates a HOSTED quorum key: submits a `CreateTvcQuorumKey` activity that has Turnkey
generate a quorum key server-side, shamir-split it, and encrypt each share to one of
the supplied operator encryption public keys. Run it when setting up an app whose
quorum key (and share custody) lives in Turnkey rather than on operator machines —
the hosted counterpart to `keys generate-local-quorum-key`.
Implementation: `tvc/src/commands/keys/create_quorum_key.rs` (dispatch
`tvc/src/cli.rs:289-291`).

## Inputs
| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| threshold (shares to reconstruct) | `--threshold` (u8, min 2 via parser `create_quorum_key.rs:26-28`) | `TVC_QUORUM_KEY_THRESHOLD` | — | none, required | never |
| operator encrypt keys (65-byte uncompressed P-256, bare hex, comma-separated) | `--operator-encrypt-keys` | `TVC_OPERATOR_ENCRYPT_KEYS` | — | none | never |
| hosted operator UUIDs (comma-separated) | `--operator-ids` | `TVC_OPERATOR_IDS` | — | none | never |
| auth | — | `TVC_ORG_ID` / `TVC_API_KEY_PUBLIC` / `TVC_API_KEY_PRIVATE` (+optional `TVC_API_BASE_URL`) | active org in `~/.config/turnkey` | — | never |

- `--operator-encrypt-keys` and `--operator-ids` form a required, mutually exclusive
  ArgGroup (`create_quorum_key.rs:33-38`); exactly one source must be supplied.
- Standard flag > env order holds for all three args; there are no config-file keys
  for any of them (the local sibling instead takes a JSON config file).
- Auth deviates the usual way (global): env auth is all-or-nothing and beats the
  config file wholesale (`tvc/src/client.rs:48-64`); partial env auth errors.
- `--operator-ids` additionally requires the local config registry (active org with
  hosted operator records) even when auth comes from env
  (`create_quorum_key.rs:167-181`, `operator/hosted.rs:264-286`).

## Interactive behavior
None. The command never prompts: both required inputs are enforced by Clap at parse
time, so a missing threshold/operator source is a usage error (exit 2) in every mode —
the runtime `missing_required_input` path can never fire for this command.
Interactive and `--non-interactive`/JSON runs are behaviorally identical. Contrast:
sibling commands interactively select operators (`deploy approve`
`tvc/src/commands/deploy/approve.rs:345`, `keys re-encrypt-local-share`
`tvc/src/commands/keys/re_encrypt_local_share.rs:151`).

## Outputs
Human mode (`create_quorum_key.rs:78-88`):
```
Quorum Key ID: <id>
Quorum Public Key: <hex>
Share IDs: <id>, <id>, ...
```
JSON mode: one terminal message, reason `quorum_key_created`
(`tvc/src/outcome.rs:63`, tag rule `outcome.rs:30`), payload
`{quorumKeyId, quorumPublicKey, shareIds[]}` (`create_quorum_key.rs:70-76`, pinned by
test `create_quorum_key.rs:444-453`). Note the threshold is not echoed back, and
nothing is persisted locally — the user must capture `quorumPublicKey` by hand to put
it in an app config (`tvc/src/config/app.rs:16`).

## Side effects
- One Turnkey activity: `create_tvc_quorum_key` (`create_quorum_key.rs:137-141`) —
  creates the key and its encrypted shares server-side.
- No file writes and no config mutation by the command itself. (Global dispatch
  creates a default config file if none exists, `tvc/src/cli.rs:219-224`.)
- No YubiKey/device interaction.
- Local pre-flight before any network I/O: count/threshold validation, ID or key
  parsing, dedup after canonicalization (`create_quorum_key.rs:112-124`); the
  registry lookup for `--operator-ids` also precedes `build_client`.

## Failure modes
- Missing/conflicting args, threshold outside 2..=255, malformed hex key or UUID:
  Clap parse errors, exit 2 (`usage_error`; JSON envelope via `cli.rs:154-182`).
  Key parsing is `OperatorPublicKey::FromStr` (`tvc/src/operator.rs:63-81`): "must be
  bare hex encoded" / "must be a 65-byte uncompressed P-256 public key" / "is not a
  valid P-256 point".
- Runtime validation, all exit 1 and classify `command_error` (anyhow with no typed
  cause, `tvc/src/errors.rs:93-103`): operator count ≥ 255
  (`create_quorum_key.rs:207-213`); threshold > count (`:215-221`); duplicate IDs
  (`:184-193`) or duplicate keys post-normalization (`:195-205`, catches two IDs
  resolving to one key); no active org on the IDs path (`:168-170`); unknown operator
  ID in org (`operator/hosted.rs:273-275`); invalid stored hosted record
  (`hosted.rs:187-203`); authenticated org ≠ configured org, IDs path only
  (`create_quorum_key.rs:129-131`, `operator.rs:149-158`).
- API failure: `hosted_activity_error` preserves the typed `TurnkeyClientError`
  (`operator/hosted.rs:355-364`), so codes map per taxonomy — `unauthorized`,
  `not_found`, `api_error`, `approval_required` (activity needs more approvals),
  `network_error` — exit 1.
- Response validation: empty quorum key ID/public key/share ID, or share count ≠ key
  count (`create_quorum_key.rs:233-259`) → `command_error`, exit 1.

## Gaps
1. **[capability] `--operator-ids` resolves hosted operators only; registered local
   and YubiKey operators cannot be referenced at all.** Resolution filters the
   registry to hosted records (`operator/hosted.rs:280-286`,
   `config/turnkey/mod.rs:461-468`), yet local key files and the YubiKey key cache
   both hold composite public keys whose first 65 bytes are the encrypt key
   (`operator.rs:282-304`). To include such an operator the user must hand-extract
   hex and switch entirely to `--operator-encrypt-keys`. This is the canonical shape:
   registry state constrains what the user can explicitly pick.
2. **[capability] Operator sources cannot be mixed.** The ArgGroup is
   `multiple(false)` (`create_quorum_key.rs:33-38`; test
   `tests/keys_create_quorum_key.rs:94-112`), so "two registered hosted operators
   plus one external partner key" forces manual hex extraction for everything —
   the registry becomes unusable the moment one non-registry key participates.
3. **[capability] No interactive path.** Sibling commands prompt to select operators
   (`deploy/approve.rs:345`, `keys/re_encrypt_local_share.rs:151`) and the registry
   plus `known_operator_candidates` (`operator.rs:448-475`) has everything a picker
   needs; here the parse-time-required ArgGroup makes prompting structurally
   impossible. (Shared with `generate-local-quorum-key`, which also never prompts.)
4. **[capability] No file-based input and no `init` counterpart, unlike the local
   flow.** Local: `keys init-local-quorum-key` writes a template and
   `generate-local-quorum-key --config-file` consumes it
   (`generate_local_quorum_key.rs:23`, `init_local_quorum_key.rs:29-45`). Hosted: a
   large operator set (the command accepts up to 254 keys) must fit on the command
   line or in one env var.
5. **[consistency] Hosted max operator count is 254; local allows 255.**
   `create_quorum_key.rs:24,207-213` requires count < 255 while the comment cites
   qos supporting "at most 255", and the local path allows exactly 255
   (`config/quorum_key.rs:10,56-61`). No rationale is recorded for the stricter
   hosted bound; either the bound or the comment looks wrong.
6. **[docs] Arbitrary encrypt keys are accepted, but only hosted-operator keys have
   any downstream use.** `deploy provision` re-encrypts shares exclusively through a
   hosted registry operator (`deploy/provision.rs:38-39,79`) and no command fetches
   or decrypts a hosted share by `shareId` (share IDs appear nowhere else in the
   CLI). Shares encrypted to external keys via `--operator-encrypt-keys` are
   currently unreachable through tvc; neither the help text nor a long_about (there
   is none, `create_quorum_key.rs:32`) says so.
7. **[bug?] The source tie-break silently prefers `--operator-ids`, and
   env-vs-env exclusivity is untested.** `create_quorum_key.rs:112-118` picks
   `OperatorIds` whenever that vec is non-empty, relying on the Clap group; the only
   mutual-exclusion test passes both as flags
   (`tests/keys_create_quorum_key.rs:94-112`). If Clap does not fire the group
   conflict when both values arrive via env (`TVC_OPERATOR_IDS` +
   `TVC_OPERATOR_ENCRYPT_KEYS` set in CI), the encrypt keys are silently ignored.
   Unverified statically — worth a test either way.
8. **[consistency] Repeated flag occurrences replace rather than append.** Both list
   args use `ArgAction::Set` (`create_quorum_key.rs:53,64`), so
   `--operator-encrypt-keys A --operator-encrypt-keys B` yields only `[B]` — earlier
   values are dropped without warning, and the help text does not say lists must be
   comma-joined in a single occurrence.
9. **[consistency][docs] Key format differs from the local sibling without either
   side documenting it.** This command takes 65-byte encrypt-only keys
   (`operator.rs:72-75`); the local config takes 130-byte qos composite keys
   (`generate_local_quorum_key.rs:97-110`, template prefilled with a composite key
   via `init_local_quorum_key.rs:34` / `operator.rs:293-296`). A key copied from the
   local template into `--operator-encrypt-keys` fails with a length error and no
   pointer to the composite/encrypt-half distinction.
10. **[docs] Semantic validation errors emit `code: command_error`, not the
    `invalid_input` the global help promises.** `LONG_ABOUT` documents
    `invalid_input` for "semantic validation failed in the command"
    (`cli.rs:56-57`), but `classify` only maps typed errors
    (`errors.rs:93-103`) and this command's validations are plain `ensure!`s — every
    one of them lands on the `command_error` fallback. (Global gap, fully evidenced
    here.)
