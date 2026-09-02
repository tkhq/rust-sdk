# tvc profile delete

## Purpose

Permanently deletes a saved login profile: its org entry in `~/.config/turnkey/tvc.config.toml` plus the API key and any local operator key files on disk. Purely local — it never calls the Turnkey API, never touches a YubiKey device or its shared `[[yubikeys]]` registry entry, and never revokes the dashboard-registered API key (it prints revocation instructions instead). Run it to remove a machine's credentials for an org you no longer use. Implementation: `run_delete` in tvc/src/commands/login.rs:118, dispatched from tvc/src/cli.rs:306-310.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| profile (alias or org ID) | `-o, --org <ORG>` | — | — | — | yes: picker over configured profiles (login.rs:311-328) |
| confirmation | `-y, --yes` | — | — | false | yes: y/N confirm, default No (login.rs:205-208) |
| non-interactive (global) | `--non-interactive` | `TVC_NON_INTERACTIVE` | — | false | makes `--org` and `--yes` hard requirements (login.rs:129-136) |
| message format (global) | `--message-format` | — | — | human | `json` implies non-interactive (output.rs:209-210) |

Deviations from flag > env > config > default: `--org` has no env var even though `tvc login --org` reads `TVC_ORG` (login.rs:36-37 vs 53-55). `--yes` has no env var. Nothing here reads a config-file value.

`--org` accepts either a profile alias (exact map-key match first) or an org ID (fallback scan) via `find_org` (login.rs:733-745).

## Interactive behavior

Order (human mode, TTY):

1. `--org` absent → `Select profile to delete` picker listing `alias (org-id)` with an `(active)` suffix on the active profile (login.rs:311-328). Zero profiles → bails `No login profiles to delete.` (login.rs:313-315).
2. `--yes` absent → multi-line WARNING block on stderr: what is deleted (config entry + key files, irreversible), what is not (the Turnkey dashboard API key stays valid), and — only when the profile references YubiKey operators — that devices and registry entries survive and `tvc yubikey unregister` is the follow-up (login.rs:147-204). Then `Permanently delete profile '<alias>' (<org-id>) and its key files?` with default No; declining bails `operation cancelled by user: deletion` (login.rs:205-208, prompts.rs:70-75).

Passing `--yes` skips both the warning block and the prompt entirely.

Non-interactive / JSON mode: `--org` and `--yes` are validated up front, before any other work; each missing one returns `MissingRequiredInput` (login.rs:129-136). No prompt is ever reached.

Caveat: in interactive mode with piped (non-TTY) stdin, the picker/confirm still run and fail inside `inquire` (prompts require a real TTY, prompts.rs:1-5) — there is no `stdin_can_prompt()` guard; see Gaps.

## Outputs

Human mode: warnings and progress on stderr; the final outcome on stdout (Display, login.rs:1032-1077):

- `Deleted login profile '<alias>' (<org-id>).`
- `Removed key directory: <dir>` (only when the default layout was deleted)
- YubiKey retention line naming kept serials + `tvc yubikey unregister` pointer (only when the profile referenced YubiKeys)
- `IMPORTANT:` dashboard revocation steps, naming the exact API public key when its file was readable before deletion, otherwise a generic "the API key associated with this profile".

JSON mode: one NDJSON outcome, reason `profile_deleted` (outcome.rs:30,36), fields `alias`, `organizationId`, `removedKeyDirectory` (null when nothing was deleted), `retainedYubikeySerials` (omitted when empty), `dashboardUrl`, `apiPublicKey` (null when unreadable) — login.rs:1018-1030. All `shell_eprintln!` warnings are suppressed in JSON mode (output.rs:246-276).

## Side effects

- Reads `~/.config/turnkey/tvc.config.toml` (dispatch, cli.rs:215-240). Note the dispatch layer creates a default config file when none exists (cli.rs:219-223) before the command then fails profile-not-found — shared behavior, not specific to this command.
- Best-effort read of the profile's API key file (before deletion) to name the public key in the revocation reminder (login.rs:215-222).
- Deletes `~/.config/turnkey/orgs/<alias>/` recursively iff `api_key_path` and every local operator `key_path` equal the default layout paths (login.rs:228-259; defaults in config/turnkey/mod.rs:573-585). Any custom path → nothing on disk is deleted; the custom paths (plus the API key path) are listed in a stderr warning (login.rs:260-274). A missing default directory is a warning, not an error (login.rs:247-254).
- Rewrites `tvc.config.toml` last, only after on-disk cleanup succeeded, so a failed delete stays retryable (login.rs:276-278). `remove_org` also drops the profile's `last_created_app_id` / `last_operator_ids` convenience state and clears `active_org` if it pointed at the deleted alias (config/turnkey/mod.rs:672-681).
- No Turnkey API calls, no device interaction. Shared `[[yubikeys]]` registry entries are never touched (verified by tests/login.rs:109-149).

## Failure modes

- Missing `--org` or `--yes` non-interactively → reason `missing_required_input`, code `missing_required_input`, exit 1 (login.rs:129-136, output.rs:326-333).
- `--org` matches nothing → `Login profile '<q>' not found. Run `tvc login` to see configured profiles.` — untyped bail, code `command_error`, exit 1 (login.rs:305-309; classify fallback errors.rs:93-102).
- No profiles configured (interactive, no `--org`) → `No login profiles to delete.`, `command_error`, exit 1.
- User declines the confirm → `operation cancelled by user: deletion`, `command_error`, exit 1.
- `remove_dir_all` failure other than NotFound → `failed to delete key directory: <dir>`, `command_error`, exit 1; config not yet saved, so the profile remains listed and the delete is retryable (login.rs:255-258, 276-278).
- Config save failure → `command_error`, exit 1; at this point key files are already gone but the profile is still listed — a retry warns `key directory was not on disk` and re-attempts the save.
- Bad flags/args → clap `usage_error`, exit 2 (cli.rs:144-182).

## Gaps

1. **[capability] There is no way to list profiles; the not-found hint points at a mutating command.** `tvc profile` has exactly one subcommand, `delete` (cli.rs:386-390), and the profile-not-found error says "Run `tvc login` to see configured profiles" (login.rs:306-308) — `login` is interactive, sets the active org, and can generate keys as a side effect. Scripts have no read-only way to enumerate aliases/org IDs before deleting; a `tvc profile list` is the missing sibling.

2. **[bug?] `--org <org-id> --yes` deletes a nondeterministic profile when two aliases share the org ID.** `find_org`'s ID fallback scans `config.orgs`, a `HashMap` (login.rs:738-744, config/turnkey/mod.rs:54), returning the first match in hash order; nothing prevents two aliases carrying the same org ID (e.g. the same org logged in twice, `add_org` never checks, config/turnkey/mod.rs:634-663). Interactively the confirm prompt exposes the chosen alias, but with `--yes` an arbitrary one of the two profiles is silently destroyed. Ambiguous ID matches should error out, listing the aliases.

3. **[consistency] `--org` has no env/config equivalent though siblings have one.** `tvc login --org` reads `TVC_ORG` (login.rs:36-37); `app delete` and `deploy delete` take their delete target from `TVC_APP_ID` / `TVC_DEPLOY_ID` (commands/app/delete.rs:21, commands/deploy/delete.rs:21). `DeleteArgs.org` is flag-only (login.rs:53-55). Plausibly deliberate for a destructive target, but it deviates from the LONG_ABOUT resolution-order contract (cli.rs:19-23) without being documented as an exception.

4. **[consistency] Piped-stdin interactive runs hit raw inquire TTY errors instead of `missing_required_input`.** `run_delete` gates prompting only on `ctx.is_non_interactive()` (login.rs:129-136), while its closest sibling `yubikey unregister` computes `can_prompt = !non_interactive && stdin_can_prompt()` and returns `MissingRequiredInput` for `--serial`/`--yes` when stdin is not a TTY (commands/yubikey/unregister.rs:33-41). Prompts require a real TTY (prompts.rs:1-5), so `echo | tvc profile delete` in human mode fails with an inquire IO error classified `command_error` instead of the actionable missing-input error.

5. **[capability] JSON mode cannot tell which key files were left on disk.** The custom-key-path retention warning (with the exact paths) and the "key directory was not on disk" warning are `shell_eprintln!`, suppressed in JSON mode (login.rs:247-274, output.rs:246-276); the payload collapses both cases to `removedKeyDirectory: null` (login.rs:243-296, 1020-1030). A JSON consumer deleting a custom-layout profile gets a success message with no signal that files still holding private key material remain, nor where they are.

6. **[consistency] Profile-not-found classifies as `command_error`, not `not_found`.** The taxonomy says `not_found` covers "a resource that resolved to empty" (cli.rs:58), and `MissingResource` exists to trigger it (errors.rs:26, 95-96), but `resolve_profile_alias` uses an untyped `bail!` (login.rs:305-309), so machine consumers see `code: "command_error"` for a lookup miss they may want to treat as idempotent success.

7. **[docs] Help text overpromises file deletion for custom layouts, and deletion is all-or-nothing.** The subcommand doc says "Permanently delete a saved login profile and its local key files" (cli.rs:388), but one custom local-operator `key_path` suppresses deletion of everything — including an API key file sitting at its default location (login.rs:238-274). Human mode warns; the help text (and JSON mode, gap 5) do not reflect the conditional.

8. **[docs] Deleting the active profile silently leaves no active org.** `remove_org` clears `active_org` (config/turnkey/mod.rs:677-679) but neither the warning block (login.rs:147-204) nor the `ProfileDeleted` outcome mentions the profile was active or that no active org remains; the user discovers it when the next org-dependent command fails. The picker labels `(active)` (login.rs:322), but `--org`/`--yes` paths never see that.
