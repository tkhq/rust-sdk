# tvc yubikey unregister

## Purpose
Removes a YubiKey's `[[yubikeys]]` registry entry from the local TVC config file. Purely
local bookkeeping: it never touches the device, never calls the Turnkey API, and never
revokes anything — the device keeps working as an operator for any org that still trusts
its keys (the command says exactly this before confirming). Run it to forget a device you
no longer use on this machine, typically after removing every org operator record that
references it (`tvc profile delete` prints this exact follow-up hint, login.rs:196).

## Inputs
| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| serial | `--serial <SERIAL>` (bare hex, parsed to `YubiKeySerial`) | — | — | sole registered key auto-picked (interactive path only) | yes — select list, only when 2+ keys registered |
| skip confirmation | `--yes` / `-y` | — | — | false | yes — y/N confirm (default No) unless `--yes` |
| non-interactive | `--non-interactive` (global) | `TVC_NON_INTERACTIVE` | — | false | n/a |

No resolution-order deviations — but neither flag has an env or config equivalent, so the
flag>env>config>default ladder collapses to flag-only here. Non-TTY stdin is treated like
non-interactive mode even without the flag (`can_prompt`, unregister.rs:33), which is
stricter than sibling `profile delete` (login.rs:129 checks only `is_non_interactive`).

## Interactive behavior
Order: (1) serial resolution — explicit `--serial` is verified against the registry;
otherwise 0 registered → bail, 1 → auto-picked silently, 2+ → select prompt
(unregister.rs:43-57); (2) org-reference guard (see Failure modes); (3) unless `--yes`,
a multi-line stderr warning ("only removes ... from the local TVC configuration ... does
not erase the keys ... does not revoke ...") followed by a y/N confirm defaulting to No
(unregister.rs:85-105).

In `--non-interactive` / JSON mode (JSON implies non-interactive, output.rs:210) — or
whenever stdin is not a TTY — both `--serial` and `--yes` become hard requirements,
checked up front before the registry is even read (unregister.rs:35-41); each missing one
errors as `missing_required_input`.

## Outputs
Human: warning block on stderr pre-confirmation; on success, stdout renders
"YubiKey {serial} was removed from the local TVC configuration. / The device was not
modified and no organization operator was revoked." (unregister.rs:133-142).

JSON: one terminal outcome, `reason: "yubikey_unregistered"` with `serial` (canonical
8-digit lowercase hex) — pinned by unit test unregister.rs:149-161. Errors emit
`reason: "command_error"` or `"missing_required_input"` per the global envelope.

## Side effects
Reads `~/.config/turnkey/tvc.config.toml` (dispatch creates a default file first if
absent, cli.rs:219-223 — so even a doomed run can create the config file). On success,
deletes the `[[yubikeys]]` entry (`YubiKeyRegistry::deregister`, yubikey.rs:195) and
rewrites the whole config via `config.save()`. No Turnkey API calls, no device I/O, no
key-file changes. Integration test pins the local-only effect (keys_yubikey.rs:176-197).

Design note: the org-reference guard is load-bearing, not just courtesy — config parsing
rejects any org operator record whose serial is missing from the registry
(config/turnkey/mod.rs:122-144), so deregistering a referenced serial would make the
config unloadable and brick every subsequent command until hand-edited.

## Failure modes
All runtime failures exit 1; bad flag syntax (e.g. non-hex serial → "must be bare hex
encoded", keys_yubikey.rs:79-85) is a clap parse failure, exit 2, `usage_error`.

- Missing `--serial` or `--yes` when prompting is impossible → `MissingRequiredInput`,
  code `missing_required_input` (unregister.rs:35-41).
- `--serial` not in the registry → "YubiKey {serial} is not in the registry"
  (unregister.rs:46-49; duplicated defensively at the deregister call, rs:107-110) —
  classifies as `command_error` (pinned by keys_yubikey.rs:152-173).
- No keys registered, no `--serial` (interactive) → "no YubiKeys are registered"
  (unregister.rs:53), `command_error`.
- Serial referenced by org operator record(s) → "YubiKey {serial} is an operator for
  organization(s) {sorted aliases}; remove those operator records first"
  (unregister.rs:77-83), `command_error`.
- User answers No → "operation cancelled by user: unregistration" (prompts.rs:70-75),
  `command_error`.
- Config save failure → chain context "failed to unregister YubiKey {serial}"
  (unregister.rs:111-114).

## Gaps
1. **[capability] The guard's remediation — "remove those operator records first" — has no CLI path.**
   `tvc operator` has only `create` (cli.rs:380-383, commands/operator/mod.rs:3); the only
   ways to remove a single YubiKey operator record are deleting the entire profile
   (`tvc profile delete`) or hand-editing tvc.config.toml. An org-referenced key is
   effectively un-unregisterable through the CLI (unregister.rs:77-83 points the user at
   an operation that does not exist).

2. **[capability] No explicit register path and no way to list registered keys.**
   Registration exists only inside `keys refresh-yubikey` (refresh_yubikey.rs:91), which
   requires the physical device connected; there is no `yubikey register` taking
   serial + public_key for offline setup, even though refresh's own save-failure message
   prints exactly that TOML for hand-editing (refresh_yubikey.rs:39-48), and no
   `yubikey list` — the multi-key select prompt in this command is the CLI's only
   enumeration of registered serials. Non-interactive callers must read the config file
   to learn what they can unregister.

3. **[consistency] The register/unregister pair is split across command groups with asymmetric names.**
   The inverse of `yubikey unregister` is `keys refresh-yubikey` (cli.rs:462-463 vs
   475-479); `yubikey --help` shows unregister with no counterpart, and users are routed
   to the register verb only via error text in `operator create` (create.rs:296-313) and
   `login`. "refresh" does not read as "register" even though it adds new entries
   (Registration::Added, refresh_yubikey.rs:123).

4. **[consistency] Non-interactive mode demands `--serial` even when exactly one key is registered.**
   The interactive path auto-picks a sole registered key with no prompt
   (unregister.rs:54) — a fully deterministic choice — yet the same situation in CI fails
   with `missing_required_input` (unregister.rs:35-37). Sibling `keys refresh-yubikey`
   auto-picks the sole connected device in any mode (yubikey.rs:104-106). Requiring
   `--yes` non-interactively is fair for a destructive op (matches `profile delete`,
   login.rs:129-136); requiring `--serial` is mode-dependent strictness on an unambiguous
   choice. (Defensible as explicitness-for-destruction; flagging because the command's
   own interactive path disagrees.)

5. **[consistency] The unknown-serial refusal does not list the registered serials.**
   "YubiKey {serial} is not in the registry" (unregister.rs:46-49) gives a non-interactive
   caller nothing to self-correct with, while the analogous refusal in
   `ConnectedYubiKeys::choose` appends "connected: {serials}" (yubikey.rs:95-103).
   Compounds gap 2 (no list command).

6. **[consistency][docs] Semantic refusals classify as `command_error` although the taxonomy reserves `invalid_input` for exactly this.**
   LONG_ABOUT defines `invalid_input` as "semantic validation failed in the command"
   (cli.rs:56), but `ErrorCode::InvalidInput` is `#[allow(dead_code)]` and never assigned
   anywhere in the crate (errors.rs:54-56; classify at errors.rs:93-103 only recognizes
   `MissingResource` and `TurnkeyClientError`). keys_yubikey.rs:169 pins this command's
   registry refusal to `command_error`. Repo-wide issue; this command is a clean example,
   and the help text documents a code no command can emit.

7. **[consistency] The multi-key select prompt offers serials that will be refused one step later.**
   Selection happens before the org-reference guard (unregister.rs:43-57 then 59-83), so
   an org-referenced key is offered, picked, then errors; when every registered key is
   referenced the prompt is a guaranteed dead end. Filtering or annotating referenced
   serials in the list would fail before the user chooses.

8. **[docs] `--serial` help overstates prompting.**
   "If not provided, prompts interactively" (unregister.rs:20-21) — it prompts only when
   2+ keys are registered; a sole key is taken silently and zero keys is an error. Minor,
   and the same phrasing pattern exists on `login --org` (login.rs:34-35).
