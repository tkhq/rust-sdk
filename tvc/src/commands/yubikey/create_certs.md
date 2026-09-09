# tvc yubikey create-certs

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the yubikey create-certs command.*

## Purpose (informative)

The command rebuilds the two self-signed `CN=QuorumOS` certificates for P-256 keys that already live in a YubiKey's PIV signing (9c) and key-agreement (9d) slots.
It writes each certificate as a PEM file for later import with `ykman`.
Run it when a device holds the QuorumOS keys and lacks the matching slot certificates.
The command works on the local device alone.
It reads slot metadata, verifies the PIN, and signs on the device.
It never writes to the device, never touches the tvc config, and never uses the network.

## Acceptance scenario (normative)

The scenario starts with no `tvc-yubikey-*.pem` files in the working directory.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Connect one YubiKey, serial `01c95c1f`, with device-generated P-256 keys in slots 9c and 9d, PIN and touch policies `Always`. | Discovery reports the sole serial `01c95c1f` (yubikey.rs:61-69). |
| 2 | With stdin a TTY, run `tvc yubikey create-certs`. | The masked prompt `YubiKey PIV PIN (touch the device once for each certificate)` appears (create_certs.rs:44-46). |
| 3 | Enter the correct PIV PIN, then touch the device when it blinks. | The signing certificate builds and verifies in memory (yubikey.rs:217-278). |
| 4 | Touch the device again when it blinks. | The key-agreement certificate builds and verifies in memory. |
| 5 | Inspect stdout and the working directory. | stdout shows the human success block (Outputs); `tvc-yubikey-01c95c1f-signing.pem` and `tvc-yubikey-01c95c1f-key-agreement.pem` exist; exit code 0. |

All steps pass in one run from a clean start with one connected device.

## Inputs (normative)

File citations use paths relative to `tvc/src/`; `create_certs.rs` means `tvc/src/commands/yubikey/create_certs.rs` and `yubikey.rs` means `tvc/src/yubikey.rs`.

| Input | Flag | Env | Config key | Default | Prompt |
|---|---|---|---|---|---|
| Device serial | `--serial <SERIAL>`: bare hex, at most 32 bits (config/turnkey/yubikey.rs:45-61) | none | none | the sole connected device (yubikey.rs:104-113) | never |
| PIV PIN | none | none | none | none | always: masked, no confirmation (create_certs.rs:44-46, prompts.rs:83-85) |
| Non-interactive | `--non-interactive` (global) | `TVC_NON_INTERACTIVE` (cli.rs:72-79) | none | false | n/a |
| Message format | `--message-format` (global) | none | none | `human` | n/a |

The command MUST resolve the serial from `--serial`, or default to the sole connected device (yubikey.rs:93-113).
No environment variable or tvc config source exists for the serial: the command runs before the tvc config loads (INV-2).
The command MUST read the PIN from the masked prompt only.
This matches the repo policy for YubiKey PINs (commands/deploy/approve.rs:407-412, commands/keys/re_encrypt_local_share.rs:164-169).

The command hard-codes these values. No flag changes them.

| Fixed value | Value | Evidence |
|---|---|---|
| Slots | both, signing (9c) then key-agreement (9d) | create_certs.rs:42-49 |
| Certificate subject | `CN=QuorumOS` | yubikey.rs:52 |
| Certificate validity | about ten years from build time | yubikey.rs:54-56, 238 |
| Output paths | `tvc-yubikey-<serial>-signing.pem` and `tvc-yubikey-<serial>-key-agreement.pem` in the current working directory | create_certs.rs:57-59 |

## Interactive behavior (normative)

The command MUST refuse to run in non-interactive mode and when stdin is not a TTY (create_certs.rs:33-38).
The refusal message is `creating YubiKey certificates is interactive: the PIN is prompted and the device must be touched once for each certificate`.
JSON mode implies non-interactive mode (INV-G1), so every JSON mode invocation MUST fail (V-4).
No non-interactive escape hatch exists, per the PIN policy above.

In interactive mode the command MUST prompt exactly once, for the PIN (create_certs.rs:44-46).
The prompt masks input and skips confirmation (prompts.rs:83-85).
The prompt appears after device selection and slot narrowing succeed (create_certs.rs:40-46).

Device selection MUST NOT prompt.
With several connected devices and no `--serial`, the command MUST refuse and list the serials (yubikey.rs:107-112).
Rationale: a serial-only prompt cannot identify which physical stick the user will touch (yubikey.rs:82-83).

After the PIN prompt the device blinks for one touch per certificate build, two in total.
The slots carry PIN policy `Always` and touch policy `Always`, so each build needs the PIN and a touch (yubikey.rs:183-189).

## Outputs (normative)

In human mode a successful run MUST print this block on stdout (create_certs.rs:100-114; test create_certs.rs:141-151):

```
YubiKey certificates created without modifying the device.

Serial:                    01c95c1f
Signing certificate:       tvc-yubikey-01c95c1f-signing.pem
Key-agreement certificate: tvc-yubikey-01c95c1f-key-agreement.pem
```

The JSON mode success outcome is `yubikey_certificates_created` (outcome.rs:61).
Its fields are `serial`, `signingCertificatePath`, and `keyAgreementCertificatePath` (test create_certs.rs:128-139).
No invocation reaches this outcome: JSON mode forces non-interactive mode and the command then refuses (Gap 3).
In JSON mode every invocation MUST fail with one NDJSON error object (INV-G3; V-4).

## Side effects (normative)

- The command discovers connected devices over PC/SC (output.rs:229-231, yubikey.rs:61-69). It opens the chosen device by serial (yubikey.rs:117-123).
- It reads PIV slot metadata for slots 9c and 9d and narrows each into a `CertificateSlot` (yubikey.rs:162-211).
- It submits PIN verification once per certificate (yubikey.rs:243-249). Each wrong PIN entry burns one device retry (yubikey.rs:462-463).
- It runs two on-device signing operations, one device touch each (yubikey.rs:251-265).
- It verifies each certificate signature against the slot metadata public key before acceptance (yubikey.rs:267-275; INV-4).
- The command MUST NOT write to the device (INV-3).
- A successful run MUST write exactly two files to the current working directory (create_certs.rs:57-75): `tvc-yubikey-<serial>-signing.pem`, then `tvc-yubikey-<serial>-key-agreement.pem`.
- The writes silently overwrite existing files (create_certs.rs:60-75; Gap 2).
- The command builds both certificates in memory before the first write (create_certs.rs:48-59). A failure before the writes leaves no files. A failed second write leaves only the signing PEM.
- The command MUST NOT read, write, or create the tvc config (INV-2). It makes no Turnkey API call and no network request.

## Failure modes (normative)

Every runtime failure exits with code 1 (INV-G2).
`classify` recognizes no typed error from this command (errors.rs:93-103).
In JSON mode every runtime error therefore MUST carry `reason` = `command_error` and `code` = `command_error` (output.rs:315, 323-342).
The refusal is a plain `bail!`, so even it carries `command_error` and never `missing_required_input` (create_certs.rs:34; output.rs:326-333).
Argument parse failures exit with code 2 through clap (Part 00, Exit codes).

| Failure | Trigger | Observation | Evidence |
|---|---|---|---|
| Interactive refusal | non-interactive mode, JSON mode, or stdin without a TTY | `creating YubiKey certificates is interactive: ...`; exit 1 | create_certs.rs:33-38 |
| No device | zero connected devices, no `--serial` | `no YubiKey is connected`; exit 1 | yubikey.rs:105; test yubikey.rs:806-812 |
| Several devices | more than one connected device, no `--serial` | `multiple YubiKeys are connected (serials <list>); unplug all but the one to use and try again, or pass --serial`; exit 1 | yubikey.rs:107-112; test yubikey.rs:813-824 |
| Unknown serial | `--serial` names an unconnected device | `YubiKey <serial> is not connected; connected: <list>`; the list part drops when nothing connects; exit 1 | yubikey.rs:94-103; tests yubikey.rs:778-798 |
| Open failure | PC/SC open fails after discovery (device unplugged in between) | `no connected YubiKey has serial <serial>` or `failed to open YubiKey <serial>`; exit 1 | yubikey.rs:117-123, 389-396 |
| Slot unsuitable | wrong algorithm, missing or wrong PIN and touch policy, missing or imported key origin, missing or malformed public key | one of `UnexpectedSlotAlgorithm`, `MissingSlotPolicy`, `UnexpectedSlotPolicy`, `MissingSlotOrigin`, `UnexpectedSlotOrigin`, `MissingSlotPublicKey`, `MalformedSlotPublicKey`; exit 1 | yubikey.rs:162-211; messages yubikey.rs:409-435; tests yubikey.rs:668-754 |
| Wrong PIN | the device rejects the entered PIN | `the YubiKey PIN was rejected; <tries> attempts remain before the PIN locks`; exit 1; one device retry burned | yubikey.rs:243-249, 462-463 |
| Missed touch | touch timeout during a certificate build | `failed to build the <slot> certificate` with the raw PIV source and no missed-touch hint (Gap 4); exit 1 | yubikey.rs:254-265, 484-489 |
| Signature mismatch | the built certificate signature fails verification against the metadata key | `the <slot> certificate signature did not verify against its metadata public key`; exit 1 | yubikey.rs:267-275, 490-495 |
| PEM encode or write failure | encoding or filesystem error | contexted error naming the certificate or file; exit 1 | create_certs.rs:50-75 |
| Usage error | bad `--serial` hex, unknown flag | clap error; exit 2 | config/turnkey/yubikey.rs:48-60; test cli.rs:567-575 |

## Test vectors (normative)

Vector comparison excludes the Part 00 global nondeterministic fields.
It also excludes the PEM file bytes.
The certificate serial is random (yubikey.rs:228-234) and the validity window starts at build time (yubikey.rs:238).
Hardware vectors (V-1, V-2, V-11, V-12) need a physically connected device and manual touches; the cited unit tests pin the logic without hardware.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | One suitable device, serial `01c95c1f`; TTY stdin; empty working directory; correct PIN; two touches | `tvc yubikey create-certs` | The Outputs human block with serial `01c95c1f`; `tvc-yubikey-01c95c1f-signing.pem` and `tvc-yubikey-01c95c1f-key-agreement.pem` exist; exit 0 (create_certs.rs:141-151 pins the block). |
| V-2 | Two suitable devices, serials `01c95c1f` and `0a1b2c3d`; TTY stdin; correct PIN; two touches on `01c95c1f` | `tvc yubikey create-certs --serial 01c95c1f` | Same as V-1: choose returns the explicit serial (yubikey.rs:94; test yubikey.rs:769-777). |
| V-3 | Any device state; TTY stdin | `tvc yubikey create-certs --non-interactive` | stderr: `error: creating YubiKey certificates is interactive: the PIN is prompted and the device must be touched once for each certificate`; exit 1 (create_certs.rs:33-38; output.rs:161-174). |
| V-4 | Any device state | `tvc yubikey create-certs --message-format json` | One NDJSON object on stdout: `reason` = `command_error`, `code` = `command_error`, `message` contains the refusal text; exit 1 (output.rs:210, 323-342; behavioral test output.rs:536-541). |
| V-5 | `TVC_NON_INTERACTIVE=true` in the environment; TTY stdin | `tvc yubikey create-certs` | Same refusal and exit 1 as V-3 (cli.rs:72-79; create_certs.rs:33). |
| V-6 | stdin redirected from `/dev/null` | `tvc yubikey create-certs < /dev/null` | Same refusal and exit 1 as V-3 (prompts.rs:17-19; create_certs.rs:33). |
| V-7 | Zero connected devices; TTY stdin | `tvc yubikey create-certs` | stderr: `error: no YubiKey is connected`; exit 1 (yubikey.rs:105; test yubikey.rs:806-812). |
| V-8 | Devices `01c95c1f` and `0a1b2c3d` connected; TTY stdin | `tvc yubikey create-certs` | stderr: `error: multiple YubiKeys are connected (serials 01c95c1f, 0a1b2c3d); unplug all but the one to use and try again, or pass --serial`; exit 1 (yubikey.rs:107-112; test yubikey.rs:813-824). |
| V-9 | Only `01c95c1f` connected; TTY stdin | `tvc yubikey create-certs --serial deadbeef` | stderr: `error: YubiKey deadbeef is not connected; connected: 01c95c1f`; exit 1 (yubikey.rs:94-103; test yubikey.rs:778-789). |
| V-10 | Any device state | `tvc yubikey create-certs --serial not-hex` | clap value validation failure quoting `must be bare hex encoded`; exit 2 (config/turnkey/yubikey.rs:53-55; test cli.rs:567-575). |
| V-11 | Device `01c95c1f` whose 9c slot key origin is `Imported`; TTY stdin | `tvc yubikey create-certs` | stderr: `error: the signing slot key is Imported; expected a key generated on the YubiKey`; exit 1 (yubikey.rs:195-197, 426-427; test yubikey.rs:708-723). |
| V-12 | Suitable device `01c95c1f`; TTY stdin; wrong PIN entered with three device retries left | `tvc yubikey create-certs` | stderr: `error: the YubiKey PIN was rejected; 2 attempts remain before the PIN locks`; exit 1; the device retry counter drops by one (yubikey.rs:243-249, 462-463; hardware test yubikey.rs:1128-1140). |

## Invariants (normative)

Part 00's global invariants apply; INV-G4 names this command as its sole exception (INV-2).

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT proceed in non-interactive mode or without a TTY on stdin. | The first statement of `Args::run` bails before any device access (create_certs.rs:33-38). JSON mode inherits the refusal through INV-G1 (output.rs:210). |
| INV-2 | The command MUST run without reading the tvc config and MUST NOT create the tvc config file (the INV-G4 exception). | `Commands::run` returns early for `Yubikey/CreateCerts` (cli.rs:205-213), ahead of the `HOME` lookup (cli.rs:215) and the load-or-create block (cli.rs:219-240). The signature `Args::run(ctx)` (create_certs.rs:32) omits the `Run` trait's config parameter (commands.rs:26-34). Runtime tripwire: the late match arm is `unreachable!` (cli.rs:313-315). No test pins the dispatch (Gap 7). |
| INV-3 | The command MUST NOT modify the device. | `create_certificate` submits only PIN verification and signing APDUs (yubikey.rs:213-278). The module never authenticates a management key and never writes device objects (yubikey.rs:13-15). |
| INV-4 | The command MUST verify each certificate signature against the device metadata public key before accepting it. | `CertificateSlot` retains the metadata-parsed `VerifyingKey` (yubikey.rs:154-160); `create_certificate` verifies after build (yubikey.rs:267-275). |
| INV-5 | The PIN MUST reach the command only through the masked prompt, and MUST NOT persist. | No PIN flag, environment variable, or tvc config field exists; the sole source is `prompts::password` (create_certs.rs:44-46). `Pin` wraps a `Zeroizing` buffer that clears on drop (yubikey.rs:367-375). |

## Gaps (informative)

1. **[capability] Hard-coded output paths.** The two PEM paths are format-string literals (create_certs.rs:57-59); the only control over the destination is `cd`. No output flag or environment variable exists. Sibling file-writing commands expose the destination. `keys backup-operator-key` takes `--output` and `TVC_OPERATOR_KEY_BACKUP_OUT` (commands/keys/backup_operator_key.rs:29-33). `keys generate-local-quorum-key` takes `TVC_QUORUM_KEY_METADATA_OUT` (commands/keys/generate_local_quorum_key.rs:28-33). `keys init-local-quorum-key`, `app init`, and `deploy init` all take an output path. An output flag would align the command with its siblings.

2. **[consistency] Silent overwrite of existing PEM files.** The command calls `tokio::fs::write` unconditionally (create_certs.rs:60-75). Every other file-writing command gates or refuses overwrite. `backup-operator-key` needs `--overwrite` or a confirmation (commands/keys/backup_operator_key.rs:92-97, 135-139). `generate-local-quorum-key` bails on an existing metadata file (commands/keys/generate_local_quorum_key.rs:50-54). `init-local-quorum-key`, `app init`, and `deploy init` bail on existing output. Evidence: commands/keys/init_local_quorum_key.rs:30-31, commands/app/init.rs:47-49, commands/deploy/init.rs:76-78. The stakes here are lower than key material. A re-run against the wrong device can still clobber a certificate the user just spent two touches on.

3. **[consistency] The JSON success outcome `yubikey_certificates_created` is unreachable.** JSON mode forces non-interactive mode (output.rs:209-215). The command refuses every non-interactive invocation (create_certs.rs:33-38). No real invocation can emit the outcome that outcome.rs:61 registers and create_certs.rs:128-139 tests. The PIN policy makes this deliberate. Still, scripts and agents can never drive this command, and the help text nowhere says so (create_certs.rs:20-23).

4. **[consistency] A missed touch loses its hint.** `DeviceError::Sign` and `DeviceError::KeyAgreement` explain that a missed touch while the device blinks times out (yubikey.rs:501-508). The certificate build path wraps the same class of timeout as `BuildCertificate`, with only the raw PIV source (yubikey.rs:254-265, 484-489). The one place that explains the touch is the PIN prompt text, and that text has scrolled away when the timeout hits.

5. **[capability] Several connected devices hard-refuse with no interactive selection.** `ConnectedYubiKeys::choose` refuses several connected devices even in interactive mode (yubikey.rs:107-112). Sibling `operator create` prompts a selection among registered serials (commands/operator/create.rs:303-314). The refusal has a documented rationale: a serial-only prompt cannot identify the physical stick the user will touch (yubikey.rs:82-83). `--serial` is the escape hatch, so this looks like a deliberate tradeoff. The entry stays because it has the shape of a hard-coded choice that a sibling lets the user make.

6. **[docs] The help text omits the command's operational contract.** The about line reads `Create importable certificates for keys already generated in slots 9c and 9d`, with `long_about = None` (create_certs.rs:20-23). It says nothing about the PIN prompt, the two touches, the interactive-only refusal, or JSON mode always failing. It also skips the output location and the silent overwrite. The success output never names the follow-up step: import the PEMs with `ykman`. Device errors elsewhere reference exactly that remediation (yubikey.rs:438-449).

7. **[docs] Only a runtime `unreachable!` enforces the pre-config dispatch contract.** The cli.rs tests cover parsing only (cli.rs:554-575). Nothing pins that create-certs runs without loading or creating the tvc config (cli.rs:205-213). A refactor of `Commands::run` could silently reintroduce the config dependency. The `unreachable!` tripwire (cli.rs:313-315) would only fire when someone runs the binary.
