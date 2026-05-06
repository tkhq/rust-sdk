# `tvc`

CLI for [Turnkey Verifiable Cloud](https://turnkey.com) - see [this guide](https://docs.turnkey.com/getting-started/verifiable-cloud-quickstart) for example usage.

## Installation

```bash
cargo install tvc
```

## Configuration precedence

Configuration values are resolved in this order, highest priority first:

1. Command-line flag (e.g. `--app-id`)
2. Environment variable (e.g. `TVC_APP_ID`)
3. Config file value (`--config-file`)
4. Built-in default

Special rules:

- `--pivot-args` replaces the config file's list entirely (does not append).

`tvc deploy create` accepts `--config-file` *or* the equivalent flags (`--app-id`, `--qos-version`, `--pivot-image-url`, `--pivot-path`, `--expected-pivot-digest`, plus optional fields). `tvc app create` and `tvc keys generate-quorum-key` require `--config-file` because their configs include nested arrays.

## Authentication

For **local use**, run `tvc login` once and the CLI will read `~/.config/turnkey/` thereafter.

For **programmatic use** (GitHub Actions, etc.), set these four env vars to authenticate directly without touching disk:

| Env | Source |
|---|---|
| `TVC_ORG_ID` | your Turnkey organization UUID |
| `TVC_API_BASE_URL` | e.g. `https://api.turnkey.com` |
| `TVC_API_KEY_PUBLIC` | hex-encoded compressed P256 public key |
| `TVC_API_KEY_PRIVATE` | hex-encoded P256 private key |

When all four are present, every command authenticates directly from env. Setting some but not all is rejected.

The typical flow: run `tvc login` once locally to generate an API key, register the public key in the Turnkey dashboard, then store the values in your CI's secret store (e.g. `TVC_API_KEY_PRIVATE` as a GitHub Secret, the rest as GitHub Variables).

### Example: CI deploy with no config file

```bash
TVC_ORG_ID=...                      \
TVC_API_BASE_URL=...                \
TVC_API_KEY_PUBLIC=...              \
TVC_API_KEY_PRIVATE=...             \
TVC_APP_ID=...                      \
TVC_QOS_VERSION=...                 \
TVC_PIVOT_PATH=...                  \
TVC_PIVOT_IMAGE_URL=...             \
TVC_EXPECTED_PIVOT_DIGEST=...       \
  tvc deploy create
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
