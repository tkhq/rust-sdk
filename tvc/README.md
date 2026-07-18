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

Some commands support environment variables and command-line flags for
programmatic use. Run command help, such as `tvc deploy create --help`, to see
the supported inputs and precedence for that command.

## Configuration

### `--api-base-url`

The `--api-base-url` flag (also settable as `TVC_API_BASE_URL`) overrides which
Turnkey API endpoint `tvc` talks to. It defaults to `https://api.turnkey.com`.

You typically only need this when pointing `tvc` at a non-production Turnkey
environment — for example a staging cluster or a local API for development.
The flag is available on `tvc login`; for newly configured orgs the value is
stored alongside the org so subsequent commands hit the same environment, and
for existing orgs it updates the stored value on that login.

For programmatic use (CI, scripts), `TVC_API_BASE_URL` is also read directly by
env-var auth alongside `TVC_ORG_ID` / `TVC_API_KEY_PUBLIC` / `TVC_API_KEY_PRIVATE`.

```bash
# One-off: log in against a staging environment
tvc login --api-base-url https://api.staging.example.turnkey.com

# CI: authenticate directly with env vars, including a non-prod base URL
TVC_ORG_ID=... TVC_API_KEY_PUBLIC=... TVC_API_KEY_PRIVATE=... \
  TVC_API_BASE_URL=https://api.staging.example.turnkey.com \
  tvc app list
```

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
