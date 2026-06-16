# `tvc`

CLI for [Turnkey Verifiable Cloud](https://turnkey.com) - see [this guide](https://docs.turnkey.com/getting-started/verifiable-cloud-quickstart) for example usage.

## Installation

```bash
cargo install tvc
```

## Authentication

For **local use**, run `tvc login` once and the CLI will read `~/.config/turnkey/` thereafter.

For **programmatic use** (GitHub Actions, etc.), set these three env vars to authenticate directly without touching disk:

| Env | Source |
|---|---|
| `TVC_ORG_ID` | your Turnkey organization UUID |
| `TVC_API_KEY_PUBLIC` | hex-encoded compressed P256 public key |
| `TVC_API_KEY_PRIVATE` | hex-encoded P256 private key |

When all three required vars are present, every command authenticates directly from env. Env vars take precedence over local config files. Setting some but not all is rejected.

The typical flow: run `tvc login` once locally to generate an API key, register the public key in the Turnkey dashboard, then store the values in your CI's secret store (e.g. `TVC_API_KEY_PRIVATE` as a GitHub Secret, the rest as GitHub Variables).

### Passkeys and WebAuthn

TVC includes the backend-compatible WebAuthn stamping and session plumbing used by Turnkey passkeys:

```bash
tvc login --passkey --passkey-transport auto
tvc auth passkey register --name "Laptop Touch ID"
tvc auth passkey list
tvc auth passkey remove <AUTHENTICATOR_ID>
```

Turnkey verifies WebAuthn assertions against the exact JSON request body. The challenge is `base64url(hex(sha256(raw_request_body)))`, and the CLI sends the assertion as protojson in `X-Stamp-WebAuthn`. After passkey login returns a Turnkey session, later commands can use the stored session via `X-Session` without requiring another touch.

Transport status:

| Transport | Intended support | Current build behavior |
|---|---|---|
| `usb` | Roaming CTAP2/FIDO2 keys such as YubiKeys | Command surface and errors are present; native CTAP2 provider still needs to be enabled |
| `browser` | Browser WebAuthn for Mac Touch ID, synced platform passkeys, and 1Password | Command surface and errors are present; browser handoff still needs to be enabled |
| `cross-device` | Hybrid/QR passkey flow | Command surface and errors are present; browser handoff still needs to be enabled |
| `auto` | Prefer browser-capable flow, fall back to USB | Available once at least one real provider is enabled |

Manual validation checklist for the real provider follow-up:

| Device | Flow |
|---|---|
| YubiKey 5/5C/NFC | `tvc login --passkey --passkey-transport usb`, then run `tvc app list` without another touch |
| Mac Touch ID | `tvc login --passkey --passkey-transport browser`, complete the local browser ceremony, then run `tvc app list` |
| 1Password | `tvc login --passkey --passkey-transport browser`, select 1Password in the browser passkey picker, then run `tvc app list` |

Some commands support environment variables and command-line flags for
programmatic use. Run command help, such as `tvc deploy create --help`, to see
the supported inputs and precedence for that command.

## Usage

### Create an App

```bash
# Login to Turnkey
tvc login

# Generate app config template
tvc app init --name my-app --output my-app.json

# Edit my-app.json to fill in required values (quorumPublicKey, operator keys, etc.)

# Create the app
tvc app create --config-file my-app.json
```

### Create and Approve a Deployment

```bash
# Generate deployment config template
tvc deploy init --output my-deploy.json

# Edit my-deploy.json to fill in required values (appId, container images, etc.)

# Create the deployment
tvc deploy create --config-file my-deploy.json

# To create a deployment without a config file, see:
tvc deploy create --help

# Recommended: uses GetTvcDeployment to fetch manifest and manifest_id automatically
tvc deploy approve \
  --deploy-id <DEPLOYMENT_UUID> \
  --operator-id <OPERATOR_UUID> # Turnkey's ID for your operator (from app create response)

# Alternative: provide manifest file and IDs manually
tvc deploy approve \
  --manifest manifest.json \
  --manifest-id <MANIFEST_UUID> \  # Turnkey's ID for the manifest (from deploy create response)
  --operator-id <OPERATOR_UUID>

# Save a single provisioning bundle JSON for later re-encryption / submission
# flows.
tvc deploy provisioning-details \
  --deploy-id <DEPLOYMENT_UUID> \
  --provision-bundle-out provisioning-bundle.json
```
