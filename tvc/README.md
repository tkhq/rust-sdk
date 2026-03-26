# TVC CLI

Command-line interface for [Turnkey Verifiable Cloud](https://turnkey.com).

## Build

From the workspace root:

```bash
cargo build -p tvc --release
```

## Basic flow

```bash
# 1. Login and create local config
tvc login

# 2. Create an app
tvc app init --output app.json
# edit app.json
tvc app create app.json

# 3. Create a deployment
tvc deploy init --output deploy.json
# edit deploy.json
tvc deploy create deploy.json

# 4. Approve the manifest
tvc deploy approve --deploy-id <DEPLOYMENT_ID>

# 5. Check status
tvc deploy status --deploy-id <DEPLOYMENT_ID>
```

## Global flags

These apply to all commands:

| Flag | Env Var | Purpose |
|---|---|---|
| `--json` | `TVC_JSON` | Emit machine-readable command results on stdout |
| `--no-input` | `TVC_NO_INPUT` | Fail instead of prompting when input is required |
| `--quiet`, `-q` | | Suppress non-essential status output |
| `--api-key-file <PATH>` | `TVC_API_KEY_FILE` | Override the API key source |
| `--api-url <URL>` | `TVC_API_URL` | Override the Turnkey API base URL |
| `--org-id <ID>` | `TVC_ORG_ID` | Override the organization ID |

## Login

`tvc login` sets up local config, API key material, and operator key material.

Examples:

```bash
# Interactive
tvc login

# Select an existing configured org
tvc login --org my-alias

# Non-interactive org config setup
tvc --no-input --org-id <ORG_ID> login --alias ci --api-env prod
```

Notes:
- first-time API key approval is still manual
- the generated public key and dashboard instructions are always printed
- `--quiet` suppresses routine status messages, not the required setup instructions

## Programmatic use

For machine consumption, use `--json` on commands that return useful data:

```bash
tvc --json app create app.json
tvc --json deploy create deploy.json
tvc --json deploy status --deploy-id <DEPLOYMENT_ID>
tvc --json deploy approve --deploy-id <DEPLOYMENT_ID> --yes --skip-post
```

For config-less automation, provide all three client overrides:

```bash
tvc \
  --api-key-file /path/to/api_key.json \
  --api-url https://api.turnkey.com \
  --org-id <ORG_ID> \
  --json \
  deploy status --deploy-id <DEPLOYMENT_ID>
```

If those overrides are not provided, API-backed commands use the active local `tvc login` config.

## Deploy approval

Important behavior:
- `--no-input` does not imply approval
- non-interactive approval still requires `--yes`
- interactive review text goes to stderr
- approval data goes to stdout

Examples:

```bash
# Review and approve interactively
tvc deploy approve --deploy-id <DEPLOYMENT_ID>

# Non-interactive approval
tvc --no-input deploy approve --deploy-id <DEPLOYMENT_ID> --yes

# Generate approval locally without posting
tvc deploy approve --manifest manifest.json --operator-seed seed.hex --yes --skip-post
```
