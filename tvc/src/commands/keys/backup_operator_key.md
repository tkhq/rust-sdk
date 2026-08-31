# tvc keys backup-operator-key

## Purpose

Copies the org's registered **local** operator key file (`StoredQosOperatorKey` JSON,
private key included) byte-for-byte to a user-chosen destination, as a disaster-recovery
backup. Run it after `tvc login` generates a local operator key, or any time you want an
off-machine copy. Hosted and YubiKey operators are explicitly not exportable
(tvc/src/commands/keys/backup_operator_key.rs:69-74). Login's key-generation flow reuses
the same prompt + copy internals as an advisory nudge (tvc/src/commands/login.rs:891-904).

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| organization | `--org` (alias or org ID) | `TVC_ORG` | `active_org` in `~/.config/turnkey/tvc.config.toml` | active org | no |
| backup destination | `-o, --output` | `TVC_OPERATOR_KEY_BACKUP_OUT` | — | — | yes: text prompt, default `operator-<alias>-backup.json` (cwd) |
| overwrite existing destination | `--overwrite` | — | — | false | yes: confirm prompt, default No |
| source key file | — | — | the sole local operator's `key_path` (tvc/src/config/turnkey/mod.rs:441-457) | — | no — not choosable |

Resolution order is respected (flag/env then config); no deviations. `--org` shares
`TVC_ORG` with `tvc login` (tvc/src/commands/login.rs:36). Globals: the command widens
"non-interactive" to also cover a non-TTY stdin for the `--output` requirement
(tvc/src/commands/keys/backup_operator_key.rs:45).

## Interactive behavior

- The org is never prompted: `--org`/`TVC_ORG` or the active org, else a hard error
  (backup_operator_key.rs:51-61).
- The only prompts are destination-related, and only when `can_prompt`
  (interactive + TTY stdin):
  1. No `--output`: text prompt "Backup file path" defaulting to
     `operator-<alias>-backup.json`; a directory answer bails; an existing file asks
     "Overwrite <path>?" (default No) and declining cancels the whole command
     (backup_operator_key.rs:111-112, 123-144). No re-prompt for a different path.
  2. `--output` given, file exists, no `--overwrite`: `confirm_or_bail` "Overwrite?"
     (default No); declining errors "operation cancelled by user: backup"
     (backup_operator_key.rs:93-105).
- Non-interactive / JSON / non-TTY stdin: `--output` becomes a hard requirement,
  checked before config load or org resolution (backup_operator_key.rs:42-49); an
  existing destination without `--overwrite` is a hard error naming the flag
  (backup_operator_key.rs:94-99). JSON mode implies non-interactive
  (tvc/src/output.rs:210).

## Outputs

- Human: "Operator key backed up!" block with org alias, public key, source path,
  backup path, a private-key handling warning, and manual restore instructions
  (copy back + `tvc login`) (backup_operator_key.rs:211-233).
- JSON: one outcome, `reason: "operator_key_backed_up"`, fields `alias`, `publicKey`,
  `sourcePath`, `backupPath` (pinned by test, backup_operator_key.rs:315-335).
- Errors: `reason: "command_error"` (or `"missing_required_input"`) with the standard
  `code` taxonomy (tvc/src/output.rs:312-342).

## Side effects

- Reads `~/.config/turnkey/tvc.config.toml`; if absent, the dispatcher writes a fresh
  default config file before the command runs (tvc/src/cli.rs:219-223).
- Reads the registered operator key file and parses it as validation; then
  `fs::copy`s the file verbatim to the destination, creating parent directories
  (backup_operator_key.rs:157-185). Unknown JSON fields survive (test at :240-268).
- Destination permissions come from `fs::copy` propagating the source's mode
  (0600 for keys saved by current tvc — tvc/src/config/turnkey/qos_operator_key.rs:151,
  tvc/src/util.rs:38-56).
- No Turnkey API calls, no device interaction, no login-config mutation.

## Failure modes

All runtime failures exit 1; all classify `command_error` except the first row.

| failure | behavior |
|---|---|
| no `--output` and cannot prompt | `MissingRequiredInput("--output")` → reason/code `missing_required_input` (backup_operator_key.rs:47-49) |
| `--org` matches no profile | "Login profile '<q>' not found. Run `tvc login`..." (backup_operator_key.rs:52-57) |
| no active org (and no `--org`) | "No active organization. Run `tvc login` first." (backup_operator_key.rs:58-61) |
| org has no local operator | `NoLocalOperator` + context explaining hosted/YubiKey keys can't be exported (backup_operator_key.rs:69-74) |
| org has several local operators | `MultipleLocalOperators`, context = org alias only — dead end (backup_operator_key.rs:76-78) |
| destination is a directory | bail, both paths (backup_operator_key.rs:86-91, 130-135) |
| destination exists, non-interactive, no `--overwrite` | bail naming `--overwrite` (backup_operator_key.rs:94-99) |
| user declines overwrite | "operation cancelled by user: backup" (backup_operator_key.rs:101-104, 111-112) |
| source file missing | "No operator key found at <path>. Run `tvc login` first." (backup_operator_key.rs:157-161) |
| source not valid key JSON | "operator key at <path> is not a valid operator key file" (backup_operator_key.rs:166-171) |

## Gaps

1. **[capability] A second local operator in the org makes the command a dead end — there is no way to say which one to back up.**
   `select_local_operator` only succeeds for exactly one local record
   (tvc/src/config/turnkey/mod.rs:452-456), and the command surfaces
   `MultipleLocalOperators` with no selector flag and no prompt
   (backup_operator_key.rs:76-78). The sibling `keys re-encrypt-local-share` solves the
   same shape for YubiKeys with `--serial` plus an interactive picker
   (tvc/src/commands/keys/re_encrypt_local_share.rs:59-62, 131-161). The config schema
   allows several local records (`operators: Vec<OperatorRecord>`,
   tvc/src/config/turnkey/mod.rs:378) even though tvc itself currently only writes one —
   the state is reachable by hand-editing and the error variant exists for it. Same
   family: the source is always the registry `key_path`; there is no `--key-path` to back
   up a key file the registry doesn't know about.

2. **[capability] With orgs configured but none active, interactive mode errors instead of offering a choice — and the error hides the escape hatch.**
   `active_org_config()` returning `None` (e.g. after `tvc profile delete` of the active
   org, which clears `active_org` while other orgs remain,
   tvc/src/config/turnkey/mod.rs:672-679) yields "No active organization. Run `tvc login`
   first." (backup_operator_key.rs:58-61) even when prompting is possible and `--org`
   would resolve it; `tvc login` prompts an org picker in the comparable situation. The
   message never mentions `--org`.

3. **[consistency] `--overwrite` is the command's only input without an env-var equivalent.**
   `--org` and `--output` both have env vars (backup_operator_key.rs:28, 31), and sibling
   boolean flags get them (`TVC_DANGEROUS_SKIP_VERIFICATION`,
   re_encrypt_local_share.rs:65-66). A CI run driven by `TVC_OPERATOR_KEY_BACKUP_OUT`
   pointing at an existing file hard-fails (backup_operator_key.rs:94-99) with no
   env-only way to consent to replacement.

4. **[docs] The permissions comment on the copy is stale in both halves.**
   backup_operator_key.rs:181-182 claims the backup gets "default (umask) permissions,
   matching `StoredQosOperatorKey::save`; tightening both is tracked by TVC-241" — but
   `save` now writes 0600 via `write_owner_only_file`
   (tvc/src/config/turnkey/qos_operator_key.rs:151, tvc/src/util.rs:38-56), and
   `fs::copy` propagates the source's mode rather than applying the umask. Looks like
   TVC-241 landed for `save` and this comment (and possibly the copy side) was missed.

5. **[bug?] The bytes that were validated are not the bytes that get backed up.**
   The source is read and parsed once (backup_operator_key.rs:157-171), then the file is
   copied from disk again with `fs::copy` (:183-185); a concurrent write between the two
   reads produces a backup whose content was never validated. Writing the already-read
   `bytes` instead would keep the verbatim-copy guarantee (the test at :240-268 asserts
   content, not copy mechanics), close the race, and let the command set 0600 explicitly
   instead of inheriting the source's mode (ties into gap 4).
