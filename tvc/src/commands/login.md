# tvc login

*Depends on: Part 00 (00_preliminaries.md). Conformance unit: the login command.*

## Purpose (informative)

`tvc login` authenticates with Turnkey and prepares local credentials for one org.
It selects an org (or creates one interactively), finds or generates the API key, and verifies it with `get_whoami`.
It then resolves the operator record for the org's default operator kind, sets the org as the active org, and saves the tvc config.
Run it once per machine per org, again to switch the active org, and again after a credential restore or rotation.
Entry point: `run` (tvc/src/commands/login.rs:97), dispatched from tvc/src/cli.rs:302.
The same file hosts `profile delete` (`DeleteArgs`, `run_delete`), which has its own specification.
Citations below give bare line numbers for login.rs and full paths for every other file.

## Acceptance scenario (normative)

The scenario universe: org alias `prod`, org id `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, a stub Turnkey API at `http://127.0.0.1:8081`.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Write a tvc config with one org `prod`: id `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, `api_base_url = "http://127.0.0.1:8081"`, `default_operator_kind = "local"`, one local operator record at the default path, no active org. | `~/.config/turnkey/tvc.config.toml` exists and parses. |
| 2 | Write a valid API key file at `~/.config/turnkey/orgs/prod/api_key.json` and a valid operator key file at `~/.config/turnkey/orgs/prod/operator.json`. | Both files parse as their stored key schemas. |
| 3 | Serve `get_whoami` at the stub URL with `organization_name` = `Acme Corp`, `organization_id` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, `username` = `alice`, `user_id` = `9e8d7c6b-5a4f-3e2d-1c0b-a9f8e7d6c5b4`. | The stub answers 200. |
| 4 | Run `tvc login --org prod`. | stdout carries `Selected org: prod (6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b)` (login.rs:468), `Using existing API key.` (login.rs:491), `Verifying credentials...` (login.rs:509), `Using existing operator key.` (login.rs:841), then the `Successfully logged in!` block with `Active Org: prod` (login.rs:1079-1114). Exit code 0. |
| 5 | Read the tvc config. | `active_org = "prod"` (login.rs:470-472). |
| 6 | Run `tvc login --org prod --message-format json`. | stdout carries exactly one NDJSON object with `reason` = `logged_in` and `operatorKind` = `local` (tvc/src/outcome.rs:30-33). Exit code 0. |

Pass criterion: all steps pass in one run that starts from an empty `~/.config/turnkey/` directory.

## Inputs (normative)

| Input | Flag | Env | Config source | Default | Prompt |
|---|---|---|---|---|---|
| org (alias or id) | `--org` | `TVC_ORG` | none: the active org is never consulted (login.rs:404-406, 601-640) | none | picker over configured orgs plus `[new]` (login.rs:619-640) |
| API base URL | `--api-base-url` | `TVC_API_BASE_URL` | the org's stored `api_base_url` (existing orgs) | `https://api.turnkey.com` for new orgs (login.rs:764-768) | never |
| YubiKey serial | `--serial` | none | none | the sole YubiKey operator record | picker when the org stores several (login.rs:370-387); interactive mode only |
| new-org org id | none | none | none | none | prompt only (login.rs:655-658) |
| new-org alias | none | none | none | `default` | prompt only (login.rs:660) |
| new-org operator kind | none | none | none | none | prompt only: `Local key file` or `YubiKey`; `hosted` is never offered (login.rs:663-710) |
| operator key backup destination | none | none | none | `operator-<alias>-backup.json` | prompt only, after generating a local key (login.rs:879-923; tvc/src/commands/keys/backup_operator_key.rs:123-128) |
| non-interactive (global) | `--non-interactive` | `TVC_NON_INTERACTIVE` | none | false (tvc/src/cli.rs:70-79) | never |

Deviations from the Part 00 value resolution order:

- For an existing org, the command MUST write a present `--api-base-url` or `TVC_API_BASE_URL` value into the org's stored `api_base_url` (login.rs:426-430, 770-781). It MUST then save the tvc config (login.rs:472). Gap 6 proposes a change.
- When `--org` and `TVC_ORG` are absent, the command MUST NOT fall back to the active org (login.rs:404-406, 601-640). Gap 4 proposes a change.
- The command reads `--serial` only when the org's default operator kind is `yubikey` (login.rs:360-393). A new org with the `YubiKey` choice also reads it (login.rs:684-693). Every other run MUST ignore it (help text, login.rs:41-44).

## Interactive behavior (normative)

Plan building (`build_login_plan_interactive`, login.rs:345-401):

1. When `--org` and `TVC_ORG` are absent, the command MUST prompt `Select organization` (login.rs:636). Each row shows `alias (id)`, the active org carries an ` (active)` suffix, and the last row is `[new] Add a new organization` (login.rs:619-634). With zero configured orgs the command MUST skip the picker and start the new-org prompts (login.rs:613-617).
2. For a new org the command prints the dashboard welcome URL (login.rs:648-652). It MUST then prompt `Organization ID` (an empty value is an error, login.rs:655-658), `Organization alias` (default `default`, login.rs:660), and `Operator key type` (login.rs:678-681).
3. For a new org with the `YubiKey` choice, a present `--serial` MUST exist in the device registry (login.rs:686-691). Without `--serial` the command MUST take the sole registered serial, MUST prompt `YubiKey to use as the operator` when several exist, and MUST fail when none exist (login.rs:694-704).
4. For an existing org whose default operator kind is `yubikey`, a present `--serial` MUST match one of the org's YubiKey operator records (login.rs:364-365). The match runs in `select_yubikey_operator` (tvc/src/config/turnkey.rs:513-515). Without `--serial` the command MUST take the sole record, or MUST prompt `Select YubiKey operator` when the org stores several (login.rs:368-387).

Execution (`execute_login`, login.rs:416-599):

5. When the API key file is absent, the command MUST generate one and print the dashboard registration steps (login.rs:496, 800-819). It MUST then block on `Press Enter when done...` read from raw stdin (login.rs:497, 824-830).
6. When the default operator kind is `local` and the operator key file is absent, the command MUST generate the key (login.rs:846-860). When stdin is a TTY it then offers a backup: a confirm prompt, a destination prompt, a file copy (login.rs:879-904). An escaped prompt or a failed backup degrades to a warning because the tvc config and both key files are already saved (login.rs:906-912).

In non-interactive mode (JSON mode implies it, INV-G1), plan building is `build_login_plan_non_interactive` (login.rs:403-414):

- A missing `--org` MUST fail with `missing_required_input` (login.rs:404-406).
- The plan is always an existing org; no new-org path exists (login.rs:408-413).
- The API key file MUST already exist (`ApiKeyPolicy::RequireExisting`, login.rs:411, 500-505).
- A `yubikey` default operator kind with several YubiKey operator records and no `--serial` MUST fail with `missing_required_input` naming `--serial` (login.rs:553-558).
- A missing local operator key file is still generated, silently (login.rs:523, 846-860). Gap 9 covers this.

## Outputs (normative)

Human mode prints progress lines, then the `LoggedIn` block. The progress lines are:

- The `Selected org:` line (login.rs:468).
- New-org YubiKey guidance: the public key and the register-it instruction (login.rs:474-486).
- `Using existing API key.` (login.rs:491), or `API Key Generated!` plus the dashboard steps (login.rs:800-819).
- `Verifying credentials...` (login.rs:509).
- The `Using hosted operator ...` or `Using YubiKey operator ...` line (login.rs:534-539, 571-576).
- The operator key generation and backup messages (login.rs:841-842, 862-874).

The `LoggedIn` block (login.rs:1079-1147) shows the `get_whoami` identity, the alias, the API public key, an operator kind specific section, and the saved paths.

JSON mode MUST emit exactly one NDJSON outcome with `reason` = `logged_in` (tvc/src/outcome.rs:30-33).

| Field | Value |
|---|---|
| `organizationName`, `organizationId`, `username`, `userId` | From the `get_whoami` response (login.rs:586-590). |
| `alias` | The org alias. |
| `apiPublicKey` | The stored API key's compressed P256 hex public key. |
| `configFilePath`, `apiKeyPath` | The tvc config path and the org's API key path (login.rs:593-596). |
| `operatorKind` | Flattened tag: `local`, `hosted`, or `yubikey` (login.rs:984-1005). |
| `local` variant | plus `operatorPublicKey`, `operatorKeyPath` (test login.rs:1178-1201). |
| `hosted` variant | plus `operatorName`, `operatorId`, `operatorPublicKey` (test login.rs:1205-1230). |
| `yubikey` variant | plus `operatorName`, `serial`, `operatorPublicKey` (test login.rs:1234-1259). |

JSON mode MUST suppress all progress lines: `shell_println!` writes through the human channel only (tvc/src/output.rs:246-254).
Errors follow the Part 00 error taxonomy.
The `local` variant's field set is a compatibility contract (INV-1).

## Side effects (normative)

| Effect | Detail |
|---|---|
| tvc config write | Adds the new org entry (login.rs:463; `Config::add_org` inserts or replaces, tvc/src/config/turnkey.rs:661), persists an `--api-base-url` override onto an existing org (login.rs:426-430), sets the active org (login.rs:470). One save at login.rs:472, before credential verification. |
| API key file | Written to the org's `api_key_path` when generated, owner-only permissions (tvc/src/config/turnkey/api_key.rs:68). |
| Operator key file | Written to the local operator record's `key_path` when missing, owner-only permissions (tvc/src/config/turnkey/qos_operator_key.rs:151). |
| Backup copy | Optional, at a user-chosen path, through `tokio::fs::copy`; TVC-241 tracks tightening its permissions (tvc/src/commands/keys/backup_operator_key.rs:181-185). |
| Network | Exactly one Turnkey API call, `get_whoami` (login.rs:947-954), with a client built from the stored API key (login.rs:942-945). No activity is submitted (INV-2). CI env auth (`TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE`; tvc/src/client.rs:20-23, 48-64) is never consulted. |
| YubiKey device | No device I/O: the command reads only the registry's cached public key (login.rs:562-570), so the device does not need a connection. |

## Failure modes (normative)

| Condition | Result | `code` | Exit code |
|---|---|---|---|
| `--org` absent in non-interactive mode | `MissingRequiredInput` naming `--org` (login.rs:404-406) | `missing_required_input` | 1 |
| `yubikey` default operator kind, several YubiKey operator records, no `--serial`, non-interactive mode | `MissingRequiredInput` naming `--serial` (login.rs:553-558) | `missing_required_input` | 1 |
| Org query matches nothing | bail `Organization '<query>' not found.` (login.rs:421-424); gap 11 | `command_error` | 1 |
| API key file absent, non-interactive mode | bail `API key is required in non-interactive mode for org '<id>'.` (login.rs:500-505) | `command_error` | 1 |
| `--serial` names no YubiKey operator record of the org | `no YubiKey operator has serial <serial>` (tvc/src/config/turnkey.rs:433-434, 513-515) | `command_error` | 1 |
| Serial absent from the device registry (new org, or yubikey login) | bail directing to `tvc keys refresh-yubikey` (login.rs:446-453, 562-570, 686-691) | `command_error` | 1 |
| `get_whoami` fails | classified by `crate::errors::classify` (tvc/src/errors.rs:212-243); config mutations from earlier in the run persist (gap 7) | `unauthorized`, `not_found`, `api_error`, `network_error`, or `client_version_too_old` | 1 |
| Zero or several local operator records, or several hosted operator records | typed selection errors with org context (login.rs:519-533; tvc/src/config/turnkey.rs:441-482) | `command_error` | 1 |
| Prompt escape, cancel, or non-TTY stdin under inquire | prompt error (tvc/src/prompts.rs:51-90) | `command_error` | 1 |
| Malformed `--serial` hex | clap value validation (tvc/src/config/turnkey/yubikey.rs:45-58; test tvc/src/cli.rs:568-573) | `usage_error` | 2 |

## Test vectors (normative)

Vectors reuse the acceptance scenario universe: org `prod`, id `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, stub API at `http://127.0.0.1:8081` answering `get_whoami` as in the acceptance scenario.
Comparison excludes the Part 00 global nondeterministic fields, all generated key material, and org picker row order (gap 12).

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | The acceptance scenario config, both key files, the stub. | `tvc login --org prod` | Human output ends with the `Successfully logged in!` block: `Organization: Acme Corp (6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b)`, `User: alice (9e8d7c6b-5a4f-3e2d-1c0b-a9f8e7d6c5b4)`, `Active Org: prod`, `Operator key: ~/.config/turnkey/orgs/prod/operator.json` path line (login.rs:1079-1114). Exit code 0. |
| V-2 | Same as V-1. | `tvc login --org prod --message-format json` | Exactly one NDJSON object: `reason` = `logged_in`, `alias` = `prod`, `operatorKind` = `local`, `operatorKeyPath` = the expanded `~/.config/turnkey/orgs/prod/operator.json`, `operatorPublicKey` = the operator key file's `public_key` (login.rs:1178-1201; tvc/src/outcome.rs:30-33). Exit code 0. |
| V-3 | Org `prod` with `default_operator_kind = "hosted"` and one hosted operator record `hosted-op` with operator id `4f9a2b1c-8d7e-4a5b-9c0d-2e3f4a5b6c7d`; API key file present; the stub. | `tvc login --org prod --message-format json` | One `logged_in` object: `operatorKind` = `hosted`, `operatorName` = `hosted-op`, `operatorId` = `4f9a2b1c-8d7e-4a5b-9c0d-2e3f4a5b6c7d`, `operatorPublicKey` = the record's encrypt key followed by its sign key (login.rs:530-548, 1205-1230). Exit code 0. |
| V-4 | Org `prod` with `default_operator_kind = "yubikey"`, YubiKey operator records with serials `01c95c1f` and `0badc0de`, device registry entries for both; API key file present; the stub; no device connected. | `tvc login --org prod --serial 01c95c1f --message-format json` | One `logged_in` object: `operatorKind` = `yubikey`, `serial` = `01c95c1f`, `operatorPublicKey` = the registry entry's cached key (login.rs:550-583, 1234-1259). Exit code 0. |
| V-5 | Any tvc config. | `tvc login --message-format json` | One JSON object: `reason` = `missing_required_input`, `code` = `missing_required_input`, `message` names `--org` (login.rs:404-406; tvc/src/output.rs:316-330). Exit code 1. |
| V-6 | The V-4 org, no `--serial`. | `tvc login --org prod --message-format json` | One JSON object: `code` = `missing_required_input`, `message` names `--serial` and lists serials `01c95c1f, 0badc0de` (login.rs:553-558; tvc/src/config/turnkey.rs:428-432). Exit code 1. |
| V-7 | The acceptance scenario config. | `tvc login --org staging --message-format json` | One JSON object: `code` = `command_error`, `message` starts `Organization 'staging' not found.` (login.rs:421-424). Exit code 1. |
| V-8 | The acceptance scenario config without `orgs/prod/api_key.json`. | `tvc login --org prod --message-format json` | One JSON object: `code` = `command_error`, `message` starts `API key is required in non-interactive mode for org '6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b'` (login.rs:500-505). Exit code 1. |
| V-9 | Org `prod` with `default_operator_kind = "yubikey"`, one YubiKey operator record with serial `01c95c1f`, an empty device registry; API key file present; the stub. | `tvc login --org prod --message-format json` | One JSON object: `code` = `command_error`, `message` names `tvc keys refresh-yubikey --serial 01c95c1f` (login.rs:562-570). Exit code 1. |
| V-10 | Same as V-1, with the stub answering 401 and `active_org` unset in the tvc config. | `tvc login --org prod --message-format json` | One JSON object: `code` = `unauthorized`, `httpStatus` = 401 (tvc/src/errors.rs:212-221). Exit code 1. The tvc config now has `active_org = "prod"` (login.rs:470-472; gap 7). |
| V-11 | Any tvc config. | `tvc login --org prod --serial not-hex --message-format json` | One JSON object with `code` = `usage_error` (tvc/src/config/turnkey/yubikey.rs:45-58; test tvc/src/cli.rs:568-573; Part 00 GV-1 mechanism). Exit code 2. |
| V-12 | Same as V-1, without `orgs/prod/operator.json`. | `tvc login --org prod --message-format json` | Exit code 0; one `logged_in` object; `orgs/prod/operator.json` now exists owner-only (login.rs:523, 846-860); stdout carries no registration guidance lines (tvc/src/output.rs:246-254; gap 9). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The JSON `logged_in` payload for the `local` operator kind MUST keep its pre-hosted field set plus the additive `operatorKind` tag. | Serde derives on `LoggedIn` and `LoggedInOperator` (login.rs:966-1005); the exact shapes are pinned by tests (login.rs:1178-1259). |
| INV-2 | A login run MUST make exactly one Turnkey API call, `get_whoami`, and MUST NOT submit an activity. | `verify_credentials` is the sole client call site (login.rs:935-954) and `GetWhoamiRequest` is the only request type the module imports (login.rs:27). |
| INV-3 | Generated API key and operator key files MUST be owner-only. | Both saves route through `write_owner_only_file` (tvc/src/util.rs:38; tvc/src/config/turnkey/api_key.rs:68; tvc/src/config/turnkey/qos_operator_key.rs:151). |
| INV-4 | Login MUST resolve an operator record of the org's default operator kind and of no other kind. | The match on `default_operator_kind` dispatches to per-kind registry queries only (login.rs:518-584). |
| INV-5 | In non-interactive mode login MUST NOT create an org entry and MUST NOT generate an API key. | `build_login_plan_non_interactive` constructs only `OrgPlan::Existing` with `ApiKeyPolicy::RequireExisting` (login.rs:403-414); the policy match bails (login.rs:500-505). |

## Gaps (informative)

1. **[capability] Non-interactive and JSON modes cannot bootstrap a profile**. `build_login_plan_non_interactive` only ever produces `OrgPlan::Existing` (login.rs:403-414). Org id, alias, and operator kind are prompt only (login.rs:642-717), with no flag or env equivalents. `ApiKeyPolicy::RequireExisting` (login.rs:500-505) has no `--api-key` or key path escape hatch. The CLI already defines env credentials (tvc/src/client.rs:20-23), and login ignores them. Headless setup requires a hand-written tvc config and key files.

2. **[capability] Login always validates the org's default operator kind; the user cannot pick an operator kind or a named operator record**. The hard match at login.rs:518 is deliberate per the comment at login.rs:512-517, and no `--operator-kind` or operator-name input exists. The only supported change path is `operator create --default`, which always creates a new operator record (tvc/src/commands/operator/create.rs:93-95, 230-232, 399-401). Hand-editing is the alternative. Nothing can flip a default operator kind back to `local`. An org that holds local and yubikey records with a `yubikey` default can therefore never have login find or generate its local key. This is the same default-state-constrains-explicit-choice shape as the `re-encrypt-local-share` canonical example.

3. **[capability] An org with several hosted operator records cannot log in at all**. `select_hosted_operator` accepts only a sole record (tvc/src/config/turnkey.rs:473-482), and login offers no disambiguator in either mode (login.rs:530-533). The yubikey path, by contrast, gets both `--serial` (login.rs:41-44) and an interactive picker (login.rs:370-387). `operator create` appends a second hosted operator record without complaint (tvc/src/commands/operator/create.rs:228), so supported commands reach the state.

4. **[consistency] Non-interactive login does not default to the active org**. `--org` is required (login.rs:404-406) even when the active org is set. Sibling `keys backup-operator-key --org` defaults to the active org (tvc/src/commands/keys/backup_operator_key.rs:26-29, 58-62). A CI re-verify of the current profile has to restate the org.

5. **[consistency] A set `TVC_ORG` env var permanently suppresses the org picker and the whole new-org flow**. Clap env feeds `args.org` (login.rs:36-37), so `prompt_for_org_plan` is unreachable while the shell exports `TVC_ORG`. The export is plausible: `backup-operator-key` shares the var (tvc/src/commands/keys/backup_operator_key.rs:28). When the value matches nothing, the error says `Run tvc login without --org` (login.rs:421-424). That advice fails identically, because the env var is the source and dropping the flag changes nothing.

6. **[bug?] `--api-base-url` and `TVC_API_BASE_URL` silently and persistently rewrite an existing org's stored base URL, before credential verification**. `update_api_base_url_from_override` mutates the org (login.rs:426-430, 770-781), and the run saves the tvc config (login.rs:472) ahead of `get_whoami` (login.rs:511). A failed login still leaves the URL switched. `TVC_API_BASE_URL` doubles as the optional CI env-auth base URL (tvc/src/client.rs:21, 196-198). An exported CI environment therefore leaks into a persistent tvc config mutation. [docs] The flag's help text (`Defaults to production for newly configured orgs`, login.rs:38) never mentions the rewrite.

7. **[bug?] A failed login still switches, and persists, the active org**. `set_active_org` and `config.save()` run at login.rs:470-472, before the API key check, `get_whoami`, and operator resolution. A login that fails verification (revoked key, wrong environment, missing operator record) leaves the active org pointing at the failed profile. That silently redirects every later command's credentials (tvc/src/client.rs:102-117). New org entries likewise persist on failure. That part is retry friendly, yet combined with the active org flip it changes global behavior on error.

8. **[bug?] A new-org alias collision silently clobbers an existing profile**. `prompt_for_new_org_inputs` never checks the alias against `config.orgs` (login.rs:660), and `Config::add_org` unconditionally inserts or replaces (tvc/src/config/turnkey.rs:661). Entering an existing alias (the prompt default is literally `default`) replaces that profile's org id, default operator kind, and operator records. The replaced hosted operator identities are recoverable from nowhere else.

9. **[consistency] Non-interactive mode gates API key generation and never gates operator key generation**. `ApiKeyPolicy::RequireExisting` blocks API key creation in CI (login.rs:494-505). `find_or_generate_operator_key` fabricates a new local operator key whenever the file is missing, in any mode (login.rs:523, 846-860). JSON mode suppresses the register-this-key guidance because the shell macros are human only (tvc/src/output.rs:246-254). A partially restored machine therefore silently gains an unregistered key that later fails approvals; only the changed `operatorPublicKey` in the JSON payload hints at it.

10. **[bug?] `wait_for_dashboard_registration` reads raw stdin with no TTY guard**. login.rs:824-830 bypasses the ctx and prompt layer. With piped or closed stdin in interactive mode (reachable without any inquire prompt when the run supplies `--org`), `read_line` returns immediately on EOF. The flow then generates a key, skips the registration wait, and fails `get_whoami` confusingly. The backup nudge a few lines later explicitly guards with `prompts::stdin_can_prompt()` (login.rs:879).

11. **[consistency] Lookup failures classify as `command_error` where the taxonomy assigns `not_found`**. `Organization '<query>' not found` is a plain string bail (login.rs:421-424), so JSON consumers get `code` = `command_error`. The documented taxonomy assigns `not_found` to a resource that resolved to empty (tvc/src/cli.rs:58). The API-key-required bail (login.rs:500-505) is conceptually missing input. It cannot be `missing_required_input` because no flag exists to satisfy it (ties to gap 1).

12. **[bug?] Minor input hygiene holes in the new-org prompts and the org picker**. The alias flows unvalidated into filesystem paths: `default_org_dir` joins it verbatim (tvc/src/config/turnkey.rs:573-575). Values like `../x` or `a/b` therefore shape directories outside `orgs/`. The org id check only catches the truly empty string (login.rs:655-658) and accepts whitespace. Both the org picker and the id fallback in `find_org` iterate a `HashMap` (login.rs:619-634, 738-744). Picker order therefore changes per run, and a duplicated org id resolves to an arbitrary alias.
