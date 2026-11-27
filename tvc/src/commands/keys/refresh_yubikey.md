# tvc keys refresh-yubikey

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the keys refresh-yubikey command.*

## Purpose (informative)

`tvc keys refresh-yubikey` reads the operator key from a connected YubiKey and syncs the device registry.
The **device registry** is the `[[yubikeys]]` list in the tvc config (tvc/src/config/turnkey.rs:55-59).
The key is the composite `encrypt_public ‖ sign_public` value that the device's two PIV slot certificates carry (yubikey.rs:570-582).
The command adds an unregistered serial, replaces a stale cached key, and leaves a matching entry untouched (registry.rs:169-192).
The refresh name understates the command's role: this is the CLI's device registration entry point.
Remediation text in `login`, `operator create`, `deploy approve`, and YubiKey pair resolution directs the user here.
(`tvc/src/commands/login.rs:448-450`, `tvc/src/commands/operator/create.rs:298-310`, `tvc/src/commands/deploy/approve.rs:284-286`, `tvc/src/yubikey/pair.rs:124-126`.)
In this specification, `refresh_yubikey.rs` means `tvc/src/commands/keys/refresh_yubikey.rs`. `yubikey.rs` means `tvc/src/yubikey.rs`. `registry.rs` means `tvc/src/config/turnkey/yubikey.rs`.

## Acceptance scenario (normative)

| Step | Action | Expected observation |
|---|---|---|
| 1 | Start from a clean home directory with one provisioned YubiKey connected, serial `01c95c1f`. | `~/.config/turnkey/tvc.config.toml` is absent. |
| 2 | Run `tvc keys refresh-yubikey`. | stdout prints `Serial was not yet registered - added it to the tvc config.`, then a `Serial:` line with `01c95c1f` and an `Operator public key:` line. Exit code 0. |
| 3 | Read the tvc config. | One `[[yubikeys]]` entry holds `serial = "01c95c1f"` and a 260-character hex `public_key`. |
| 4 | Run `tvc keys refresh-yubikey --serial 01c95c1f`. | stdout prints `Registry already matches the device - nothing to update.`. Exit code 0. The tvc config file is unchanged. |

All steps pass in one run from a clean start with one provisioned YubiKey connected (flow pinned by refresh_yubikey.rs:167-211).

## Inputs (normative)

| Input | Flag | Env | Config key | Default | Prompt |
|---|---|---|---|---|---|
| YubiKey serial | `--serial <SERIAL>` (bare hex, parsed to `YubiKeySerial`) | none | none | the sole connected device | never |

`--serial` MUST resolve per the Part 00 value resolution order.
It has no environment variable and no config key (refresh_yubikey.rs:22-27), so only the flag and the built-in default apply.
The command reads no command config.
The global `--non-interactive` flag changes nothing here: the command never prompts (INV-2).

## Interactive behavior (normative)

The command MUST NOT prompt in any mode (INV-2).
The slot certificate read needs no PIN and no touch (refresh_yubikey.rs:87-88), so both modes run unattended and identically.
Device selection MUST follow `ConnectedYubiKeys::choose` (yubikey.rs:84-114).
The mode MUST NOT change the selection result.

| `--serial` | Connected devices | Behavior |
|---|---|---|
| present and in the discovery list | any | The command MUST use that device (yubikey.rs:94). |
| present, absent from the list | any | The command MUST fail (yubikey.rs:95-103; see Failure modes). |
| absent | zero | The command MUST fail (yubikey.rs:105). |
| absent | one | The command MUST use the sole device (yubikey.rs:106). |
| absent | two or more | The command MUST fail in every mode, with no selection prompt (yubikey.rs:107-111; gap 1). |

## Outputs (normative)

In human mode the command MUST print one summary line for the registration outcome (refresh_yubikey.rs:120-137).
A blank line, a `Serial:` line, and an `Operator public key:` line follow.

| `registration` | Summary line |
|---|---|
| `added` | `Serial was not yet registered - added it to the tvc config.` |
| `updated` | `Registry entry refreshed - its cached public key was stale.` |
| `unchanged` | `Registry already matches the device - nothing to update.` |

In JSON mode the command MUST emit exactly one NDJSON object with `reason` = `yubikey_refreshed` (tvc/src/outcome.rs:30-31, 60).

| Field | Value |
|---|---|
| `serial` | The canonical serial: lowercase hex, zero-padded to eight digits (registry.rs:63-67). |
| `operatorPublicKey` | The composite key as 260 lowercase hex characters (tvc/src/config/turnkey/qos_operator_key.rs:75-79). |
| `registration` | `added`, `updated`, or `unchanged` (registry.rs:150-161; refresh_yubikey.rs:101-112). |

Failed runs MUST use the Part 00 error envelope (INV-G3).

## Side effects (normative)

- The tvc config load at dispatch follows INV-G4 (tvc/src/cli.rs:215-240). Dispatch creates and saves a default tvc config when the file is absent, even for a run that later fails (tvc/src/cli.rs:219-223).
- One PC/SC discovery pass runs over all connected smartcards (yubikey.rs:61-69). The command then opens the chosen device by serial (yubikey.rs:118-123).
- Device access MUST be read-only (INV-1). The command reads slot certificates and key metadata for the status check (yubikey.rs:570-582). It then reads the composite pair key from the certificates (yubikey.rs:606-612).
- The command MUST NOT verify the PIN and MUST NOT request a touch (refresh_yubikey.rs:87-90).
- The command MUST save the tvc config exactly when `registration` is `added` or `updated` (refresh_yubikey.rs:37-63; INV-3). It MUST NOT write the file for `unchanged`.
- A registry update MUST preserve unknown TOML fields and the entry's position (registry.rs:183-190; INV-4).
- No network I/O happens: the command builds no client and submits no activity.

## Failure modes (normative)

Runtime failures exit with code 1.
The malformed `--serial` row is a clap parse failure and exits with code 2.
`classify` recognizes `MissingResource` and `TurnkeyClientError` only (tvc/src/errors.rs:93-103).
This command produces neither type, so every runtime error carries `code` = `command_error`.

| Failure | Observation |
|---|---|
| Malformed `--serial`: empty, non-hex, or over 32 bits | clap value parse failure: exit code 2, JSON `code` = `usage_error` (registry.rs:45-61; the shared parser message is pinned end-to-end at tvc/tests/keys_yubikey.rs:79-85). |
| No device connected and `--serial` absent | `no YubiKey is connected`: exit code 1, `code` = `command_error` (yubikey.rs:105). |
| `--serial` absent from the discovery list | `YubiKey <serial> is not connected`, plus `; connected: <list>` when any device is present: exit code 1, `command_error` (yubikey.rs:95-103; unit test refresh_yubikey.rs:214-230). |
| Two or more devices and `--serial` absent | `multiple YubiKeys are connected (serials <list>); unplug all but the one to use and try again, or pass --serial`: exit code 1, `command_error` (yubikey.rs:107-111). |
| Device open failure after discovery | `failed to open YubiKey <serial>`, or `no connected YubiKey has serial <serial>` on a disconnect race: exit code 1, `command_error` (yubikey.rs:118-123, 387-396). |
| Unusable slot: foreign certificate, key without certificate, or undeterminable slot state | Refusal before the key read: exit code 1, `command_error` (yubikey.rs:337-356 through 570-575; messages yubikey.rs:436-458). |
| Empty slot | `the <slot> slot holds no QuorumOS key`: exit code 1, `command_error` (yubikey.rs:577-579, 460-461; unit test refresh_yubikey.rs:233-242). |
| tvc config unreadable, unparsable, or invalid at dispatch | Exit code 1 before the command runs (tvc/src/cli.rs:225-240); includes the dangling-serial rejection (tvc/src/config/turnkey.rs:125-144; gap 3). |
| tvc config save failure after a successful device read | Exit code 1; the error embeds a paste-ready `[[yubikeys]]` TOML fragment (`added`) or the replacement `public_key` line (`updated`) (refresh_yubikey.rs:37-63). |

## Test vectors (normative)

A real invocation needs a physical YubiKey; no integration test drives this command's binary path.
The unit tests drive the command flow with scripted discovery and an in-memory device (refresh_yubikey.rs:146-164).
The fake device serial is `01c95c1f` (tvc/src/yubikey/test_support.rs:15-17).
The fake generates a fresh key pair per run, so comparison excludes the operator public key hex.
Vector comparison also excludes the nondeterministic fields that Part 00 lists.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | No `[[yubikeys]]` entries; one provisioned device `01c95c1f` connected. | `tvc keys refresh-yubikey` | stdout summary `Serial was not yet registered - added it to the tvc config.`, a `Serial:` line with `01c95c1f`, and the operator public key line; the tvc config gains the `[[yubikeys]]` entry; exit code 0 (refresh_yubikey.rs:120-137; unit test refresh_yubikey.rs:167-180). |
| V-2 | Same as V-1. | `tvc keys refresh-yubikey --message-format json` | One NDJSON object: `reason` = `yubikey_refreshed`, `serial` = `01c95c1f`, `registration` = `added`, `operatorPublicKey` = the device's 260-character hex key (excluded from comparison); exit code 0 (refresh_yubikey.rs:101-112; tvc/src/outcome.rs:60). |
| V-3 | A `[[yubikeys]]` entry for `01c95c1f` whose `public_key` is `09` repeated 130 times; the same device connected. | `tvc keys refresh-yubikey --serial 01c95c1f` | stdout summary `Registry entry refreshed - its cached public key was stale.`; the saved entry's `public_key` equals the device key; exit code 0 (unit test refresh_yubikey.rs:183-196). |
| V-4 | A `[[yubikeys]]` entry for `01c95c1f` equal to the device key. | `tvc keys refresh-yubikey` | stdout summary `Registry already matches the device - nothing to update.`; no tvc config write; exit code 0 (unit test refresh_yubikey.rs:199-211; refresh_yubikey.rs:37-63). |
| V-5 | One device `01c95c1f` connected. | `tvc keys refresh-yubikey --serial deadbeef` | Error `YubiKey deadbeef is not connected; connected: 01c95c1f`; exit code 1; JSON `code` = `command_error` (unit test refresh_yubikey.rs:214-230; yubikey.rs:95-103). |
| V-6 | Zero devices connected. | `tvc keys refresh-yubikey` | Error `no YubiKey is connected`; exit code 1; `code` = `command_error` (yubikey.rs:105). |
| V-7 | Devices `01c95c1f` and `deadbeef` connected. | `tvc keys refresh-yubikey` | Error `multiple YubiKeys are connected (serials 01c95c1f, deadbeef); unplug all but the one to use and try again, or pass --serial`; exit code 1; `code` = `command_error` (yubikey.rs:107-111). |
| V-8 | One connected device with both QuorumOS slots empty. | `tvc keys refresh-yubikey` | Error chain contains `the signing slot holds no QuorumOS key`; exit code 1; `code` = `command_error` (yubikey.rs:577-579, 460-461; unit test refresh_yubikey.rs:233-242). |
| V-9 | Any state. | `tvc keys refresh-yubikey --serial zzzz` | stderr contains `must be bare hex encoded`; exit code 2; JSON `code` = `usage_error` (registry.rs:45-61; the same parser is pinned at tvc/tests/keys_yubikey.rs:79-85). |
| V-10 | The tvc config file read-only; unregistered device `01c95c1f` connected. | `tvc keys refresh-yubikey` | Exit code 1; the error contains `the key was read but saving the config failed` and a `[[yubikeys]]` fragment with `serial = "01c95c1f"` (refresh_yubikey.rs:39-48, 59-63). |

## Invariants (normative)

The global invariants INV-G1 through INV-G4 apply.

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT write to the device: no PIN verification, no touch, no slot mutation. | The flow calls only `verified_pair_public_key` (refresh_yubikey.rs:89-90), a read of slot status, certificates, and key metadata (yubikey.rs:570-582, 586-612). Behavioral check: the unit tests drive a fake device and never supply a PIN (refresh_yubikey.rs:167-211). |
| INV-2 | The command MUST NOT prompt in any mode. | The flow contains no prompt call (refresh_yubikey.rs:73-98); ambiguous selection fails (yubikey.rs:104-113). Behavioral check: the unit tests run on buffer-backed shells with no stdin (refresh_yubikey.rs:146-148). |
| INV-3 | The command MUST write the tvc config exactly when the device registry changed, and at most once. | `Config::save` runs only inside the `if let Some(manual_fix)` arm, which `Registration::Unchanged` skips (refresh_yubikey.rs:37-63). |
| INV-4 | A registry update MUST preserve the entry's position and its unknown TOML fields. | `register` edits `public_key` in place on the occupied IndexMap entry (registry.rs:183-190). Behavioral check: round-trip test registry.rs:250-264. |
| INV-5 | The device registry MUST hold at most one entry per serial. | The registry is an IndexMap keyed by serial; deserialization rejects duplicates (registry.rs:99-132). Behavioral checks: registry.rs:275-284, 311-321. |

## Gaps (informative)

1. **[capability] Interactive selection among several connected devices is missing; the command hard-fails where siblings prompt**.
   `ConnectedYubiKeys::choose` bails on multiple devices even on a TTY (yubikey.rs:104-113).
   Its stated rationale: a serial prompt cannot identify which physical stick the user intends to touch (yubikey.rs:81-83).
   That rationale does not apply here: this command needs no PIN and no touch (refresh_yubikey.rs:87-88).
   Siblings prompt in the same situation.
   `yubikey unregister` selects among registered serials (tvc/src/commands/yubikey/unregister.rs:55).
   `login` selects a registered YubiKey (tvc/src/commands/login.rs:702).
   `keys re-encrypt-local-share` selects among YubiKey operator records (tvc/src/commands/keys/re_encrypt_local_share.rs:146-153).
   A select prompt would close this gap.
   An `--all` flag that refreshes every connected device would also close it; the touchless read makes that safe.

2. **[consistency] The multi-device refusal carries the fallback `command_error` code; the sibling modeling the same ambiguity emits `missing_required_input`**.
   The multi-device bail is a plain `anyhow::bail!` (yubikey.rs:107-111), so JSON consumers see `command_error`.
   `keys re-encrypt-local-share` wraps the analogous ambiguity in `MissingRequiredInput::new("--serial")` (tvc/src/commands/keys/re_encrypt_local_share.rs:155-158).
   The output boundary maps that marker to the `missing_required_input` reason and code (tvc/src/output.rs:326-333).
   CI callers of this command cannot machine-distinguish the supply-a-serial case from any other failure.

3. **[consistency] A tvc config this command could repair blocks the command from running**.
   Dispatch loads and validates the full tvc config before the command sees it (tvc/src/cli.rs:215-240).
   Validation rejects any org operator record whose serial is missing from the device registry (tvc/src/config/turnkey.rs:125-144).
   The rejection tells the user to hand-edit the TOML.
   Re-adding that registry entry from the device is exactly what this command does, yet the command is unreachable in that state.
   `yubikey create-certs` shows the escape: dispatch runs it before config loading, so device-local work never depends on the tvc config (tvc/src/cli.rs:206-213).
   This command's device read is equally config-independent; only the final registry write needs the tvc config.

4. **[docs] Help text omits the command's registration role**.
   The subcommand summary describes only refreshing an existing entry (tvc/src/cli.rs:462-463).
   The Args doc repeats it, with `long_about = None` (refresh_yubikey.rs:19-21).
   Adding an unregistered device (`Registration::Added`) is a first-class outcome.
   At least six error messages across `login`, `operator create`, `deploy approve`, and pair resolution tell the user to run this command to register a device.
   (`tvc/src/commands/login.rs:448-450,564-566,688-690,697-699`, `tvc/src/commands/operator/create.rs:298-310`, `tvc/src/commands/deploy/approve.rs:284-286`, `tvc/src/yubikey/pair.rs:124-126,136-140`.)
   A user reading `--help` cannot connect those instructions to this command.
   The help also never states that the command needs no PIN and no touch.

5. **[consistency] Command-group placement splits the device registry's lifecycle across two groups**.
   This command adds and updates `[[yubikeys]]` entries yet lives under `keys` (tvc/src/cli.rs:462-463).
   The command that removes the same entries lives under `yubikey` (`yubikey unregister`, tvc/src/cli.rs:478-479).
   No `yubikey list` exists to inspect the registry the pair manage.
   The only view of registered serials is reading the TOML.

6. **[bug?] Discovery silently drops devices that fail to open, so an explicit `--serial` for a present device can get a misleading refusal**.
   `connected_serials` filters with `reader.open().ok()` (yubikey.rs:65-68).
   A YubiKey held open by another PC/SC client, or one failing transiently, vanishes from the connected list.
   An explicit `--serial` for that physically present device then fails as `YubiKey <serial> is not connected` (yubikey.rs:95-103).
   The open failure itself never surfaces.
   `yubikey create-certs` shares this discovery path and the same behavior.
