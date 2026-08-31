# tvc operator create

## Purpose

Creates one TVC operator and saves it to the active organization in
`~/.config/turnkey/tvc.config.toml`. Two kinds: `hosted` (default) submits a
`create_tvc_operator` activity to Turnkey, which mints a wallet + encrypt/sign
accounts and returns the operator identity; `yubikey` adds a serial-only org
record referencing an already-registered YubiKey (no device I/O, no network).
Run it after `tvc login` to add signing operators to an org. Sole subcommand of
the `operator` group (tvc/src/cli.rs:379-384).

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| operator kind | `--kind hosted\|yubikey` | `TVC_OPERATOR_KIND` | — | `hosted` | never |
| operator name | `--name` | `TVC_OPERATOR_NAME` | — | `tvc-operator` (hosted), `yubikey-<serial>` (yubikey) | never |
| new wallet name (hosted) | `--wallet-name` | `TVC_OPERATOR_WALLET_NAME` | — | `tvc-wallet` | never |
| existing wallet UUID (hosted) | `--wallet-id` | `TVC_OPERATOR_WALLET_ID` | — | none (new wallet) | never |
| base derivation path (hosted) | `--account-path` | `TVC_OPERATOR_ACCOUNT_PATH` | — | `m/5527107'/0'/0'` | never |
| YubiKey serial (yubikey) | `--serial` | — | — | sole registered key, else prompt | yes (multi-key select) |
| make org default kind | `--default` | — | — | false | yes (yubikey arm only) |

- `--wallet-name`/`--wallet-id` are a clap ArgGroup (mutually exclusive);
  `--serial` declares clap conflicts with all three wallet flags
  (tvc/src/commands/operator/create.rs:43-47, 84-91).
- Kind-compat is enforced post-parse: `--serial` with `--kind hosted`, or any
  wallet/path flag with `--kind yubikey`, is an `ensure!` failure in
  `CreatePlan::try_from` (create.rs:129-132, 152-155).
- No input has a config-file source; `--serial` and `--default` have no env
  var (siblings are consistent: no serial flag anywhere has an env var).
- Auth (hosted arm) follows the global contract: `TVC_ORG_ID`/
  `TVC_API_KEY_PUBLIC`/`TVC_API_KEY_PRIVATE` beat the org's on-disk API key
  (tvc/src/client.rs:38-63); the authenticated org must equal the configured
  active org (create.rs:195, tvc/src/operator.rs:149-158).

## Interactive behavior

Hosted arm: never prompts, in any mode. All inputs default.

YubiKey arm (`can_prompt = !non_interactive && stdin is a TTY`, create.rs:283):
1. If `--serial` omitted and `can_prompt`: zero registered YubiKeys → bail
   ("run `tvc keys refresh-yubikey` first", create.rs:307-310); exactly one →
   auto-selected silently (create.rs:311); several → `select` prompt "YubiKey
   to use as the operator" (create.rs:312).
2. Unless `--default` was passed: `confirm` prompt "Make this the default
   operator for the organization?" (default No), only when `can_prompt`
   (create.rs:320-325).

Non-interactive / JSON / piped stdin: `--serial` becomes a hard requirement —
the check fires before the registry is even read, so it fails even when
exactly one YubiKey is registered (create.rs:285-287; pinned by
tvc/tests/operator_create.rs:170-182). `--default` falls back to the flag
alone. JSON mode implies non-interactive (tvc/src/output.rs:206-213).

## Outputs

Human, hosted (create.rs:426-446): `Hosted operator created!` block — operator
name, operator ID, wallet ID, encryption/signing/composite public keys, and a
literal `Saved: true`.

Human, yubikey (create.rs:467-485): `YubiKey operator added!` block — name,
serial, composite operator public key, plus "It is now the organization's
default operator." when defaulted.

JSON (camelCase; reasons from tvc/src/outcome.rs:34-35):
- `reason: "operator_created"` — `name`, `operatorId`, `walletId`,
  `encryptPublicKey`, `signPublicKey`, `compositePublicKey`, `saved`
  (create.rs:413-424).
- `reason: "yubikey_operator_added"` — `name`, `serial`,
  `operatorPublicKey`, `madeDefault` (`org_alias` is serde-skipped;
  create.rs:448-459).

## Side effects

- Reads `~/.config/turnkey/tvc.config.toml`; the dispatcher creates a default
  config file if none exists (tvc/src/cli.rs:219-223).
- Hosted: one Turnkey activity `create_tvc_operator` (create.rs:211-215) —
  creates a remote wallet (or accounts in `--wallet-id`'s wallet) and a remote
  operator. Network + credentials required.
- YubiKey: no device interaction ever ("Never modifies a device",
  create.rs:84-91); the cached registry public key is reused
  (create.rs:379-387).
- Both arms append an `OperatorRecord` to the active org and rewrite the whole
  config file via `Config::save` (create.rs:228, 353); `--default` (or the
  prompt) flips `org.default_operator_kind` to the created kind
  (create.rs:230-232, 399-401).
- On save failure the error embeds a paste-ready recovery TOML fragment
  (`recovery_toml`, create.rs:504-514). Hosted warns that re-running would
  create another remote operator (create.rs:256-263); yubikey says re-running
  is safe because duplicate serials are refused (create.rs:335-359).

## Failure modes

- Flag conflicts / malformed UUID or serial / empty strings → clap usage
  error: exit 2, JSON `code: usage_error` (tvc/src/cli.rs:154-182).
- Kind-compat `ensure!`s (`--serial` with hosted, wallet flags with yubikey)
  → runtime anyhow: exit 1, `code: command_error` (create.rs:129-132,
  152-155).
- Missing `--serial` non-interactively → `MissingRequiredInput`: reason
  `missing_required_input`, `code: missing_required_input`, exit 1
  (create.rs:285-287, tvc/src/output.rs:326-330).
- No active org (both arms) → `command_error` "No active organization. Run
  `tvc login` first." (create.rs:189-192, 289-291).
- Env-auth org ≠ configured org → `command_error` (tvc/src/operator.rs:149-158).
- Hosted API failure → `hosted_activity_error` preserves
  `TurnkeyClientError`, classifying to `unauthorized`/`not_found`/`api_error`/
  `approval_required`/`network_error` per `classify`
  (tvc/src/operator/hosted.rs:355-364, tvc/src/errors.rs:93-103).
- Malformed creation result (non-UUID ids, bad or identical public keys) →
  `command_error` (hosted.rs:122-158, 89-99).
- Unregistered / duplicate serial, empty registry → `command_error` with
  `keys refresh-yubikey` remediation (create.rs:296-310, 379-386;
  tvc/src/config/turnkey/mod.rs:530-549).
- Config save failure after remote creation → exit 1 with recovery TOML; the
  remote operator exists but is absent locally (create.rs:239-263).

## Gaps

1. **[capability] A local operator cannot be created here — or anywhere except
   the login new-org flow.** `CreateKind` is only `Hosted | Yubikey`
   (create.rs:31-37), yet local is a first-class consumable kind everywhere
   else: `deploy approve` accepts registered local operators and seeds
   (tvc/src/commands/deploy/approve.rs:52-57, 99-116), `keys
   re-encrypt-local-share` and `keys backup-operator-key` need a local record
   (tvc/src/operator.rs:411-419, tvc/src/commands/keys/backup_operator_key.rs:64),
   and `OperatorKind::Local` is the config default
   (tvc/src/config/turnkey/mod.rs:273-280). `OperatorRecord::local` is only
   reachable through `Config::add_org` during `tvc login` org creation
   (config/turnkey/mod.rs:634-651, tvc/src/commands/login.rs:61-74, 678-710) —
   an existing org (e.g. one created with a YubiKey default) can never gain a
   local operator via the CLI; only hand-editing the config works.

2. **[capability] `--default` at create time is the only CLI control over
   `default_operator_kind`, and it can only move it away from local.** The
   sole non-login write sites for the field are create.rs:231 and
   create.rs:400; there is no `operator set-default`/`list`/`remove` (the
   group has exactly one subcommand, cli.rs:379-384). Once flipped to hosted
   or yubikey, restoring a local default requires hand-editing the config —
   the exact state-over-choice shape that leaves `keys
   re-encrypt-local-share` unable to use a configured local operator
   (operator.rs:411-441 dispatches on `default_operator_kind` only).

3. **[consistency] Non-interactive YubiKey creation hard-requires `--serial`
   even when exactly one YubiKey is registered.** The requirement fires
   before the registry is consulted (create.rs:285-287), so the sole-key
   auto-select at create.rs:311 is unreachable in CI/JSON mode; interactively
   the same sole key is used with no prompt. Sibling `tvc login` accepts an
   omitted serial non-interactively when the org has a sole YubiKey record
   (login.rs:91-93, 403-414, 550-561).

4. **[consistency] Only the YubiKey arm offers the make-default choice
   interactively.** YubiKey creation confirms "Make this the default
   operator...?" when it can prompt (create.rs:320-325); hosted creation
   never asks — hosted users must know about `--default` up front
   (create.rs:188-277 contains no prompt).

5. **[consistency][bug?] Nothing guards against a second hosted operator,
   which quietly breaks every sole-hosted consumer.** The yubikey arm refuses
   a duplicate serial (config/turnkey/mod.rs:530-540) but the hosted arm
   appends unconditionally — duplicate default name `tvc-operator` included
   (create.rs:141-149, 228). With two hosted records,
   `select_hosted_operator` errors `MultipleHostedOperators`
   (config/turnkey/mod.rs:473-483), failing `tvc login` for a hosted-default
   org (login.rs:530-533) and blanking `default_operator_public_key`
   (operator.rs:291-296). Create neither warns nor offers a way to pick among
   several hosted operators later (no per-operator default exists).

6. **[docs] The one-operator-per-wallet path constraint lives only in a Rust
   doc comment.** `DEFAULT_HOSTED_OPERATOR_BASE_PATH` docs state callers
   creating a second operator in the same wallet must supply a different base
   path themselves (tvc/src/operator/hosted.rs:26-28), but `--wallet-id` help
   (create.rs:70-73) and `--account-path` help (create.rs:75-77) never say
   so, and the command does not vary the path or pre-flight the collision —
   the failure surfaces as an opaque server error.

7. **[consistency] Kind-compat mistakes classify as `command_error`/exit 1
   while near-identical flag conflicts are `usage_error`/exit 2, and
   `ErrorCode::InvalidInput` is dead code.** `--serial --wallet-name` is a
   clap conflict (exit 2, usage_error) but `--kind hosted --serial` is a
   runtime `ensure!` (exit 1, command_error) (create.rs:129-132 vs 84-91).
   The taxonomy's `invalid_input` code — documented in LONG_ABOUT
   (cli.rs:56) — is `#[allow(dead_code)]` and never assigned
   (tvc/src/errors.rs:54-56), so semantic validation is indistinguishable
   from arbitrary failures in JSON mode. Env-sourced values (e.g. an exported
   `TVC_OPERATOR_WALLET_NAME` with `--kind yubikey`) trip the same ensures
   with no flag typed (create.rs:62-68, 152-155).

8. **[docs] `saved` in `operator_created` is vestigial — it is always
   `true`.** The struct field is set literally once (create.rs:275) and the
   human rendering hard-codes `Saved: true` (create.rs:437); a failed save
   aborts with an error before the outcome is emitted (create.rs:239-263), so
   the field can never be false and misleadingly implies a partial-success
   mode exists.
