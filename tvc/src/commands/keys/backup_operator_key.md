# tvc keys backup-operator-key

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the keys backup-operator-key command.*

## Purpose (informative)

The command copies the org's registered local operator key file to a chosen destination file.
The copy preserves every byte, private key included, so a restored copy brings back manifest signing after machine loss.
Only the `local` operator kind is exportable: Turnkey holds hosted operators' private keys, and YubiKey private keys never leave the device (`tvc/src/commands/keys/backup_operator_key.rs:69-75`).
Citations of the form `backup_operator_key.rs:NN` refer to that file.
The key generation flow of `tvc login` reuses the same prompt and copy internals as an advisory backup nudge (`tvc/src/commands/login.rs:879-904`).

## Acceptance scenario (normative)

Every example in this specification uses one worked universe.

- `HOME` is `/home/op`. The tvc config sets the active org to `acme`.
- Org `acme` has id `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` and exactly one operator record. That record has kind `local` and `key_path` `/home/op/.config/turnkey/orgs/acme/operator.json`.
- The key file holds the JSON below. `<hex260>` stands for one fixed string of 260 lowercase hex characters that parses as the composite public key (`tvc/src/config/turnkey/qos_operator_key.rs:65-73`). `<hex64>` stands for one fixed string of 64 hex characters.

```json
{
  "public_key": "<hex260>",
  "private_key": "<hex64>",
  "future_field": 42
}
```

| Step | Action | Expected observation |
|---|---|---|
| 1 | Start from the worked universe. Confirm `/home/op/backups/acme-backup.json` does not exist. | The destination is absent. |
| 2 | Run `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json`. | Exit code 0. stdout begins `Operator key backed up!` and shows org `acme`, public key `<hex260>`, the source path, and the backup path (`backup_operator_key.rs:211-233`). |
| 3 | Compare the backup with the source file. | The two files are byte for byte identical; `future_field` survives (`backup_operator_key.rs:240-268`). |
| 4 | Run `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json --overwrite --message-format json`. | Exit code 0. stdout is one NDJSON object with `reason` = `operator_key_backed_up` and `backupPath` = `/home/op/backups/acme-backup.json` (`backup_operator_key.rs:315-335`). No prompt appears. |

Pass criterion: all steps MUST pass in one run from a clean start.

## Inputs (normative)

| Input | Flag | Environment variable | tvc config source | Default | Prompt |
|---|---|---|---|---|---|
| org | `--org <ORG>` (alias or org id) | `TVC_ORG` (`backup_operator_key.rs:28`) | the active org | the active org | never |
| destination | `-o, --output <PATH>` (`backup_operator_key.rs:31`) | `TVC_OPERATOR_KEY_BACKUP_OUT` | none | none | interactive mode: text prompt `Backup file path`, default answer `operator-<alias>-backup.json` |
| overwrite consent | `--overwrite` (`backup_operator_key.rs:34-35`) | none (gap 3) | none | `false` | interactive mode: confirm prompt when the destination exists, default No |
| source key file | none | none | the sole local operator record's `key_path` (`tvc/src/config/turnkey.rs:441-457`) | none | never |

Value resolution MUST follow Part 00: flag first, then environment variable, then the tvc config.
The command MUST resolve the org from `--org` when set, else from the active org (`backup_operator_key.rs:51-61`).
`--org` accepts an org alias or an org id; an alias match wins (`tvc/src/commands/login.rs:733-745`).
`--org` shares `TVC_ORG` with `tvc login` (`tvc/src/commands/login.rs:36`).
The source key file is never a direct input; the sole local operator record supplies it (gap 1).

## Interactive behavior (normative)

- The command MUST NOT prompt for the org (`backup_operator_key.rs:51-61`).
- In non-interactive mode the command MUST reject a missing `--output` before it resolves the org (`backup_operator_key.rs:45-49`). This failure is the `missing_required_input` row of Failure modes.
- In interactive mode with no `--output`, the command MUST prompt `Backup file path` with the default answer `operator-<alias>-backup.json` (`backup_operator_key.rs:124-128`). The default resolves in the current working directory.
- The command MUST reject a prompted destination that names a directory (`backup_operator_key.rs:130-135`).
- When the prompted destination exists, the command MUST ask `Overwrite <path>?` with default No (`backup_operator_key.rs:137-141`). A decline MUST cancel the command with the error `operation cancelled by user: backup` (`backup_operator_key.rs:111-112`). The command MUST NOT offer a second path prompt.
- With `--output` set, an existing destination, and no `--overwrite`: in interactive mode the command MUST ask `Overwrite <path>?` with default No (`backup_operator_key.rs:101-104`). In non-interactive mode the command MUST fail with an error that names `--overwrite` (`backup_operator_key.rs:94-99`).
- With `--overwrite` set the command MUST skip the overwrite confirmation (`backup_operator_key.rs:93`).
- JSON mode implies non-interactive mode (INV-G1).

## Outputs (normative)

- In human mode the command MUST print one report block on stdout (`backup_operator_key.rs:211-233`; `tvc/src/output.rs:88-95`). The block begins `Operator key backed up!` and shows the org alias, the public key, the source path, and the backup path. The block ends with a private key handling warning and restore instructions (copy the file back, then run `tvc login`).
- In JSON mode the command MUST emit exactly one NDJSON object on stdout with `reason` = `operator_key_backed_up` and fields `alias`, `publicKey`, `sourcePath`, `backupPath` (`backup_operator_key.rs:315-335`).
- In human mode a runtime failure MUST print `error: <chain>` on stderr (`tvc/src/output.rs:161-182`).
- In JSON mode a failed run MUST emit one NDJSON error object per Part 00 (INV-G3). A missing `--output` MUST carry `reason` = `missing_required_input`; every other failure MUST carry `reason` = `command_error` (`tvc/src/output.rs:315-342`).
- Every failure of this command classifies as `code` = `command_error` or `code` = `missing_required_input` (`tvc/src/errors.rs:93-102`; `tvc/src/output.rs:326-333`).

## Side effects (normative)

- The dispatcher loads the tvc config before the command runs and creates the file when absent (INV-G4; `tvc/src/cli.rs:219-223`).
- The command MUST NOT write the tvc config (INV-5).
- The command MUST read the source key file and parse it as `StoredQosOperatorKey` before any write (`backup_operator_key.rs:157-171`).
- The command MUST create missing parent directories of the destination (`backup_operator_key.rs:175-179`).
- The command MUST copy the source file verbatim with `tokio::fs::copy` (`backup_operator_key.rs:183-185`). The copy reads the file from disk a second time (gap 5).
- The destination permission bits come from the copy, which carries over the source file mode. Keys that current tvc saves have mode 0600 (`tvc/src/config/turnkey/qos_operator_key.rs:151`; `tvc/src/util.rs:38-56`).
- The command makes no Turnkey API call and touches no device (INV-5).

## Failure modes (normative)

Every runtime failure MUST exit with code 1 (INV-G2).
In JSON mode the first row MUST carry `code` = `missing_required_input`; every other row MUST carry `code` = `command_error` (`tvc/src/output.rs:326-342`; `tvc/src/errors.rs:93-102`).
The renderer joins error chain layers with `: ` (`tvc/src/errors.rs:142-152`).

| Condition | Observable error | Source |
|---|---|---|
| No `--output` in non-interactive mode | `--output is required in non-interactive mode (set --output or run in a TTY without --non-interactive / TVC_NON_INTERACTIVE=true)` | `backup_operator_key.rs:47-49`; `tvc/src/output.rs:284-292` |
| `--org` matches no org | ``Login profile '<query>' not found. Run `tvc login` to see configured profiles.`` | `backup_operator_key.rs:52-57` |
| No `--org` and no active org | ``No active organization. Run `tvc login` first.`` | `backup_operator_key.rs:58-61` |
| The org has no local operator record | context `org '<alias>' has no local operator key file to back up; hosted operators' private keys are held by Turnkey, and YubiKey operators' private keys never leave the device — neither can be exported` over source `no local operator is configured` | `backup_operator_key.rs:69-75`; `tvc/src/config/turnkey.rs:388` |
| The org has several local operator records | context `org '<alias>'` over source `multiple local operators are configured` | `backup_operator_key.rs:76-78`; `tvc/src/config/turnkey.rs:390-391` |
| The destination names a directory (flag or prompt) | `destination <path> is a directory; include a file name` | `backup_operator_key.rs:86-91, 130-135` |
| The destination exists, non-interactive mode, no `--overwrite` | `destination <path> already exists; pass --overwrite to replace it` | `backup_operator_key.rs:94-99` |
| The user declines an overwrite prompt | `operation cancelled by user: backup` | `backup_operator_key.rs:101-104, 111-112`; `tvc/src/prompts.rs:70-75` |
| The source key file is absent | ``No operator key found at <path>. Run `tvc login` first.`` | `backup_operator_key.rs:157-161` |
| The source fails to parse as key JSON | context `operator key at <path> is not a valid operator key file` over the serde error | `backup_operator_key.rs:166-171` |

Other I/O failures surface as anyhow context over the cause (informative, no dedicated vectors):

- `failed to read operator key: <path>` (`backup_operator_key.rs:162-163`).
- `failed to create backup directory: <parent>` (`backup_operator_key.rs:175-179`).
- `failed to write backup: <path>` (`backup_operator_key.rs:183-185`).

## Test vectors (normative)

Every vector starts from the acceptance scenario universe; the Given column states the deltas.
Vector comparison excludes the Part 00 nondeterministic fields.
Every compared field of this command is deterministic in this universe.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | `/home/op/backups/acme-backup.json` absent; interactive mode. | `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json` | Exit 0. stdout block begins `Operator key backed up!`, shows `acme`, `<hex260>`, both paths (`backup_operator_key.rs:211-233`). Destination bytes equal source bytes, `future_field` included (test `backup_operator_key.rs:240-268`). No prompt appears. |
| V-2 | Destination absent. | `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json --message-format json` | Exit 0. stdout is exactly one NDJSON object: `{"reason":"operator_key_backed_up","alias":"acme","publicKey":"<hex260>","sourcePath":"/home/op/.config/turnkey/orgs/acme/operator.json","backupPath":"/home/op/backups/acme-backup.json"}` (test `backup_operator_key.rs:315-335`). |
| V-3 | No delta. | `tvc keys backup-operator-key --org ghost --message-format json` | Exit 1. One NDJSON object, `reason` = `missing_required_input`, `code` = `missing_required_input`, message `--output is required in non-interactive mode (set --output or run in a TTY without --non-interactive / TVC_NON_INTERACTIVE=true)`. The invalid `--org` never surfaces: the output check runs first (`backup_operator_key.rs:45-49`; `tvc/src/output.rs:284-292, 326-333`). |
| V-4 | No delta. | `tvc keys backup-operator-key --org ghost --output /home/op/backups/acme-backup.json --message-format json` | Exit 1. `code` = `command_error`, message ``Login profile 'ghost' not found. Run `tvc login` to see configured profiles.`` (`backup_operator_key.rs:52-57`). |
| V-5 | The tvc config has org `acme` and no active org. | `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json --message-format json` | Exit 1. `code` = `command_error`, message ``No active organization. Run `tvc login` first.`` (`backup_operator_key.rs:58-61`). |
| V-6 | Org `acme` has one `yubikey` operator record and no `local` record. | `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json --message-format json` | Exit 1. `code` = `command_error`, message `org 'acme' has no local operator key file to back up; hosted operators' private keys are held by Turnkey, and YubiKey operators' private keys never leave the device — neither can be exported: no local operator is configured` (`backup_operator_key.rs:69-75`; `tvc/src/config/turnkey.rs:388`). |
| V-7 | Org `acme` has two `local` operator records (hand edited). | `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json --message-format json` | Exit 1. `code` = `command_error`, message `org 'acme': multiple local operators are configured` (`backup_operator_key.rs:76-78`; `tvc/src/config/turnkey.rs:390-391`). |
| V-8 | `/home/op/backups` exists as a directory. | `tvc keys backup-operator-key --output /home/op/backups --message-format json` | Exit 1. `code` = `command_error`, message `destination /home/op/backups is a directory; include a file name` (`backup_operator_key.rs:86-91`). |
| V-9 | `/home/op/backups/acme-backup.json` exists. | `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json --non-interactive` | Exit 1. Human stderr `error: destination /home/op/backups/acme-backup.json already exists; pass --overwrite to replace it` (`backup_operator_key.rs:94-99`). |
| V-10 | Destination exists; interactive mode; answer `n` at `Overwrite /home/op/backups/acme-backup.json?`. | `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json` | Exit 1. stderr `error: operation cancelled by user: backup`; the destination is unchanged (`backup_operator_key.rs:101-104`; `tvc/src/prompts.rs:70-75`). |
| V-11 | The file at `key_path` is absent. | `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json --message-format json` | Exit 1. `code` = `command_error`, message ``No operator key found at /home/op/.config/turnkey/orgs/acme/operator.json. Run `tvc login` first.`` (`backup_operator_key.rs:157-161`; test `backup_operator_key.rs:270-290`). |
| V-12 | The file at `key_path` holds `not json`. | `tvc keys backup-operator-key --output /home/op/backups/acme-backup.json --message-format json` | Exit 1. `code` = `command_error`, message begins `operator key at /home/op/.config/turnkey/orgs/acme/operator.json is not a valid operator key file` (the chain appends the serde cause) (`backup_operator_key.rs:166-171`; test `backup_operator_key.rs:292-313`). |

## Invariants (normative)

Global invariants INV-G1 through INV-G4 apply. The invariants below are specific to this command.

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The backup MUST equal the source file byte for byte at copy time; unknown JSON fields survive. | `back_up` copies the source with `tokio::fs::copy` and never re-serializes (`backup_operator_key.rs:183-185`). Behavioral check: `backs_up_key_bytes_verbatim` (`backup_operator_key.rs:240-268`). |
| INV-2 | The command MUST NOT replace an existing destination without consent (`--overwrite`, or a confirmed prompt). | Existence checks fence both destination paths before the copy (`backup_operator_key.rs:93-105, 137-141`). A decline cancels the command (`backup_operator_key.rs:101-104, 111-112`). |
| INV-3 | Only a `local` operator record MAY supply the source file. | `select_local_operator` filters records to `OperatorRecordKind::Local`; other kinds yield typed errors (`tvc/src/config/turnkey.rs:444-456`). |
| INV-4 | The command MUST NOT prompt in non-interactive mode. | Every prompt sits behind the `can_prompt` gate (`backup_operator_key.rs:45, 47, 94, 111`). JSON mode forces non-interactive mode (INV-G1). |
| INV-5 | The command MUST NOT write the tvc config and MUST NOT call the Turnkey API. | `run` never calls `Config::save`, and the module imports no API client (`backup_operator_key.rs:4-19`). Filesystem writes touch only the destination path (`backup_operator_key.rs:175-185`). |

## Gaps (informative)

1. **[capability] A second local operator record makes the command a dead end: no flag or prompt chooses which record to back up**.
   `select_local_operator` only succeeds for exactly one local record (`tvc/src/config/turnkey.rs:452-456`).
   The command surfaces `MultipleLocalOperators` with no selector flag and no prompt (`backup_operator_key.rs:76-78`).
   The sibling `keys re-encrypt-local-share` solves the same shape for YubiKeys with `--serial` plus an interactive picker (`tvc/src/commands/keys/re_encrypt_local_share.rs:59-62, 131-161`).
   The config schema allows several local records (`operators: Vec<OperatorRecord>`, `tvc/src/config/turnkey.rs:378`) even though tvc itself writes at most one.
   The state is reachable by hand editing, and the error variant exists for it.
   Same family: the source is always the registry `key_path`; no `--key-path` flag exists to back up a key file outside the registry.

2. **[capability] With orgs configured and none active, interactive mode errors without offering a choice, and the error hides the escape hatch**.
   `active_org_config()` returns `None` after `tvc profile delete` of the active org, which clears the active org while other orgs remain (`tvc/src/config/turnkey.rs:672-681`).
   The command then reports the no-active-org error (`backup_operator_key.rs:58-61`) even when prompting is possible and `--org` would resolve it.
   `tvc login` prompts an org picker in the comparable situation.
   The message never mentions `--org`.

3. **[consistency] `--overwrite` is the command's only input without an environment variable**.
   `--org` and `--output` both have one (`backup_operator_key.rs:28, 31`).
   Sibling boolean flags get them, for example `TVC_DANGEROUS_SKIP_VERIFICATION` (`tvc/src/commands/keys/re_encrypt_local_share.rs:65-66`).
   A CI run that points `TVC_OPERATOR_KEY_BACKUP_OUT` at an existing file hard fails (`backup_operator_key.rs:94-99`).
   No environment variable can grant the overwrite consent.

4. **[docs] The permissions comment on the copy is stale in both halves**.
   `backup_operator_key.rs:181-182` claims the backup gets default (umask) permissions, matching `StoredQosOperatorKey::save`, with tightening tracked by TVC-241.
   Today `save` writes mode 0600 through `write_owner_only_file` (`tvc/src/config/turnkey/qos_operator_key.rs:151`; `tvc/src/util.rs:38-56`).
   And `fs::copy` carries the source file mode over; the umask plays no part.
   TVC-241 landed for `save`; the comment, and possibly the copy side, did not follow.

5. **[bug?] The validated bytes and the backed up bytes can differ**.
   The command reads and parses the source once (`backup_operator_key.rs:157-171`).
   The copy then reads the file from disk a second time with `fs::copy` (`backup_operator_key.rs:183-185`).
   A concurrent write between the two reads yields a backup that skipped validation.
   Writing the already read `bytes` would keep the verbatim copy guarantee and close the race.
   It would also allow an explicit 0600 mode on the backup (ties into gap 4).
   The test at `backup_operator_key.rs:240-268` asserts content and leaves the copy mechanics free.
