# tvc yubikey create-certs

## Purpose
Rebuilds the two self-signed `CN=QuorumOS` certificates for P-256 keys already
generated in a YubiKey's PIV signing (9c) and key-agreement (9d) slots, and writes
them as PEM files for the user to import with `ykman`. Run it when a device has the
QuorumOS keys but lost (or never had) the matching slot certificates. Strictly
device-local: it never modifies the device (no management-key auth, no object
writes — only PIN-verify and signing APDUs; tvc/src/yubikey.rs:217-219) and never
touches the TVC config, Turnkey API, or network.

**Dispatch contract (verified):** `cli.rs` special-cases this command to run *before*
the TVC config file is loaded or created. `Commands::run` returns early for
`Yubikey/CreateCerts` (tvc/src/cli.rs:206-213) ahead of the `HOME` lookup
(cli.rs:215) and the load-or-create-default-config block (cli.rs:219-240), so the
command works with `HOME` unset, with a corrupt config file, and never creates
`~/.config/turnkey/config.toml` as a side effect. The later match arm is an
`unreachable!` tripwire (cli.rs:313-315). This is the only command with a
config-free signature (`Args::run(ctx)` — create_certs.rs:32 — vs the `Run` trait's
`run(ctx, config)`, commands/mod.rs:26-33).

## Inputs
| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| device serial | `--serial <SERIAL>` (bare hex, ≤32 bits; config/turnkey/yubikey.rs:45-61) | — | — | the sole connected device | no — 0 or >1 connected devices without `--serial` is a hard refusal (yubikey.rs:104-112) |
| PIV PIN | — | — | — | — | yes, always (masked, no confirmation; create_certs.rs:44-46, prompts.rs:83-85) |
| non-interactive | `--non-interactive` (global) | `TVC_NON_INTERACTIVE` | — | false | n/a — any non-interactive means the whole command refuses (create_certs.rs:33-38) |
| message format | `--message-format` (global) | — | — | human | `json` implies non-interactive (output.rs:209-215), so the command refuses |

Deviations from flag > env > config > default: the serial is flag-only (no env, no
config lookup — the command runs before config exists by design), and the PIN is
prompt-only, matching the repo-wide policy that a YubiKey PIN is never read from
config or the environment (deploy/approve.rs:407-412,
keys/re_encrypt_local_share.rs:164-169).

Hard-coded, not selectable: both slots always (signing then key-agreement),
certificate subject `CN=QuorumOS` (yubikey.rs:52), validity ~10 years from now
(yubikey.rs:54-56), output file names/location (see Side effects).

## Interactive behavior
Interactive-only. First statement: if `ctx.is_non_interactive()` or stdin is not a
TTY, bail with "creating YubiKey certificates is interactive: the PIN is prompted
and the device must be touched once for each certificate" (create_certs.rs:33-38).
There is no non-interactive escape hatch, deliberately (PIN policy above).

In interactive mode the single prompt is the masked PIN prompt ("YubiKey PIV PIN
(touch the device once for each certificate)"), after device selection and slot
metadata narrowing succeed. The device then blinks for a physical touch twice —
once per certificate build (PIN policy and touch policy on the slots must both be
"Always"; yubikey.rs:183-189). Device selection itself never prompts: multiple
connected devices are refused with instructions to unplug or pass `--serial`
(yubikey.rs:107-112), with the documented rationale that a serial-only prompt
cannot identify which physical stick will be touched (yubikey.rs:82-83).

## Outputs
Human mode (Display, create_certs.rs:100-114):

```
YubiKey certificates created without modifying the device.

Serial:                    <8-hex-digit serial>
Signing certificate:       tvc-yubikey-<serial>-signing.pem
Key-agreement certificate: tvc-yubikey-<serial>-key-agreement.pem
```

JSON mode: outcome `reason: "yubikey_certificates_created"` with `serial`,
`signingCertificatePath`, `keyAgreementCertificatePath` (outcome.rs:61,
create_certs.rs:128-139) — but this is unreachable in practice, because JSON mode
forces non-interactive and the command then refuses (see Gaps #3). JSON consumers
only ever see a `command_error` line.

## Side effects
- PC/SC device discovery via `Ctx::connected_yubikeys` → `connected_serials()`
  (output.rs:229-231, yubikey.rs:61-69), then opens the device by serial
  (yubikey.rs:117-123).
- Reads PIV slot metadata for 9c and 9d and narrows it (P-256, PIN/touch policy
  Always/Always, key origin Generated, public key present; yubikey.rs:162-211).
- Submits PIN verification (a wrong PIN decrements the device's retry counter;
  yubikey.rs:243-249) and two on-device signing operations, each needing a touch.
  Each certificate's signature is verified against the metadata public key before
  acceptance (yubikey.rs:267-275). Never writes to the device.
- Writes exactly two files to the **current working directory**:
  `tvc-yubikey-<serial>-signing.pem` and `tvc-yubikey-<serial>-key-agreement.pem`
  (create_certs.rs:57-75), overwriting silently if they exist. Certificates are
  built fully in memory first, so a failure before the writes leaves no files; a
  failure on the second write can leave just the signing PEM.
- No config file read or write, no Turnkey API calls, no network.

## Failure modes
All failures exit 1; none of the paths produce a typed error that `classify`
recognizes (errors.rs:93-103 downcasts only `MissingResource` /
`TurnkeyClientError`), so every JSON error from this command carries
`code: "command_error"` — including the non-interactive refusal (it is a plain
`bail!`, not `MissingRequiredInput`) and "no YubiKey is connected". Usage errors
(bad `--serial` hex, unknown flags) are clap failures, exit 2.

Notable paths:
- Non-interactive / non-TTY stdin: refusal (create_certs.rs:33-38).
- No device, several devices, or an explicit serial not connected: refusal listing
  connected serials (yubikey.rs:93-113; unit-tested yubikey.rs:769-824).
- Slot unsuitable: `UnexpectedSlotAlgorithm`, `MissingSlotPolicy`/`UnexpectedSlotPolicy`
  (must be Always/Always), `MissingSlotOrigin`/`UnexpectedSlotOrigin` (must be
  Generated — imported keys refused), `MissingSlotPublicKey`, `MalformedSlotPublicKey`
  (yubikey.rs:162-211; unit-tested yubikey.rs:668-754).
- Wrong PIN: `WrongPin { tries }` reports remaining attempts (yubikey.rs:243-249,
  462-463); each wrong attempt burns a device retry.
- Missed touch during a certificate build: surfaces as `BuildCertificate` with the
  raw PIV source — without the missed-touch hint (see Gaps #4).
- Post-build signature mismatch: `InvalidCertificateSignature` (yubikey.rs:273-275).
- PEM encode / file write failures with per-file context (create_certs.rs:50-75).

## Gaps

1. **[capability] Output paths are hard-coded to fixed names in the CWD — no
`--out`/`-o` flag, no env var.** Sibling file-writing commands all expose the
destination: `keys backup-operator-key` has `--out` + `TVC_OPERATOR_KEY_BACKUP_OUT`
(backup_operator_key.rs:31), `keys generate-local-quorum-key` has
`TVC_QUORUM_KEY_METADATA_OUT` (generate_local_quorum_key.rs:31), `keys
init-local-quorum-key`, `app init`, `deploy init` likewise. Here the two PEM paths
are format-string literals (create_certs.rs:57-59); the only control is `cd`.

2. **[consistency] Existing PEM files are silently overwritten.** Every other
file-writing command gates or refuses overwrite: `backup-operator-key` requires
`--overwrite` or a prompt (backup_operator_key.rs:93-96, 137),
`generate-local-quorum-key` bails on an existing metadata file
(generate_local_quorum_key.rs:50-53), `init-local-quorum-key`/`app init`/`deploy
init` bail on existing output (init_local_quorum_key.rs:30-31, app/init.rs:47-49,
deploy/init.rs:76-78). `create-certs` calls `tokio::fs::write` unconditionally
(create_certs.rs:60-75). Lower stakes than key material, but a freshly signed
certificate the user just spent two touches on can be clobbered by a re-run
against the wrong device.

3. **[consistency] The JSON success outcome `yubikey_certificates_created` is
unreachable.** JSON mode forces non-interactive (output.rs:209-215) and the command
refuses whenever non-interactive (create_certs.rs:33-38), so no real invocation can
emit the outcome that outcome.rs:61 registers and create_certs.rs:128-139 tests.
Deliberate given the PIN policy, but worth stating: agents/scripts can never drive
this command, and the LONG_ABOUT/help nowhere says so.

4. **[consistency] A missed touch during certificate signing loses the
missed-touch hint.** `DeviceError::Sign` and `KeyAgreement` explain "a missed touch
while it blinks times out" (yubikey.rs:503-508), but the certificate-build path —
which also requires a touch per certificate — wraps the same class of timeout as
`BuildCertificate`/"failed to build the {slot} certificate" with only the raw PIV
error (yubikey.rs:484-489, 254-265). The one place the touch is explained is the
PIN prompt text, which has scrolled past by the time the timeout hits.

5. **[capability] Multiple connected devices hard-refuse instead of offering an
interactive selection.** `ConnectedYubiKeys::choose` bails with "unplug all but
the one to use ... or pass --serial" (yubikey.rs:107-112) even in interactive mode,
while sibling `operator create` prompts a `select` among registered serials
(operator/create.rs:304-314). The refusal has a documented rationale — a serial
prompt can't identify the physical stick (yubikey.rs:82-83) — and `--serial` is the
escape, so this is a deliberate tradeoff more than an oversight; flagged because it
is exactly the "hard-coded choice a sibling lets the user make" shape.

6. **[docs] Help text omits the command's operational contract.** The about line
("Create importable certificates for keys already generated in slots 9c and 9d",
create_certs.rs:21-22, with `long_about = None` at :23) says nothing about: the PIN
prompt and two touches, the interactive-only refusal (and JSON mode always
failing), where the PEMs are written, or the silent overwrite. The success output
also never names the follow-up step — importing the PEMs with `ykman` — even
though device errors elsewhere reference exactly that remediation
(yubikey.rs:439-442, 446-449).

7. **[docs] The pre-config dispatch contract is enforced only by a runtime
`unreachable!`, not by any test.** cli.rs tests cover parsing only
(cli.rs:554-573); nothing pins "create-certs runs without loading or creating the
config" (cli.rs:206-213), so a refactor of `Commands::run` could silently
reintroduce the config dependency until someone runs the binary and hits
cli.rs:313-315.
