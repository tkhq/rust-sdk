# tvc yubikey unregister

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the yubikey unregister command.*

## Purpose (informative)

`tvc yubikey unregister` removes one `[[yubikeys]]` entry from the tvc config.
The operation is local bookkeeping.
The command makes no Turnkey API call, performs no device I/O, and revokes nothing.
The device keeps working as an operator for every org that still trusts its public keys.
The pre-confirmation warning states exactly this.
Run it to forget a device on this machine, after every operator record that references it is gone.
`tvc profile delete` prints that follow-up hint (commands/login.rs:196).
Implementation: `Run for Args` (commands/yubikey/unregister.rs:32), dispatched from the `yubikey` subcommand (cli.rs:475-479).
Unqualified citations in this specification name files under `tvc/src`.

## Acceptance scenario (normative)

| Step | Action | Expected observation |
|---|---|---|
| 1 | Write a tvc config whose `[[yubikeys]]` registry holds one entry: serial `01c95c1f`, a cached operator public key of 130 `07` bytes, zero orgs (tvc/tests/keys_yubikey.rs:31-42). Set `TVC_NON_INTERACTIVE=1`. | Setup complete. |
| 2 | Run `tvc yubikey unregister --serial 01c95c1f --yes`. | Exit code 0. |
| 3 | Inspect stdout. | Line 1: `YubiKey 01c95c1f was removed from the local TVC configuration.` Line 2: `The device was not modified and no organization operator was revoked.` |
| 4 | Read the tvc config. | The `[[yubikeys]]` entry for `01c95c1f` is absent. Every other field is unchanged. |

All steps pass in one run from a clean start (pinned by tvc/tests/keys_yubikey.rs:176-197).

## Inputs (normative)

| Input | Flag | Env | Config key | Default | Interactive fallback |
|---|---|---|---|---|---|
| serial | `--serial <SERIAL>` | none | none | none | sole registered serial picked silently; picker when two or more serials are registered (commands/yubikey/unregister.rs:52-56) |
| confirmation | `-y, --yes` | none | none | false | y/N confirm, default No (commands/yubikey/unregister.rs:85-105) |
| non-interactive (global) | `--non-interactive` | `TVC_NON_INTERACTIVE` | none | false | none; makes `--serial` and `--yes` hard requirements (commands/yubikey/unregister.rs:35-41) |
| message format (global) | `--message-format` | none | none | `human` | none; `json` implies non-interactive mode (INV-G1) |

`--serial` MUST parse as a YubiKey serial: bare hex, surrounding whitespace allowed, at most 32 bits (config/turnkey/yubikey.rs:45-61).
`--serial` and `--yes` have no environment variable and no config key (commands/yubikey/unregister.rs:19-27).
The Part 00 value resolution order therefore collapses to flag, then default.
The command reads no command config.

## Interactive behavior (normative)

The command prompts only in interactive mode, as Part 00 defines it (commands/yubikey/unregister.rs:33; prompts.rs:17-19).
The gate needs `--non-interactive` absent, a TTY on stdin, and human output (INV-G1).

In interactive mode the command runs these steps in order.

1. Serial resolution. An explicit `--serial` MUST exist in the registry (commands/yubikey/unregister.rs:46-49). When `--serial` is absent and the registry is empty, the command MUST fail with `no YubiKeys are registered` (commands/yubikey/unregister.rs:53). When the registry holds exactly one serial, the command MUST pick it silently (commands/yubikey/unregister.rs:54). When the registry holds two or more serials, the command MUST show the `YubiKey to unregister` picker (commands/yubikey/unregister.rs:55). The picker lists serials in config order (config/turnkey/yubikey.rs:208-210).
2. Org-reference guard. The command MUST fail when any org holds an operator record with the chosen serial (commands/yubikey/unregister.rs:59-83). The Failure modes section gives the message.
3. Confirmation. Without `--yes`, the command MUST print a warning block on stderr (commands/yubikey/unregister.rs:85-103). The block states that the command only removes the entry from the tvc config. It states that the keys and certificates stay on the device. It states that no org revocation happens and that the device can still act as an operator. The command MUST then ask `Unregister YubiKey <serial>?` with default No (commands/yubikey/unregister.rs:104). A decline MUST fail the run with `operation cancelled by user: unregistration` (prompts.rs:70-75).

`--yes` MUST skip the warning block and the confirmation prompt (commands/yubikey/unregister.rs:85).

In non-interactive mode, and whenever stdin is no TTY, the command MUST validate `--serial`, then `--yes`, first (commands/yubikey/unregister.rs:35-41).
The validation runs before the registry read.
Each absent flag MUST produce the `missing_required_input` error that names the flag (prompts.rs:21-23; output.rs:283-293).
No prompt runs on that path.
Piped stdin in human mode without `--non-interactive` takes the same path.
The sibling `profile delete` gate skips the TTY test (commands/login.rs:129-136); its specification records that asymmetry as its gap 4.

## Outputs (normative)

In human mode the command MUST write the warning block to stderr and the outcome to stdout.
On success stdout MUST render exactly two lines (`Display`, commands/yubikey/unregister.rs:133-142):

```
YubiKey 01c95c1f was removed from the local TVC configuration.
The device was not modified and no organization operator was revoked.
```

Every serial renders in canonical form: lowercase hex, zero-padded to eight digits (config/turnkey/yubikey.rs:63-66).

In JSON mode the command MUST emit exactly one NDJSON object with `reason` = `yubikey_unregistered` (outcome.rs; commands/yubikey/unregister.rs:127-131).

| Field | Value |
|---|---|
| `serial` | The unregistered serial in canonical form. |

The acceptance scenario run with `--message-format json` emits:

```json
{"reason":"yubikey_unregistered","serial":"01c95c1f"}
```

The unit test pins this object (commands/yubikey/unregister.rs:148-161).
Errors follow the Part 00 error taxonomy (INV-G3).

## Side effects (normative)

1. The dispatch layer loads the tvc config before the command runs and creates the file when absent (cli.rs:219-223; INV-G4). A run that later fails can still create the tvc config on a fresh machine.
2. On success the command MUST remove the chosen `[[yubikeys]]` entry (`YubiKeyRegistry::deregister`, config/turnkey/yubikey.rs:195-197). It MUST then save the whole tvc config (commands/yubikey/unregister.rs:107-114).
3. The command makes no Turnkey API call, performs no device I/O, and changes no key files (INV-1; tvc/tests/keys_yubikey.rs:176-197).
4. The org-reference guard protects config loadability. Config parsing rejects any org operator record whose serial is absent from the registry (config/turnkey.rs:125-144). A saved tvc config with a dangling serial fails to load for every later command (INV-2).

## Failure modes (normative)

Each failure MUST exit through the Part 00 exit code and error taxonomy (INV-G2, INV-G3).
Every runtime failure exits with code 1; a parse failure exits with code 2.

| Failure | Observation | Mechanism |
|---|---|---|
| `--serial` absent when prompting is impossible | `reason` `missing_required_input`, `code` `missing_required_input`, message contains `--serial is required in non-interactive mode`; exit code 1 | commands/yubikey/unregister.rs:35-37; output.rs:326-333; tvc/tests/keys_yubikey.rs:88-99 |
| `--yes` absent when prompting is impossible | Same shape; the message names `--yes` | commands/yubikey/unregister.rs:39-41; tvc/tests/keys_yubikey.rs:101-110 |
| `--serial` not in the registry | `YubiKey <serial> is not in the registry`; `code` `command_error`; exit code 1 | commands/yubikey/unregister.rs:46-49 (defensive duplicate at 107-110); tvc/tests/keys_yubikey.rs:113-127 |
| Zero registered serials (interactive, no `--serial`) | `no YubiKeys are registered`; `code` `command_error`; exit code 1 | commands/yubikey/unregister.rs:53 |
| The chosen serial is an org operator | `YubiKey <serial> is an operator for organization(s) <aliases>; remove those operator records first`, aliases sorted and comma-joined; `code` `command_error`; exit code 1 | commands/yubikey/unregister.rs:59-83; tvc/tests/keys_yubikey.rs:129-149 |
| User declines the confirmation | `operation cancelled by user: unregistration`; `code` `command_error`; exit code 1 | prompts.rs:70-75 |
| tvc config save fails | Chain context `failed to unregister YubiKey <serial>`; `code` `command_error`; exit code 1 | commands/yubikey/unregister.rs:111-114 |
| Malformed `--serial` (non-hex) | `must be bare hex encoded`; exit code 2; in JSON mode one JSON object with `code` `usage_error` | config/turnkey/yubikey.rs:25-27; tvc/tests/keys_yubikey.rs:79-85; GV-1 |

## Test vectors (normative)

Fixture `registered`: a tvc config with one `[[yubikeys]]` entry and zero orgs (tvc/tests/keys_yubikey.rs:31-42).
The entry holds serial `01c95c1f` and a cached operator public key of 130 `07` bytes.
Fixture `referenced`: fixture `registered` plus org alias `test` with org id `org-test` (tvc/tests/keys_yubikey.rs:44-59).
That org's sole operator record is a yubikey record with serial `01c95c1f`.
Each vector starts from a clean home directory that holds only its Given column.
Part 00 lists the globally excluded nondeterministic fields; no field of this command's outcome is nondeterministic.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Fixture `registered`; `TVC_NON_INTERACTIVE=1` | `tvc yubikey unregister --serial 01c95c1f --yes` | stdout: `YubiKey 01c95c1f was removed from the local TVC configuration.` then `The device was not modified and no organization operator was revoked.`; the saved tvc config has no `[[yubikeys]]` entry; exit code 0 (tvc/tests/keys_yubikey.rs:176-197). |
| V-2 | Fixture `registered` | `tvc --message-format json yubikey unregister --serial 01c95c1f --yes` | stdout: exactly `{"reason":"yubikey_unregistered","serial":"01c95c1f"}`; exit code 0 (commands/yubikey/unregister.rs:148-161; INV-G3). |
| V-3 | Empty default tvc config; `TVC_NON_INTERACTIVE=1` | `tvc yubikey unregister` | stderr contains `--serial is required in non-interactive mode`; exit code 1 (commands/yubikey/unregister.rs:35-37; tvc/tests/keys_yubikey.rs:88-99). |
| V-4 | Empty default tvc config; `TVC_NON_INTERACTIVE=1` | `tvc yubikey unregister --serial 01c95c1f` | stderr contains `--yes is required in non-interactive mode`; exit code 1 (commands/yubikey/unregister.rs:39-41; tvc/tests/keys_yubikey.rs:101-110). |
| V-5 | Empty default tvc config; `TVC_NON_INTERACTIVE=1` | `tvc yubikey unregister --serial 01c95c1f --yes` | stderr contains `YubiKey 01c95c1f is not in the registry`; exit code 1 (commands/yubikey/unregister.rs:46-49; tvc/tests/keys_yubikey.rs:113-127). |
| V-6 | Empty default tvc config | `tvc --message-format json yubikey unregister --serial 01c95c1f --yes` | stdout: one JSON object, `reason` `command_error`, `code` `command_error`, message contains `YubiKey 01c95c1f is not in the registry`; exit code 1 (tvc/tests/keys_yubikey.rs:152-173; errors.rs:93-103). |
| V-7 | Fixture `referenced`; `TVC_NON_INTERACTIVE=1` | `tvc yubikey unregister --serial 01c95c1f --yes` | stderr contains `YubiKey 01c95c1f is an operator for organization(s) test; remove those operator records first`; the tvc config keeps the entry (the guard precedes the mutation); exit code 1 (commands/yubikey/unregister.rs:77-83; tvc/tests/keys_yubikey.rs:129-149). |
| V-8 | Fixture `registered`; TTY stdin; human mode; answer `n` at the confirm | `tvc yubikey unregister` | The command picks `01c95c1f` silently; stderr: the warning block, then `operation cancelled by user: unregistration`; the tvc config is unchanged; exit code 1 (commands/yubikey/unregister.rs:54, 85-105; prompts.rs:70-75). |
| V-9 | Fixture `registered` plus a second registry entry, serial `0a1b2c3d`; TTY stdin; pick `0a1b2c3d`, answer `y` | `tvc yubikey unregister` | The `YubiKey to unregister` picker lists `01c95c1f` and `0a1b2c3d` in config order; the saved tvc config keeps `01c95c1f` and drops `0a1b2c3d`; exit code 0 (commands/yubikey/unregister.rs:55; config/turnkey/yubikey.rs:195-210). |
| V-10 | Any tvc config | `tvc yubikey unregister --serial zzzz` | stderr contains `must be bare hex encoded`; exit code 2 (config/turnkey/yubikey.rs:25-27; tvc/tests/keys_yubikey.rs:79-85). |
| V-11 | Fixture `registered`; piped stdin; human mode; no `--non-interactive` | `tvc yubikey unregister` | stderr contains `--serial is required in non-interactive mode`; exit code 1 (commands/yubikey/unregister.rs:33-37). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT call the Turnkey API, open a YubiKey device, or change key files. | `run` (commands/yubikey/unregister.rs:32-117) builds no client and performs no device I/O; its only writes are `YubiKeyRegistry::deregister` and `Config::save`. Behavioral check: tvc/tests/keys_yubikey.rs:176-197 passes with no server and no device. |
| INV-2 | The command MUST NOT deregister a serial that an org operator record references. | The guard (commands/yubikey/unregister.rs:59-83) precedes the mutation (107-114). Config parsing rejects the dangling state (config/turnkey.rs:125-144), so a violation would leave the tvc config unloadable. Behavioral check: tvc/tests/keys_yubikey.rs:129-149 (V-7). |
| INV-3 | The command MUST mutate the tvc config only after confirmation: `--yes` or an accepted prompt. | Statement order in `run`: `confirm_or_bail` (commands/yubikey/unregister.rs:104) precedes `deregister` and `save` (107-114). Vector V-8. |
| INV-4 | Non-interactive input validation MUST run before the registry read and before any mutation. | The guard block at the top of `run` (commands/yubikey/unregister.rs:35-41) precedes the registry read (43). Vectors V-3, V-4, V-11. |
| INV-5 | A successful run MUST change the tvc config only by removing the one chosen `[[yubikeys]]` entry. | `deregister` removes one registry map entry (config/turnkey/yubikey.rs:195-197); `run` performs no other config mutation. Behavioral check: tvc/tests/keys_yubikey.rs:176-197 reparses the saved file. |

## Gaps (informative)

1. **[capability] The guard's remediation, `remove those operator records first`, has no CLI path.** `tvc operator` has exactly one subcommand, `create` (cli.rs:380-383; commands/operator.rs:3). The only ways to remove a single YubiKey operator record are `tvc profile delete`, which removes the whole profile, and hand-editing the tvc config. An org-referenced serial therefore cannot leave the registry through the CLI. The error text (commands/yubikey/unregister.rs:77-83) points the user at an operation that does not exist.

2. **[capability] No explicit register path and no way to list registered serials.** Registration lives inside `keys refresh-yubikey` (commands/keys/refresh_yubikey.rs:91). That command needs the physical device connected. No `yubikey register` takes a serial plus public key for offline setup. Yet refresh's save-failure message prints exactly that `[[yubikeys]]` TOML for hand editing (commands/keys/refresh_yubikey.rs:39-48). No `yubikey list` exists; the multi-key picker in this command is the CLI's only enumeration of registered serials. A non-interactive caller has to read the tvc config to learn what it can unregister.

3. **[consistency] The register/unregister pair is split across command groups with asymmetric names.** The inverse of `yubikey unregister` is `keys refresh-yubikey` (cli.rs:462-463 versus 475-479). `yubikey --help` shows unregister with no counterpart. Error text in `operator create` (commands/operator/create.rs:296-313) and in `login` routes users to the register verb. `refresh` does not read as `register` although it adds new entries (`Registration::Added`, commands/keys/refresh_yubikey.rs:123).

4. **[consistency] Non-interactive mode demands `--serial` even for a sole registered serial.** The interactive path picks a sole serial silently (commands/yubikey/unregister.rs:54). That choice is fully deterministic. The same situation in CI fails with `missing_required_input` (commands/yubikey/unregister.rs:35-37). Sibling `keys refresh-yubikey` picks the sole connected device in any mode (yubikey.rs:104-106). Requiring `--yes` non-interactively matches `profile delete` (commands/login.rs:129-136) and fits a destructive operation. Requiring `--serial` applies mode-dependent strictness to an unambiguous choice; the command's own interactive path disagrees.

5. **[consistency] The unknown-serial refusal does not list the registered serials.** `YubiKey <serial> is not in the registry` (commands/yubikey/unregister.rs:46-49) gives a non-interactive caller nothing to self-correct with. The analogous refusal in `ConnectedYubiKeys::choose` appends `; connected: <serials>` (yubikey.rs:95-102). This compounds gap 2 (no list command).

6. **[consistency][docs] Semantic refusals miss the `invalid_input` code.** LONG_ABOUT defines `invalid_input` as semantic validation failure in the command (cli.rs:56). `ErrorCode::InvalidInput` carries `#[allow(dead_code)]` and no crate code assigns it (errors.rs:54-56). `classify` recognizes only `MissingResource` and `TurnkeyClientError` (errors.rs:93-103). tvc/tests/keys_yubikey.rs:152-173 pins this command's registry refusal to `command_error`. The issue is repo wide; this command is a clean example, and the help text documents a code that no command can emit.

7. **[consistency] The multi-key picker offers serials that the next step refuses.** Selection runs before the org-reference guard (commands/yubikey/unregister.rs:43-57, then 59-83). The picker offers an org-referenced serial; the guard then refuses the pick. When orgs reference every registered serial, the picker is a guaranteed dead end. A picker that filters or annotates referenced serials would fail before the user chooses.

8. **[docs] The `--serial` help overstates prompting.** The help says `If not provided, prompts interactively` (commands/yubikey/unregister.rs:20-21). The command prompts only when the registry holds two or more serials; it takes a sole serial silently and fails on zero. The same phrasing pattern exists on `login --org` (commands/login.rs:34-35).
