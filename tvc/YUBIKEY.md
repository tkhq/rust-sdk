# Use a YubiKey with `tvc`

TVC uses two keys in the YubiKey PIV application:

| Slot | Purpose | Key requirements |
|---|---|---|
| `9c` | Sign deployment approvals | P-256, generated on the device, PIN Always, touch Always |
| `9d` | Decrypt and re-encrypt quorum-key shares | P-256, generated on the device, PIN Always, touch Always |

The private keys never leave the YubiKey. TVC never generates, imports, or
deletes device keys or certificates and never authenticates with the PIV
management key. Every management-key-protected PIV operation is an explicit
`ykman` command.

The examples below assume exactly one YubiKey is connected. When several are
connected, unplug the others or pass TVC's canonical hexadecimal serial with
`--serial`.

## Prerequisites

- YubiKey firmware 5.3 or later. Run `ykman info` to see the firmware version.
- [YubiKey Manager CLI (`ykman`)](https://docs.yubico.com/software/yubikey/tools/ykman/).
- A current `tvc` installation.
- An active TVC login for the target organization before step 6.

Firmware 5.3 or later is required because TVC retrieves the generated public
keys and their policies from PIV metadata after key generation.

## 1. Harden PIV access

This step is optional for an already-hardened device, but recommended before
generating operator keys. Each command prompts for the current and replacement
credential; do not put credentials in command-line flags or shell history.

```sh
ykman piv access change-pin
ykman piv access change-puk
ykman piv access change-management-key --generate --protect
```

`--protect` stores the generated management key on the YubiKey protected by
the PIN. Later `ykman` administrative operations can then authorize through a
PIN prompt. TVC never reads or uses this key.

## 2. Generate the operator keys

Generate both private keys on the YubiKey with the policies TVC requires:

```sh
ykman piv keys generate \
  --algorithm eccp256 \
  --pin-policy always \
  --touch-policy always \
  9c - >/dev/null

ykman piv keys generate \
  --algorithm eccp256 \
  --pin-policy always \
  --touch-policy always \
  9d - >/dev/null
```

The public-key output can be discarded on firmware 5.3 or later because
`tvc yubikey create-certs` reads it back from device metadata. On a shell
without `/dev/null`, write each public key to a temporary file and keep it
until certificate creation succeeds.

Key algorithm, origin, PIN policy, and touch policy cannot be corrected by
certificate creation. If TVC rejects a slot's metadata, fix the slot with
`ykman` before continuing. Recreating a slot replaces its private key, so do
not regenerate a key that is already in use.

## 3. Create the certificates

```sh
tvc yubikey create-certs
```

The command prompts for the PIV PIN and requires two touches, one per slot. It
then:

1. Reads the public key and metadata from `9c` and `9d`.
2. Requires P-256, generated origin, PIN Always, and touch Always.
3. Builds a self-signed certificate with subject and issuer `CN=QuorumOS` for
   each slot.
4. Verifies the PIN immediately before each slot signs its certificate.
5. Verifies each finished certificate against the public key read from that
   slot's metadata.
6. Writes both PEM files in the current directory:

   - `tvc-yubikey-<serial>-signing.pem`
   - `tvc-yubikey-<serial>-key-agreement.pem`

Both certificates are built, encoded, and verified before either file is
written. This command does not change the YubiKey and does not use a management
key.

## 4. Import the certificates

Use the exact filenames printed by `create-certs`:

```sh
ykman piv certificates import \
  --verify \
  9c tvc-yubikey-<serial>-signing.pem

ykman piv certificates import \
  --verify \
  9d tvc-yubikey-<serial>-key-agreement.pem
```

`--verify` proves that each certificate matches the private key in its slot.
Certificate import is an administrative device write, so `ykman` authorizes
it with the protected management key or the corresponding management-key
prompt.

The PEM files contain public certificates, not private key material. Keep them
until registration succeeds; they may also be retained for auditing.

## 5. Register the YubiKey with TVC

The current registry command reads the installed certificates and caches the
device's composite operator public key in the TVC config. It needs neither the
PIN nor a touch:

```sh
tvc keys refresh-yubikey --serial <serial>
```

You can omit `--serial` when exactly one YubiKey is connected. The command
prints the canonical hexadecimal serial used by subsequent TVC commands.

## 6. Add the operator to an organization

With the target organization active in TVC, reference the registered serial:

```sh
tvc operator create \
  --kind yubikey \
  --serial <serial> \
  --default
```

Omit `--default` if the YubiKey should not become the organization's default
operator kind. Because the serial is already registered, this step uses the
cached public key and does not modify the device.

## Use the operator

Approve a deployment with the `9c` signing key:

```sh
tvc deploy approve \
  --deploy-id <deployment-uuid> \
  --serial <serial>
```

Re-encrypt a local quorum-key share with the `9d` key-agreement key:

```sh
tvc keys re-encrypt-local-share \
  --quorum-key-metadata <quorum-key-metadata.json> \
  --serial <serial>
```

Private-key operations prompt for the PIV PIN and require a physical touch.
When the organization has only one YubiKey operator, TVC can usually select it
without `--serial`.

## Unregister the YubiKey locally

After removing the YubiKey operator from every local organization profile that
references it, remove its registry entry with:

```sh
tvc yubikey unregister --serial <serial>
```

This command only changes the local TVC configuration. It does not open or
modify the YubiKey, erase its keys or certificates, or revoke the operator from
any organization. The YubiKey remains able to act as an operator for every
organization that still trusts its public keys.

The command refuses to unregister a serial while a local organization profile
still references it. In non-interactive mode, pass both `--serial` and `--yes`.

## Troubleshooting

| Error | What to check |
|---|---|
| No YubiKey is connected | Reinsert the device and confirm that `ykman info` can see it. |
| Multiple YubiKeys are connected | Unplug the others or pass `--serial`. |
| Metadata is unavailable | Confirm firmware 5.3 or later. |
| Unexpected algorithm, origin, or policy | If the key is not already in use, recreate the affected slot with the exact `ykman piv keys generate` options above. |
| PIN rejected | Stop before exhausting the retry counter; recover or unblock the PIN with the PUK. |
| Signing timed out | Retry and touch the device while it blinks. |
| Certificate import verification failed | Confirm the certificate filename and slot match (`signing` to `9c`, `key-agreement` to `9d`). |

Run `tvc yubikey create-certs --help` or the relevant `ykman ... --help`
command for the complete option list.
