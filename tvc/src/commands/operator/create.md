# tvc operator create

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the operator create command.*

## Purpose (informative)

`tvc operator create` creates one TVC operator and saves it as an operator record on the active org.
The `hosted` operator kind (the default) submits a `create_tvc_operator` activity.
Turnkey mints a wallet with encrypt and sign accounts and returns the operator identity.
The `yubikey` operator kind adds a serial-only operator record for a YubiKey that the local registry already holds.
That arm performs no device I/O and no network I/O.
Run this command after `tvc login` to add signing operators to an org.
It is the sole subcommand of the `operator` group (tvc/src/cli.rs:380-383).
In this specification, create.rs means tvc/src/commands/operator/create.rs.

## Acceptance scenario (normative)

| Step | Action | Expected observation |
|---|---|---|
| 1 | Start from a clean home directory. Write a tvc config with active org `default` (id `org-123`) and one registered YubiKey with serial `01c95c1f`. | The tvc config at `~/.config/turnkey/tvc.config.toml` lists the YubiKey under `[[yubikeys]]`. |
| 2 | Run `tvc operator create --kind yubikey --serial 01c95c1f --default` with `TVC_NON_INTERACTIVE=1`. | stdout prints `YubiKey operator added!` with operator name `yubikey-01c95c1f` and `It is now the organization's default operator.`. Exit code 0. |
| 3 | Read the tvc config. | Org `default` holds an operator record with `name = "yubikey-01c95c1f"` and `default_operator_kind = "yubikey"`. |
| 4 | Run the same command again. | The command fails with `YubiKey 01c95c1f is already an operator of this organization`. Exit code 1. |

All steps pass in one run from a clean start (pinned by tvc/tests/operator_create.rs:237-274).

## Inputs (normative)

Each input MUST resolve per the Part 00 value resolution order.
No input reads the tvc config or a command config.
`--serial` and `--default` have no environment variable.

| Input | Flag | Env | Default | Prompt |
|---|---|---|---|---|
| operator kind | `--kind hosted\|yubikey` | `TVC_OPERATOR_KIND` | `hosted` | never |
| operator name | `--name` | `TVC_OPERATOR_NAME` | `tvc-operator` (hosted), `yubikey-<serial>` (yubikey) | never |
| new wallet name (hosted) | `--wallet-name` | `TVC_OPERATOR_WALLET_NAME` | `tvc-wallet` | never |
| existing wallet UUID (hosted) | `--wallet-id` | `TVC_OPERATOR_WALLET_ID` | none (create a new wallet) | never |
| base derivation path (hosted) | `--account-path` | `TVC_OPERATOR_ACCOUNT_PATH` | `m/5527107'/0'/0'` | never |
| YubiKey serial (yubikey) | `--serial` | none | sole registry entry, else prompt | select, interactive mode only |
| make default operator kind | `--default` | none | false | confirm, yubikey arm only |

The parser MUST reject `--wallet-name` together with `--wallet-id` (clap ArgGroup, create.rs:43-47).

The parser MUST reject `--serial` together with `--wallet-name`, `--wallet-id`, or `--account-path` (create.rs:86-91).

After parsing, the command MUST reject `--serial` with the `hosted` operator kind (create.rs:129-132).

After parsing, the command MUST reject `--wallet-name`, `--wallet-id`, or `--account-path` with the `yubikey` operator kind (create.rs:152-155).
These post-parse checks also fire on values that arrive through environment variables (create.rs:62-82).

The hosted arm authenticates per the global contract: `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` take precedence over the active org's stored credentials (tvc/src/client.rs:38-63).
The authenticated org id MUST equal the active org's id (create.rs:195, tvc/src/operator.rs:149-158).

## Interactive behavior (normative)

The hosted arm MUST NOT prompt in any mode. Every hosted input has a default (create.rs:127-150).

The yubikey arm MAY prompt only in interactive mode (create.rs:283). Two prompts exist.

When `--serial` is absent in interactive mode, the registry state selects the behavior (create.rs:304-313).

| Registry state | Behavior |
|---|---|
| Zero YubiKeys | Fail with the `tvc keys refresh-yubikey` remediation (create.rs:307-310). |
| Exactly one YubiKey | Use it with no prompt (create.rs:311). |
| Two or more YubiKeys | Select prompt `YubiKey to use as the operator` (create.rs:312). |

When `--default` is absent and prompting is possible, the yubikey arm MUST show the confirm prompt `Make this the default operator for the organization?` with default No (create.rs:320-325).

In non-interactive mode the yubikey arm MUST require `--serial` (create.rs:285-287).
This check runs before the registry read, so it fails even when the registry holds exactly one YubiKey (tvc/tests/operator_create.rs:170-182).
Without `--default`, a non-interactive run keeps the org's default operator kind unchanged (create.rs:320-325).
JSON mode implies non-interactive mode (INV-G1).

## Outputs (normative)

In human mode the hosted arm MUST print a `Hosted operator created!` block (create.rs:426-446).
The block lists operator name, operator ID, wallet ID, encryption public key, signing public key, composite public key, and the literal line `Saved: true`.

In human mode the yubikey arm MUST print a `YubiKey operator added!` block with operator name, serial, and operator public key (create.rs:467-485).
When the invocation sets the default operator kind, the block MUST end with `It is now the organization's default operator.` (create.rs:469-473).

In JSON mode the command MUST emit exactly one NDJSON object with camelCase fields (reasons: tvc/src/outcome.rs:30-35).

| `reason` | Fields |
|---|---|
| `operator_created` | `name`, `operatorId`, `walletId`, `encryptPublicKey`, `signPublicKey`, `compositePublicKey`, `saved` (create.rs:413-424). |
| `yubikey_operator_added` | `name`, `serial`, `operatorPublicKey`, `madeDefault`; serde skips `org_alias` (create.rs:448-459). |

`compositePublicKey` MUST equal `encryptPublicKey` followed by `signPublicKey` (create.rs:267-270).

## Side effects (normative)

- tvc config load and creation follow INV-G4.
- The hosted arm MUST submit exactly one `create_tvc_operator` activity (create.rs:211-215). The activity creates a new wallet, or accounts inside the `--wallet-id` wallet, plus the remote operator.
- The yubikey arm MUST NOT touch a device. It reuses the registry's cached public key (create.rs:379-387).
- Each arm MUST append one operator record to the active org (create.rs:228, 397). Each arm MUST save the whole tvc config once (create.rs:239, 353).
- With `--default`, or a confirmed prompt, the command MUST set the org's default operator kind to the created operator kind (create.rs:230-232, 399-401).
- On save failure the error MUST embed a paste-ready recovery TOML fragment (create.rs:504-514). The hosted message warns that a retry creates another remote operator (create.rs:256-262). The yubikey message states that a retry is safe (create.rs:335-359).

## Failure modes (normative)

| Failure | Observation |
|---|---|
| Flag conflict, malformed `--wallet-id` UUID, malformed `--serial`, or empty string value | clap usage error: exit code 2; JSON `code` = `usage_error` (tvc/src/cli.rs:154-182; tvc/tests/operator_create.rs:87-153). |
| `--serial` with the `hosted` operator kind | Message `--serial is only valid with --kind yubikey`: exit code 1, `code` = `command_error` (create.rs:129-132). |
| `--wallet-name`, `--wallet-id`, or `--account-path` with the `yubikey` operator kind | Exit code 1, `code` = `command_error` (create.rs:152-155; unit test create.rs:602-615). |
| `--serial` absent in non-interactive mode | `reason` = `missing_required_input`, `code` = `missing_required_input`, exit code 1 (create.rs:285-287; tvc/src/output.rs:326-330). |
| No active org (either arm) | Message starts `No active organization. Run 'tvc login' first.`: exit code 1, `code` = `command_error` (create.rs:189-192, 289-291). |
| Env auth org differs from the active org | Exit code 1, `code` = `command_error` (tvc/src/operator.rs:149-158). |
| Hosted activity failure | The typed client error survives the chain; `classify` maps it to `unauthorized`, `not_found`, `api_error`, `approval_required`, or `network_error` (tvc/src/operator/hosted.rs:355-364; tvc/src/errors.rs:93-103). |
| Malformed creation result: non-UUID ids, bad or identical public keys | Exit code 1, `code` = `command_error` (tvc/src/operator/hosted.rs:89-99, 122-158). |
| `--serial` outside the device registry (either mode) | Exit code 1, `code` = `command_error`, remediation `tvc keys refresh-yubikey --serial <serial>` (create.rs:296-303, 379-386; tvc/tests/operator_create.rs:184-202). |
| Serial already an operator of the org | Message `YubiKey <serial> is already an operator of this organization`: exit code 1 (tvc/src/config/turnkey.rs:417, 530-549). |
| Config save failure after remote creation (hosted) | Exit code 1 with recovery TOML; the remote operator exists with no local operator record (create.rs:239-263; tvc/tests/operator_create.rs:276-307). |
| Config save failure (yubikey) | Exit code 1 with recovery TOML and a safe-retry note (create.rs:353-359; tvc/tests/operator_create.rs:309-332). |

## Test vectors (normative)

Vector comparison excludes the nondeterministic fields that Part 00 lists.
Hosted vectors run against the stub server fixture (tvc/tests/operator_create.rs:11-60).
The fixture returns operator id `11111111-1111-4111-8111-111111111111` and wallet id `22222222-2222-4222-8222-222222222222`.
It generates fresh public keys per run, so comparison excludes the key hex values.
The yubikey vectors start from the yubikey base config (tvc/tests/operator_create.rs:204-232).
It holds active org `default` (id `org-123`), one local operator record, and registered YubiKey `01c95c1f`.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Active org `default` (id `org-123`) with valid credentials; the stub API completes the activity. | `tvc operator create` | stdout `Hosted operator created!` block with `Operator name: tvc-operator` and `Saved: true`; exit code 0 (create.rs:265-276, 426-446). |
| V-2 | Same as V-1. | `tvc operator create --message-format json` | One NDJSON object: `reason` = `operator_created`, `name` = `tvc-operator`, `operatorId` = `11111111-1111-4111-8111-111111111111`, `walletId` = `22222222-2222-4222-8222-222222222222`, `saved` = true; exit code 0 (create.rs:413-424; tvc/src/outcome.rs:30-35). |
| V-3 | The yubikey base config; `TVC_NON_INTERACTIVE=1`. | `tvc operator create --kind yubikey --serial 01c95c1f --default` | stdout `YubiKey operator added!`, name `yubikey-01c95c1f`, `It is now the organization's default operator.`; the saved tvc config gains the operator record and `default_operator_kind = "yubikey"`; exit code 0 (tvc/tests/operator_create.rs:237-263). |
| V-4 | The yubikey base config. | `tvc operator create --kind yubikey --serial 01c95c1f --message-format json` | One NDJSON object: `reason` = `yubikey_operator_added`, `name` = `yubikey-01c95c1f`, `serial` = `01c95c1f`, `madeDefault` = false; exit code 0 (create.rs:448-459). |
| V-5 | Any tvc config. | `tvc operator create --wallet-name w --wallet-id 22222222-2222-4222-8222-222222222222 --message-format json` | One JSON object with `code` = `usage_error`; exit code 2 (tvc/tests/operator_create.rs:87-99; tvc/src/cli.rs:154-182). |
| V-6 | Any tvc config. | `tvc operator create --serial 01c95c1f` | stderr `--serial is only valid with --kind yubikey`; exit code 1; JSON `code` = `command_error` (create.rs:129-132; tvc/tests/operator_create.rs:155-167). |
| V-7 | Any tvc config. | `tvc operator create --kind yubikey --wallet-name w` | stderr `--wallet-name, --wallet-id, and --account-path are only valid with --kind hosted`; exit code 1 (create.rs:152-155; unit test create.rs:602-615). |
| V-8 | Default tvc config; `TVC_NON_INTERACTIVE=1`. | `tvc operator create --kind yubikey` | stderr `--serial is required in non-interactive mode`; exit code 1; JSON `reason` = `code` = `missing_required_input` (create.rs:285-287; tvc/tests/operator_create.rs:169-182). |
| V-9 | Default tvc config with no orgs. | `tvc operator create --wallet-id 11111111-1111-4111-8111-111111111111` | stderr contains `No active organization`; exit code 1 (create.rs:189-192; tvc/tests/operator_create.rs:115-129). |
| V-10 | The yubikey base config; `TVC_NON_INTERACTIVE=1`. | `tvc operator create --kind yubikey --serial deadbeef` | stderr contains `is not in the device registry` and `tvc keys refresh-yubikey --serial deadbeef`; exit code 1 (create.rs:296-303; tvc/tests/operator_create.rs:184-202). |
| V-11 | The yubikey base config where org `default` already references `01c95c1f`. | `tvc operator create --kind yubikey --serial 01c95c1f` | stderr `YubiKey 01c95c1f is already an operator of this organization`; exit code 1; tvc config unchanged (tvc/src/config/turnkey.rs:530-549; tvc/tests/operator_create.rs:264-274). |
| V-12 | The yubikey base config with the tvc config file read-only; `TVC_NON_INTERACTIVE=1`. | `tvc operator create --kind yubikey --serial 01c95c1f --default` | stderr contains `the YubiKey operator could not be saved`, a `[[orgs."default".operators]]` recovery fragment, and the `default_operator_kind` note; exit code 1 (create.rs:353-359; tvc/tests/operator_create.rs:309-332). |

## Invariants (normative)

The global invariants INV-G1 through INV-G4 apply.

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The yubikey arm MUST NOT perform device I/O. | `YubikeyCreate::execute` reads only the registry's cached key (create.rs:379-387). Behavioral checks: create.rs:653-674 (FakeDevice with both slots empty) and tvc/tests/operator_create.rs:237-274 (no device attached). |
| INV-2 | One invocation MUST call `Config::save` at most once, carrying the operator record and any default operator kind change together. | Each arm mutates the in-memory config, then saves once (create.rs:228-239, 333-353). |
| INV-3 | An org MUST NOT hold two operator records for one YubiKey serial. | `OrgConfig::new_yubikey_operator` refuses a duplicate before any mutation (tvc/src/config/turnkey.rs:530-549). Behavioral check: tvc/tests/operator_create.rs:264-274. |
| INV-4 | On save failure the error MUST carry a recovery TOML fragment that parses back to the exact operator record. | `recovery_toml` (create.rs:504-514); round-trip test create.rs:540-555; end-to-end checks tvc/tests/operator_create.rs:276-332. |
| INV-5 | Kind-incompatible flag combinations MUST fail before credential use or network I/O. | `CreatePlan::try_from` runs before any config read or client build (create.rs:187); unit tests create.rs:587-615. |

## Gaps (informative)

1. **[capability] The command cannot create a local operator; only the login new-org flow can**.
   `CreateKind` offers `hosted` and `yubikey` only (create.rs:32-37).
   Local is a first-class consumable operator kind everywhere else.
   `deploy approve` accepts registered local operators and seeds (tvc/src/commands/deploy/approve.rs:52-57, 99-116).
   `keys re-encrypt-local-share` and `keys backup-operator-key` need a local operator record (tvc/src/operator.rs:411-419, tvc/src/commands/keys/backup_operator_key.rs:64).
   `OperatorKind::Local` is the config default (tvc/src/config/turnkey.rs:273-280).
   `OperatorRecord::local` is reachable only through `Config::add_org` during `tvc login` org creation (tvc/src/config/turnkey.rs:303-310, 634-651; tvc/src/commands/login.rs:61-74, 678-710).
   An existing org can never gain a local operator record through the CLI.
   That includes an org that started with a YubiKey default.
   The only path is hand-editing the tvc config.

2. **[capability] `--default` at create time is the only CLI write to the default operator kind, and only away from local**.
   The non-login write sites for the field are create.rs:231 and create.rs:400.
   The `operator` group has exactly one subcommand (tvc/src/cli.rs:380-383); no `operator set-default`, `operator list`, or `operator remove` exists.
   Once the field moves to hosted or yubikey, restoring a local default requires hand-editing the tvc config.
   That state-over-choice shape leaves `keys re-encrypt-local-share` unable to use a configured local operator.
   That command dispatches on the default operator kind only (tvc/src/operator.rs:411-441).

3. **[consistency] Non-interactive YubiKey creation hard-requires `--serial` even when the registry holds exactly one YubiKey**.
   The requirement fires before the registry read (create.rs:285-287), so the sole-key auto-select at create.rs:311 is unreachable in JSON mode and CI.
   An interactive run uses the same sole key with no prompt.
   `tvc login` accepts an omitted serial in non-interactive mode when the org has a sole YubiKey operator record (tvc/src/commands/login.rs:91-93, 403-414, 550-561).

4. **[consistency] Only the yubikey arm offers the make-default choice interactively**.
   YubiKey creation confirms `Make this the default operator for the organization?` when it can prompt (create.rs:320-325).
   The hosted arm never asks (create.rs:188-277 contains no prompt); hosted users have to know about `--default` up front.

5. **[consistency][bug?] Nothing guards against a second hosted operator record, which quietly breaks every sole-hosted consumer**.
   The yubikey arm refuses a duplicate serial (tvc/src/config/turnkey.rs:530-540).
   The hosted arm appends unconditionally, duplicate default name `tvc-operator` included (create.rs:141-149, 228).
   With two hosted operator records, `select_hosted_operator` errors with `MultipleHostedOperators` (tvc/src/config/turnkey.rs:473-483).
   That fails `tvc login` for a hosted-default org (tvc/src/commands/login.rs:530-533) and blanks `default_operator_public_key` (tvc/src/operator.rs:291-296).
   Create neither warns nor offers a later way to pick among several hosted operator records; no per-operator default exists.

6. **[docs] The one-operator-per-wallet path constraint lives only in a Rust doc comment**.
   The `DEFAULT_HOSTED_OPERATOR_BASE_PATH` docs tell callers to supply a different base path for a second operator in the same wallet (tvc/src/operator/hosted.rs:21-29).
   The `--wallet-id` help (create.rs:70-73) and the `--account-path` help (create.rs:75-77) never say so.
   The command does not vary the path and does not pre-flight the collision; the failure surfaces as an opaque server error.

7. **[consistency] Kind-compat mistakes exit 1 as `command_error`, while near-identical flag conflicts exit 2 as `usage_error`, and `ErrorCode::InvalidInput` is dead code**.
   `--serial --wallet-name` is a clap conflict (exit code 2, `usage_error`), while `--kind hosted --serial` is a runtime `ensure!` (exit code 1, `command_error`) (create.rs:86-91 versus 129-132).
   The taxonomy's `invalid_input` code appears in the CLI long help (tvc/src/cli.rs:56).
   It carries `#[allow(dead_code)]` and no code path assigns it (tvc/src/errors.rs:54-56).
   So in JSON mode semantic validation looks the same as an arbitrary failure.
   Values from environment variables trip the same checks with no flag typed, for example an exported `TVC_OPERATOR_WALLET_NAME` with `--kind yubikey` (create.rs:62-68, 152-155).

8. **[docs] The `saved` field in `operator_created` is vestigial: it always carries `true`**.
   The code sets the field literally once (create.rs:275) and the human rendering hard-codes `Saved: true` (create.rs:437).
   A failed save aborts with an error before the command emits the outcome (create.rs:239-263).
   The field can never be false, and it implies a partial-success mode that does not exist.
