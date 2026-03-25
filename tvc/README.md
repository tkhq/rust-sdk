# TVC CLI

Command-line interface for [Turnkey Verifiable Cloud](https://turnkey.com). Manages applications, deployments, and cryptographic manifest approvals.

## Installation

Build from the workspace root:

```bash
cargo build -p tvc --release
# Binary is at target/release/tvc
```

## Quick Start

```bash
# 1. Authenticate
tvc login

# 2. Create an app
tvc app init --output my-app.json
# Edit my-app.json to fill in required values
tvc app create my-app.json

# 3. Create and approve a deployment
tvc deploy init --output my-deploy.json
# Edit my-deploy.json to fill in required values
tvc deploy create my-deploy.json
tvc deploy approve --deploy-id <DEPLOYMENT_ID>

# 4. Check deployment status
tvc deploy status --deploy-id <DEPLOYMENT_ID>
```

## Global Flags

These flags are available on all commands:

| Flag | Env Var | Description |
|------|---------|-------------|
| `--json` | `TVC_JSON` | Output results as JSON to stdout |
| `--no-input` | `TVC_NO_INPUT` | Disable all interactive prompts |
| `--quiet` / `-q` | | Suppress non-essential output |
| `--api-key-file <PATH>` | `TVC_API_KEY_FILE` | Path to API key JSON file (overrides login config) |
| `--operator-key-file <PATH>` | `TVC_OPERATOR_KEY_FILE` | Path to operator key JSON file (overrides login config) |
| `--api-url <URL>` | `TVC_API_URL` | API base URL override |
| `--org-id <ID>` | `TVC_ORG_ID` | Organization ID override |

When `--api-key-file`, `--api-url`, and `--org-id` are all provided, commands that call the API work without running `tvc login` first.

## Commands

### `tvc login`

Authenticate with Turnkey and set up local credentials (API key and operator key).

```bash
# Interactive setup
tvc login

# Select an existing org
tvc login --org my-alias

# Non-interactive setup (for CI/CD)
tvc login --no-input --org-id <ORG_UUID> --alias prod --api-env prod --skip-api-key-wait
```

| Flag | Env Var | Description |
|------|---------|-------------|
| `--org <ALIAS_OR_ID>` | | Select an existing org by alias or ID |
| `--alias <NAME>` | `TVC_ORG_ALIAS` | Alias for the org config (default: "default") |
| `--api-env <ENV>` | `TVC_API_ENV` | API environment: `prod`, `preprod`, `dev`, `local` |
| `--skip-api-key-wait` | | Skip the "press Enter" prompt after API key generation |

### `tvc app init`

Generate a template app configuration file.

```bash
tvc app init --output my-app.json
```

| Flag | Default | Description |
|------|---------|-------------|
| `--output` / `-o` | `app.json` | Output file path |

### `tvc app create`

Create a new TVC application from a config file.

```bash
tvc app create my-app.json
```

Returns the app ID and manifest set operator IDs needed for deployment approval.

### `tvc deploy init`

Generate a template deployment configuration file.

```bash
tvc deploy init --output my-deploy.json
```

| Flag | Default | Description |
|------|---------|-------------|
| `--output` / `-o` | `deploy-<timestamp>.json` | Output file path |

### `tvc deploy create`

Create a new deployment from a config file.

```bash
tvc deploy create my-deploy.json

# With pull secret encryption
tvc deploy create my-deploy.json --pivot-pull-secret ./pull-secret.txt
```

| Flag | Description |
|------|-------------|
| `--pivot-pull-secret <PATH>` | Unencrypted pull secret file (encrypted automatically) |

### `tvc deploy approve`

Cryptographically approve a deployment manifest with your operator key.

```bash
# Fetch manifest from API (recommended)
tvc deploy approve --deploy-id <DEPLOYMENT_ID>

# Use a local manifest file
tvc deploy approve --manifest manifest.json --manifest-id <MANIFEST_UUID> --operator-id <OPERATOR_UUID>

# Non-interactive approval
tvc deploy approve --deploy-id <DEPLOYMENT_ID> --yes

# Dry run (review without generating approval)
tvc deploy approve --deploy-id <DEPLOYMENT_ID> --dry-run

# Generate approval without posting to API
tvc deploy approve --deploy-id <DEPLOYMENT_ID> --yes --skip-post
```

| Flag | Env Var | Description |
|------|---------|-------------|
| `--manifest` / `-m` | | Path to local manifest file |
| `--deploy-id` / `-d` | `TVC_DEPLOY_ID` | Deployment ID (fetches manifest from API) |
| `--manifest-id` | `TVC_MANIFEST_ID` | Manifest UUID (required for API posting with `--manifest`) |
| `--operator-id` | `TVC_OPERATOR_ID` | Operator UUID (required for API posting) |
| `--operator-seed <PATH>` | | Custom operator key file |
| `--yes` / `-y` | | Skip interactive approval prompts |
| `--dry-run` | | Review manifest without generating approval |
| `--skip-post` | | Don't post approval to API |
| `--output` / `-o` | | Write approval to file instead of stdout |

### `tvc deploy status`

Get the status of a deployment.

```bash
tvc deploy status --deploy-id <DEPLOYMENT_ID>
```

| Flag | Env Var | Description |
|------|---------|-------------|
| `--deploy-id` / `-d` | `TVC_DEPLOY_ID` | Deployment ID |

## Environment Variables

All environment variables use the `TVC_` prefix:

| Variable | Used By | Description |
|----------|---------|-------------|
| `TVC_JSON` | Global | Enable JSON output |
| `TVC_NO_INPUT` | Global | Disable interactive prompts |
| `TVC_API_KEY_FILE` | Global | Path to API key JSON file |
| `TVC_OPERATOR_KEY_FILE` | Global | Path to operator key JSON file |
| `TVC_API_URL` | Global | API base URL |
| `TVC_ORG_ID` | Global, Login | Organization ID |
| `TVC_ORG_ALIAS` | Login | Organization alias |
| `TVC_API_ENV` | Login | API environment name |
| `TVC_DEPLOY_ID` | Deploy approve, Deploy status | Deployment ID |
| `TVC_MANIFEST_ID` | Deploy approve | Manifest ID |
| `TVC_OPERATOR_ID` | Deploy approve | Operator ID |

## Configuration

Config files are stored at `~/.config/turnkey/`:

```
~/.config/turnkey/
  tvc.config.toml          # Main config (orgs, active org, cached IDs)
  orgs/
    <alias>/
      api_key.json          # API key (public + private)
      operator.json         # Operator key (public + private)
```

### Config file format (tvc.config.toml)

```toml
active_org = "default"

[orgs.default]
id = "org-uuid"
api_key_path = "/Users/you/.config/turnkey/orgs/default/api_key.json"
operator_key_path = "/Users/you/.config/turnkey/orgs/default/operator.json"
api_base_url = "https://api.turnkey.com"
```

### API key file format

```json
{
  "public_key": "hex-encoded-compressed-p256-public-key",
  "private_key": "hex-encoded-p256-private-key",
  "curve": "p256"
}
```

### Operator key file format

```json
{
  "public_key": "hex-encoded-compressed-p256-public-key",
  "private_key": "hex-encoded-master-seed"
}
```

## Automation / CI Usage

The CLI is designed for non-interactive use in CI/CD pipelines and agent-driven workflows.

### Using override flags (no login required)

```bash
export TVC_API_KEY_FILE=/path/to/api_key.json
export TVC_API_URL=https://api.turnkey.com
export TVC_ORG_ID=your-org-uuid

# All commands work without tvc login
tvc --json app create my-app.json
tvc --json deploy create my-deploy.json
tvc --json --no-input deploy approve --deploy-id $DEPLOY_ID
tvc --json deploy status --deploy-id $DEPLOY_ID
```

### Non-interactive login

```bash
tvc login --no-input --org-id $ORG_ID --alias ci --api-env prod --skip-api-key-wait
```

### Parsing JSON output

```bash
# Extract app ID from create output
APP_ID=$(tvc --json app create my-app.json | jq -r '.app_id')

# Extract deployment ID
DEPLOY_ID=$(tvc --json deploy create my-deploy.json | jq -r '.deployment_id')

# Check deployment stage
STAGE=$(tvc --json deploy status --deploy-id $DEPLOY_ID | jq -r '.stage')
```

### Full CI pipeline example

```bash
#!/bin/bash
set -euo pipefail

export TVC_API_KEY_FILE="$CI_API_KEY_PATH"
export TVC_API_URL="https://api.turnkey.com"
export TVC_ORG_ID="$CI_ORG_ID"
export TVC_JSON=true

# Create the app
APP_RESULT=$(tvc app create app-config.json)
APP_ID=$(echo "$APP_RESULT" | jq -r '.app_id')
OPERATOR_ID=$(echo "$APP_RESULT" | jq -r '.manifest_set_operator_ids[0]')

# Create the deployment
DEPLOY_RESULT=$(tvc deploy create deploy-config.json)
DEPLOY_ID=$(echo "$DEPLOY_RESULT" | jq -r '.deployment_id')

# Approve the manifest
tvc --no-input deploy approve \
  --deploy-id "$DEPLOY_ID" \
  --operator-id "$OPERATOR_ID" \
  --yes

# Check status
tvc deploy status --deploy-id "$DEPLOY_ID"
```

## Troubleshooting

**"No active organization. Run `tvc login` first."**
You need to either run `tvc login` or provide `--api-key-file`, `--api-url`, and `--org-id` flags.

**"Config file contains placeholder values"**
Edit the config file generated by `tvc app init` or `tvc deploy init` and replace all `<FILL_IN_...>` values.

**"Organization not found"**
The `--org` flag expects an alias or ID that was previously configured via `tvc login`.

**"whoami request failed"**
Your API key may not be registered in the Turnkey dashboard. Check the key setup instructions printed during `tvc login`.

**Command hangs waiting for input**
Use `--no-input` to fail instead of prompting, or provide the required flags directly.
