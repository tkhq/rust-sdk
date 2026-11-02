# tvc profile delete

*Depends on: Part 00 (00_preliminaries.md). Conformance unit: the profile delete command.*

## Purpose (informative)

`tvc profile delete` removes a profile: the org entry in the tvc config plus its API key file and any local operator key files.
The operation is local.
The command makes no Turnkey API call, leaves YubiKey devices and the shared `[[yubikeys]]` registry alone, and keeps the dashboard registered API key valid.
After deletion it prints instructions to revoke that key on the dashboard.
Run it to remove one machine's credentials for an org that you no longer use.
Implementation: `run_delete` (commands/login.rs:118), dispatched at cli.rs:306-310.
Unqualified citations in this specification name files under `tvc/src`.

## Acceptance scenario (normative)

`<home>` stands for the user's home directory.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Write a tvc config with one profile: alias `staging`, org id `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, default key paths, active org `staging`. Create `api_key.json` and `operator.json` under `<home>/.config/turnkey/orgs/staging/`. | Setup complete. |
| 2 | Run `tvc profile delete --org staging --yes`. | Exit code 0. |
| 3 | Inspect stdout. | Line 1: `Deleted login profile 'staging' (6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b).` Line 2: `Removed key directory: <home>/.config/turnkey/orgs/staging`. Then the `IMPORTANT:` dashboard revocation block. |
| 4 | Inspect the file system. | `<home>/.config/turnkey/orgs/staging/` is absent. |
| 5 | Read the tvc config. | The `staging` org is absent. Its `last_created_app_id` and `last_operator_ids` entries are absent. The active org is unset. |

All steps pass in one run from a clean start (pinned by tvc/tests/login.rs:75-103).

## Inputs (normative)

| Input | Flag | Env | Config key | Default | Interactive fallback |
|---|---|---|---|---|---|
| profile (alias or org id) | `-o, --org <ORG>` | none | none | none | picker over configured profiles (commands/login.rs:316-327) |
| confirmation | `-y, --yes` | none | none | false | y/N confirm, default No (commands/login.rs:205-208) |
| non-interactive (global) | `--non-interactive` | `TVC_NON_INTERACTIVE` | none | false | none; makes `--org` and `--yes` hard requirements (commands/login.rs:129-136) |
| message format (global) | `--message-format` | none | none | `human` | none; `json` implies non-interactive mode (INV-G1) |

The command MUST resolve `--org` as a profile alias first, then as an org id (`find_org`, commands/login.rs:733-745).
`--org` and `--yes` have no environment variable and no config key (commands/login.rs:51-59).
Gap 3 records the deviation from the Part 00 value resolution order.
The command reads no command config.

## Interactive behavior (normative)

In interactive mode the command runs these steps in order.

1. When `--org` is absent, the command MUST show the `Select profile to delete` picker (commands/login.rs:316-327). Each row renders `alias (org-id)`, with an ` (active)` suffix on the active org's row (commands/login.rs:338-343). When the tvc config holds zero orgs, the command MUST fail with `No login profiles to delete.` (commands/login.rs:313-315).
2. When `--yes` is absent, the command MUST print a warning block on stderr (commands/login.rs:147-204). The block states that deletion removes the org entry and the key files, and cannot be undone. It states that the dashboard registered API key stays valid until removed there. When the profile references YubiKey operator records, the block adds that the devices and their `[[yubikeys]]` registry entries stay. It then points at `tvc yubikey unregister` (commands/login.rs:176-202).
3. The command MUST then ask `Permanently delete profile '<alias>' (<org-id>) and its key files?` with default No (commands/login.rs:205-208). A decline MUST fail the run with `operation cancelled by user: deletion` (prompts.rs:70-75).

`--yes` MUST skip both the warning block and the confirmation prompt (commands/login.rs:147).

In non-interactive mode the command MUST validate `--org` and `--yes` before all other work (commands/login.rs:129-136).
Each absent flag MUST produce the `missing_required_input` error that names the flag (prompts.rs:21-23; output.rs:283-293).
No prompt runs in non-interactive mode.

In interactive mode with piped stdin, the prompts still run and fail inside `inquire`, because every prompt needs a TTY (prompts.rs:2-5).
Gap 4 proposes a TTY guard.

## Outputs (normative)

In human mode the command MUST write warnings and progress to stderr, and the outcome to stdout (`Display`, commands/login.rs:1032-1077).
The outcome contains, in order:

- `Deleted login profile '<alias>' (<org-id>).`, always.
- `Removed key directory: <dir>`, only when the command deleted the default org directory.
- A retention line that names the kept YubiKey serials and `tvc yubikey unregister`, only when the profile referenced YubiKey operator records (commands/login.rs:1043-1055).
- An `IMPORTANT:` block with dashboard revocation steps (commands/login.rs:1057-1073). Step 2 MUST name the exact API public key when the key file was readable before deletion. Otherwise step 2 names `the API key associated with this profile`.

In JSON mode the command MUST emit exactly one NDJSON object with `reason` = `profile_deleted` (outcome.rs:29-36; commands/login.rs:1018-1030).

| Field | Value |
|---|---|
| `alias` | The deleted profile's alias. |
| `organizationId` | The deleted org's id. |
| `removedKeyDirectory` | The deleted directory path, or null when the command deleted nothing. |
| `retainedYubikeySerials` | Serials of the profile's YubiKey operator records; omitted when empty (commands/login.rs:1026-1027). |
| `dashboardUrl` | The dashboard base URL that maps from the org's API base URL (config/turnkey.rs:263-270). |
| `apiPublicKey` | The API key's public key, or null when its file was missing or unreadable. |

The acceptance scenario run with `--message-format json` emits:

```json
{"reason":"profile_deleted","alias":"staging","organizationId":"6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b","removedKeyDirectory":"<home>/.config/turnkey/orgs/staging","dashboardUrl":"https://app.turnkey.com","apiPublicKey":"02a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90"}
```

Every human presentation line is silent in JSON mode: each `shell_eprintln!` warning disappears (output.rs:154-159).

## Side effects (normative)

1. The dispatch layer loads the tvc config before the command runs and creates the file when absent (cli.rs:215-240; INV-G4). On a fresh machine the run therefore creates the tvc config and then fails with profile not found.
2. The command reads the profile's API key file before deletion, to capture the public key for the revocation reminder (commands/login.rs:215-222). The read is best-effort: a missing or unreadable file leaves `apiPublicKey` null.
3. The command MUST delete `<home>/.config/turnkey/orgs/<alias>/` recursively when every configured key path is a default path (commands/login.rs:228-259). The default paths live in config/turnkey.rs:573-585. The check compares `api_key_path` with the default API key path, and each local operator `key_path` with the default operator key path (commands/login.rs:238-241).
4. With any custom key path the command MUST NOT delete files. It MUST list the custom paths plus the API key path in a stderr warning (commands/login.rs:260-274).
5. When the default directory is absent from disk, the command MUST warn `key directory was not on disk` and continue (commands/login.rs:247-254).
6. The command MUST save the tvc config last, after the on-disk cleanup succeeds, so a failed delete stays retryable (commands/login.rs:276-278; INV-2).
7. `remove_org` MUST drop the profile's `last_created_app_id` and `last_operator_ids` entries (config/turnkey.rs:675-676). It MUST clear the active org when the deleted alias was active (config/turnkey.rs:677-679).
8. The command makes no Turnkey API call and no device interaction (INV-1). The shared `[[yubikeys]]` registry entries stay (INV-3; tvc/tests/login.rs:109-149).

## Failure modes (normative)

Each failure MUST exit through the Part 00 exit code and error taxonomy (INV-G2, INV-G3).

| Failure | Observation | Mechanism |
|---|---|---|
| `--org` absent in non-interactive mode | `reason` `missing_required_input`, `code` `missing_required_input`, exit code 1 | commands/login.rs:130-132; output.rs:326-333 |
| `--yes` absent in non-interactive mode | Same shape; the message names `--yes` | commands/login.rs:133-135 |
| `--org` matches no profile | Message `Login profile '<query>' not found.` plus a `tvc login` hint; `code` `command_error`; exit code 1 | commands/login.rs:305-308; errors.rs:93-102 |
| Zero profiles (interactive, no `--org`) | `No login profiles to delete.`; `code` `command_error`; exit code 1 | commands/login.rs:313-315 |
| User declines the confirmation | `operation cancelled by user: deletion`; `code` `command_error`; exit code 1 | prompts.rs:70-75 |
| `remove_dir_all` fails (except NotFound) | `failed to delete key directory: <dir>`; `code` `command_error`; exit code 1; the tvc config still lists the profile, so the delete stays retryable | commands/login.rs:255-258, 276-278 |
| tvc config save fails | `code` `command_error`; exit code 1; the key files are already gone while the profile stays listed; a retry warns `key directory was not on disk` and repeats the save | commands/login.rs:278, 247-254 |
| Bad flags or arguments | Usage error, exit code 2; in JSON mode one JSON object with `code` `usage_error` | cli.rs:154-182 |

## Test vectors (normative)

The vectors share one fixture, `staging`: alias `staging`, org id `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, API base URL `https://api.turnkey.com`.
It has one local operator record, default key paths, key files on disk, and active org `staging`.
Each vector starts from a clean home directory that holds only its Given column.
`<home>` stands for the user's home directory.
Part 00 lists the globally excluded nondeterministic fields; no field of this command's outcome is nondeterministic.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Fixture `staging` | `tvc profile delete --org staging --yes` | stdout: `Deleted login profile 'staging' (6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b).` then `Removed key directory: <home>/.config/turnkey/orgs/staging`; the org directory and the org entry are gone; exit code 0 (tvc/tests/login.rs:75-103). |
| V-2 | Fixture `staging` | `tvc profile delete --org staging --yes --message-format json` | stdout: exactly the NDJSON object in Outputs; no stderr warnings; exit code 0 (commands/login.rs:288-295; outcome.rs:29-36). |
| V-3 | Fixture `staging` | `tvc profile delete --yes --non-interactive --message-format json` | One JSON object: `reason` `missing_required_input`, `code` `missing_required_input`, message contains `--org is required in non-interactive mode`; exit code 1 (commands/login.rs:130-132; output.rs:283-293, 326-333). |
| V-4 | Fixture `staging` | `tvc profile delete --org staging --non-interactive --message-format json` | Same shape as V-3 with `--yes` in the message; exit code 1 (commands/login.rs:133-135). |
| V-5 | Fixture `staging` | `tvc profile delete --org ghost --yes --message-format json` | One JSON object: `code` `command_error`, message contains `Login profile 'ghost' not found`; exit code 1 (commands/login.rs:305-308; errors.rs:93-102). |
| V-6 | tvc config with zero orgs; TTY stdin | `tvc profile delete` | stderr error `No login profiles to delete.`; exit code 1 (commands/login.rs:313-315). |
| V-7 | Fixture `staging`; TTY stdin; answer `n` at the confirm | `tvc profile delete --org staging` | stderr: the warning block, then error `operation cancelled by user: deletion`; the tvc config is unchanged; exit code 1 (commands/login.rs:205-208; prompts.rs:70-75). |
| V-8 | Profile `test`, org id `org-test`, sole operator record YubiKey serial `01c95c1f`, matching `[[yubikeys]]` registry entry; `TVC_NON_INTERACTIVE=1` | `tvc profile delete --org test --yes` | stdout contains `Kept the YubiKey registry entries (serials 01c95c1f)` and `tvc yubikey unregister`; the saved tvc config keeps `[[yubikeys]]` with `serial = "01c95c1f"` and drops `org-test`; exit code 0 (tvc/tests/login.rs:109-128). |
| V-9 | Profiles `one` (org id `org-1`) and `two` (org id `org-2`), both referencing registry serial `01c95c1f`; `TVC_NON_INTERACTIVE=1` | `tvc profile delete --org one --yes` | The saved tvc config drops `org-1`, keeps `org-2`, and keeps `[[yubikeys]]` with `serial = "01c95c1f"`; exit code 0 (tvc/tests/login.rs:133-149). |
| V-10 | Fixture `staging`, except the operator record's `key_path` is `<home>/keys/staging-operator.json` | `tvc profile delete --org staging --yes` | stderr: `WARNING: custom key paths are configured and were NOT deleted.` followed by `<home>/keys/staging-operator.json` and the API key path; no file removed; stdout has no `Removed key directory:` line; exit code 0 (commands/login.rs:260-274). |
| V-11 | Fixture `staging` in the tvc config while `<home>/.config/turnkey/orgs/staging/` is absent | `tvc profile delete --org staging --yes` | stderr: `WARNING: key directory was not on disk: <home>/.config/turnkey/orgs/staging`; stdout: the `Deleted login profile` line with no `Removed key directory:` line; the org entry is gone; exit code 0 (commands/login.rs:247-254). |
| V-12 | Any tvc config | `tvc profile delete --frobnicate --message-format json` | stdout: one JSON object with `code` `usage_error`; exit code 2 (cli.rs:154-182; GV-1). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT call the Turnkey API or touch a YubiKey device. | `run_delete` (commands/login.rs:118-296) builds no client and performs only tvc config and file system work. Behavioral check: tvc/tests/login.rs:75-149 runs with no server. |
| INV-2 | The command MUST save the tvc config only after the on-disk cleanup succeeds. | Statement order in `run_delete`: `config.save()` (commands/login.rs:278) follows the `remove_dir_all` match (commands/login.rs:243-274). Failure mode row 6. |
| INV-3 | Deleting a profile MUST NOT change the shared `[[yubikeys]]` registry. | `remove_org` touches only the org map, the per-alias convenience state, and the active org (config/turnkey.rs:672-681). Behavioral checks: tvc/tests/login.rs:109-149 (V-8, V-9). |
| INV-4 | The command MUST delete files only by removing the default org directory, and only when every configured key path is a default path. | The `uses_default_layout` gate (commands/login.rs:238-241) guards the single deletion call, `remove_dir_all` (commands/login.rs:243-245). Vector V-10. |
| INV-5 | Non-interactive input validation MUST run before alias resolution and before any mutation. | The guard block at the top of `run_delete` (commands/login.rs:129-136). Vectors V-3, V-4. |

## Gaps (informative)

1. **[capability] No way to list profiles; the not-found hint points at a mutating command.** `tvc profile` has exactly one subcommand, `delete` (cli.rs:386-390). The profile-not-found error says to run `tvc login` to see configured profiles (commands/login.rs:305-308). `login` is interactive, sets the active org, and can generate keys as a side effect. Scripts have no read-only way to enumerate aliases and org ids before deleting. A `tvc profile list` is the missing sibling.

2. **[bug?] `--org <org-id> --yes` deletes a nondeterministic profile on a shared org id.** The `find_org` id fallback scans `config.orgs` in hash order (commands/login.rs:738-742). `config.orgs` is a `HashMap` (config/turnkey.rs:54). Nothing prevents two aliases from carrying the same org id: `add_org` never checks (config/turnkey.rs:634-663). Logging in to the same org twice creates the collision. Interactively the confirm prompt exposes the chosen alias. With `--yes` the command silently destroys an arbitrary one of the two profiles. A fix: fail on an ambiguous id match and list the aliases.

3. **[consistency] `--org` has no env or config equivalent though siblings have one.** `tvc login --org` reads `TVC_ORG` (commands/login.rs:36-37). `app delete` and `deploy delete` take their delete target from `TVC_APP_ID` and `TVC_DEPLOY_ID` (commands/app/delete.rs:21; commands/deploy/delete.rs:21). `DeleteArgs.org` is flag-only (commands/login.rs:52-55). The choice is plausibly deliberate for a destructive target. It still deviates from the LONG_ABOUT resolution-order contract (cli.rs:19-23) without a documented exception.

4. **[consistency] Piped-stdin interactive runs hit raw inquire TTY errors and miss the `missing_required_input` classification.** `run_delete` gates prompting only on `ctx.is_non_interactive()` (commands/login.rs:129-136). Its closest sibling, `yubikey unregister`, computes `can_prompt = !non_interactive && stdin_can_prompt()` and returns the missing-input error for `--serial` and `--yes` when stdin is no TTY (commands/yubikey/unregister.rs:33-41). Prompts need a real TTY (prompts.rs:2-5), so `echo | tvc profile delete` in human mode fails with an inquire IO error classified `command_error`. The actionable missing-input error never appears.

5. **[capability] JSON mode cannot tell which key files stay on disk.** Both warnings about retained files go through `shell_eprintln!` (output.rs:154-159). That macro writes only in human mode. The custom-key-path warning names the exact retained paths; the `key directory was not on disk` warning marks a skipped delete (commands/login.rs:247-274). The payload collapses both cases to `removedKeyDirectory: null` (commands/login.rs:243-296, 1023). A JSON consumer deleting a custom-layout profile gets a success message with no signal that files holding private key material remain, and no paths.

6. **[consistency] Profile-not-found classifies as `command_error` and misses `not_found`.** The taxonomy says `not_found` covers a resource that resolved to empty (cli.rs:58). `MissingResource` exists to trigger it (errors.rs:26, 93-96). `resolve_profile_alias` uses an untyped `bail!` (commands/login.rs:305-308), so machine consumers see `code: "command_error"` for a lookup miss that some consumers treat as idempotent success.

7. **[docs] Help text overpromises file deletion for custom layouts, and deletion is all-or-nothing.** The subcommand doc says `Permanently delete a saved login profile and its local key files` (cli.rs:388). One custom local operator `key_path` suppresses deletion of everything, including an API key file at its default location (commands/login.rs:238-274). Human mode warns; the help text and the JSON payload (gap 5) stay silent about the conditional.

8. **[docs] Deleting the active profile silently leaves no active org.** `remove_org` clears the active org (config/turnkey.rs:677-679). Neither the warning block (commands/login.rs:147-204) nor the `ProfileDeleted` outcome mentions that the profile was active or that no active org remains. The user discovers the change when the next org-dependent command fails. The picker labels the active profile (commands/login.rs:340); the `--org` plus `--yes` path never shows that label.
