# YubiKey

Users keep operator private keys on a hardware device, register its public
identity, refresh local metadata, and remove unused local registration.

## Sub-features

- `yubikey.certificates` writes local PEM certificates for existing device keys.
- `yubikey.operator` registers a hardware-backed operator.
- `yubikey.refresh` refreshes local device information.
- `yubikey.unregister` removes an unused local registration.

## How to get to it (user POV)

Use `tvc yubikey create-certs`, `tvc operator create --kind yubikey`,
`tvc keys refresh-yubikey`, and `tvc yubikey unregister`. Follow the repository's
[YubiKey setup guide](../../../../YUBIKEY.md) for `ykman` setup before TVC use.

## Driving it with verify.sh

Preconditions: hardware verification needs a designated test device,
existing PIV keys, and PIN access. API
registration also needs a test organization. Empty-home smoke has no device.

- Run `tvc/.agents/skills/verify/scripts/verify.sh test keys_yubikey` for CLI
  validation and rejection paths. Hardware fakes do not prove touch or PIN entry.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test operator_create` and
  `tvc/.agents/skills/verify/scripts/verify.sh test pty` for backend selection and
  prompt behavior at the existing boundaries.
- On the designated device, run `tvc yubikey create-certs --serial "$serial"`
  in a disposable working directory. Inspect the generated PEM certificates
  and compare their public keys with the device. Do not record PINs.
- Register through `tvc operator create --kind yubikey` in a PTY. Then run
  `tvc keys refresh-yubikey` and compare the saved public identity with the device.
- Exercise a mapped deployment approval or local-share re-encryption using
  `--serial` for the registered device. Observe PIN entry and touch as
  required, then inspect the resulting signature or share artifact.
- For a run-created unused registration, inspect `tvc yubikey unregister --help`
  and remove that serial. Read local registration again and confirm that device
  certificates remain present. Referenced operators must prevent unregistering.

## Gotchas

- `create-certs` bypasses TVC config loading and writes certificates locally
  without modifying the device. It still uses device signing. An isolated home
  does not isolate hardware access. Do not double-drive the same YubiKey.
- Operator PINs require interactive entry. JSON mode cannot replace that prompt.
- Unregistering removes local registration; it neither erases device keys nor
  revokes remote authority. Config removal alone is not proof of revocation.
