# tvc keys refresh-yubikey

## Purpose

Reads the composite `encrypt_public ‖ sign_public` operator key off a connected
YubiKey's two PIV slot certificates and syncs the shared `[[yubikeys]]` device
registry in `tvc.config.toml`: an unregistered serial is added, a stale cached
key replaced, a matching entry left alone. Despite the "refresh" name it is the
CLI's device *registration* entry point — the remediation text in `login`,
`operator create`, `deploy approve`, and YubiKey pair resolution all direct the
user here (tvc/src/yubikey/pair.rs:124-126, tvc/src/commands/login.rs:448-450,
tvc/src/commands/operator/create.rs:298-300,
tvc/src/commands/deploy/approve.rs:284-286).

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| YubiKey serial | `--serial <SERIAL>` (bare hex, parsed to `YubiKeySerial`) | — | — | the sole connected device | never |

Inherited globals (`--non-interactive`, `--message-format`, `--color`) get no
special treatment: the command never prompts, so interactive and
non-interactive runs are identical. `--serial` is the only command input; it
exists at the flag level only (no env var, no config default), so the global
flag > env > config > default order is trivially satisfied but only its first
and last rungs exist.

## Interactive behavior

Nothing is ever prompted. Reading the slot certificates needs neither the PIN
nor a touch (tvc/src/commands/keys/refresh_yubikey.rs:87-88), so the whole
command runs unattended in both modes.

Device selection (`ConnectedYubiKeys::choose`, tvc/src/yubikey.rs:84-114):

- explicit `--serial` connected → chosen; not connected → error listing the
  connected serials;
- no `--serial`: zero devices → "no YubiKey is connected"; exactly one → chosen;
  several → hard error telling the user to unplug all but one or pass
  `--serial` — even when a TTY is available (no selection prompt; see Gaps).

## Outputs

Human mode: a one-line summary keyed to the registration outcome ("Serial was
not yet registered - added it to the tvc config." / "Registry entry refreshed -
its cached public key was stale." / "Registry already matches the device -
nothing to update."), followed by a `Serial:` / `Operator public key:` block
(tvc/src/commands/keys/refresh_yubikey.rs:120-137).

JSON mode: one terminal outcome, reason `yubikey_refreshed`
(tvc/src/outcome.rs:60), payload
`{"reason":"yubikey_refreshed","serial":"<8-hex>","operatorPublicKey":"<260-hex composite>","registration":"added"|"updated"|"unchanged"}`
(tvc/src/commands/keys/refresh_yubikey.rs:101-112,
tvc/src/config/turnkey/yubikey.rs:150-161).

## Side effects

- Config read at dispatch (`~/.config/turnkey/tvc.config.toml`,
  tvc/src/cli.rs:215-240); on a machine with no config, dispatch creates and
  saves a default one before the command runs (tvc/src/cli.rs:219-223).
- PC/SC discovery pass over all connected smartcards
  (tvc/src/yubikey.rs:61-69), then a second open of the chosen serial
  (tvc/src/yubikey.rs:118-123).
- Device reads only: both slot certificates plus key metadata for the status
  check (`verified_pair_public_key`, tvc/src/yubikey.rs:570-583) and the
  composite pair key from the certificates (tvc/src/yubikey.rs:606-612). No PIN
  verification, no touch, no device mutation.
- `tvc.config.toml` rewritten when registration is `Added` or `Updated`
  (tvc/src/commands/keys/refresh_yubikey.rs:59-63); untouched when `Unchanged`.
  Registry mutation preserves unknown TOML fields on an existing entry
  (tvc/src/config/turnkey/yubikey.rs:183-190).
- No network access, no Turnkey API calls, no activities.

## Failure modes

All runtime failures exit 1. `classify` recognizes only `MissingResource` and
`TurnkeyClientError` (tvc/src/errors.rs:93-103), and this command produces
neither, so every runtime error below carries `code: command_error`.

- Selection refusals: no device / serial not connected (with connected list) /
  several devices without `--serial` (tvc/src/yubikey.rs:93-113).
- Unusable device, refused before the key read: foreign certificate, key
  without a certificate, undeterminable slot state
  (tvc/src/yubikey.rs:337-356), or an empty slot ("holds no QuorumOS key",
  tvc/src/yubikey.rs:577-579 via 460-461).
- Config parse failure at dispatch, including the dangling-serial rejection
  when an org operator references a serial missing from the registry
  (tvc/src/config/turnkey/mod.rs:125-144) — see Gaps.
- Save failure after a successful device read: the error context embeds a
  ready-to-paste `[[yubikeys]]` TOML snippet (Added) or the replacement
  `public_key` line (Updated) so the user can finish the registration by hand
  (tvc/src/commands/keys/refresh_yubikey.rs:37-63).
- Malformed `--serial` (non-hex, >32 bits) fails clap value validation →
  `usage_error`, exit 2 (tvc/src/config/turnkey/yubikey.rs:45-61).

## Gaps

1. **[capability] Several connected devices cannot be selected interactively —
   the command hard-fails where siblings prompt.**
   `ConnectedYubiKeys::choose` bails on multiple devices even on a TTY
   (tvc/src/yubikey.rs:104-113); its stated rationale — a serial prompt cannot
   identify which stick to touch (tvc/src/yubikey.rs:82-84) — does not apply
   here, since refresh needs no PIN or touch
   (tvc/src/commands/keys/refresh_yubikey.rs:87-88). Siblings prompt in the
   same situation: `yubikey unregister` selects among registered serials
   (tvc/src/commands/yubikey/unregister.rs:55), `login` selects a registered
   YubiKey (tvc/src/commands/login.rs:702), `keys re-encrypt-local-share`
   selects among YubiKey operators
   (tvc/src/commands/keys/re_encrypt_local_share.rs:146-153). A prompt — or an
   `--all` that refreshes every connected device, which the touchless read
   makes safe — would close this.

2. **[consistency] The missing-serial refusal is a plain `command_error`, not
   `missing_required_input`, unlike the sibling that models the same
   situation.** The multi-device bail is `anyhow::bail!`
   (tvc/src/yubikey.rs:107-111), so JSON consumers see the fallback
   `command_error` code; `keys re-encrypt-local-share` wraps the analogous
   ambiguity in `MissingRequiredInput::new("--serial")`
   (tvc/src/commands/keys/re_encrypt_local_share.rs:155-158), which the output
   layer maps to the `missing_required_input` reason/code
   (tvc/src/output.rs:326-330). CI callers cannot machine-distinguish "supply
   --serial" from any other failure of this command.

3. **[consistency] A config the command could repair blocks the command from
   running.** Dispatch loads and validates the full config before the command
   sees it (tvc/src/cli.rs:215-240), and validation rejects any org operator
   whose serial is missing from the `[[yubikeys]]` registry, telling the user
   to hand-edit the TOML (tvc/src/config/turnkey/mod.rs:125-144). Re-adding
   that registry entry from the device is exactly what refresh-yubikey does,
   but it is unreachable in that state. `yubikey create-certs` shows the
   escape: it is dispatched before config loading precisely so device-local
   work never depends on the TVC config (tvc/src/cli.rs:207-212); refresh's
   device read is equally config-independent — only its final registry write
   needs the config.

4. **[docs] Help text omits the command's registration role.** Both the
   subcommand summary ("Refresh the registry's cached operator key for a
   YubiKey from the device", tvc/src/cli.rs:462-463) and the Args about
   (tvc/src/commands/keys/refresh_yubikey.rs:19-21, `long_about = None`)
   describe only refreshing an existing entry, yet adding an unregistered
   device (`Registration::Added`) is a first-class outcome and at least six
   error messages across login, operator create, deploy approve, and pair
   resolution tell the user to run this command to *register* a device
   (tvc/src/commands/login.rs:448-450,564-566,688-690,697-699,
   tvc/src/commands/operator/create.rs:298-300,
   tvc/src/commands/deploy/approve.rs:284-286, tvc/src/yubikey/pair.rs:124-126,
   136-140). A user reading `--help` cannot connect those instructions to this
   command. The help also never states that no PIN/touch is needed.

5. **[consistency] Command-group placement splits the registry's lifecycle
   across two groups.** This command adds/updates `[[yubikeys]]` entries but
   lives under `keys` (tvc/src/cli.rs:462-463), while the command that removes
   the same entries lives under `yubikey` (`yubikey unregister`,
   tvc/src/cli.rs:478-479). There is also no `yubikey list` to inspect the
   registry the pair manage — the only view of registered serials is reading
   the TOML.

6. **[bug?] Discovery silently drops devices that fail to open, producing a
   misleading "not connected" refusal.** `connected_serials` filters
   `reader.open().ok()` (tvc/src/yubikey.rs:65-68), so a YubiKey held open by
   another PC/SC client (or failing transiently) vanishes from the connected
   list; an explicit `--serial` for that physically present device is then
   refused as "YubiKey X is not connected" (tvc/src/yubikey.rs:95-103) instead
   of surfacing the open failure. Affects `yubikey create-certs` identically.
