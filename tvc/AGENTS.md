# TVC CLI — Agent Guide

This document teaches an AI coding agent (Claude Code, Cursor, similar) how to
drive the `tvc` CLI end-to-end on behalf of a human. It is a runnable playbook,
not a reference manual — for exhaustive flag listings run `tvc <command> --help`.

## 1. What TVC is

`tvc` is the CLI for **Turnkey Verifiable Cloud (TVC)** — a platform that runs
your code inside AWS Nitro Enclaves with cryptographic remote attestation.
`tvc` is how you (or the agent driving on your behalf) authenticate, create
applications, deploy them, approve deployment manifests, and inspect live
deployment state.

Concepts and dashboard docs live at
<https://docs.turnkey.com/features/verifiable-cloud/overview>.

## 2. Prerequisites

Before running any `tvc` command, confirm all of the following:

1. The user has a **Turnkey account** and belongs to at least one organization.
2. You know the **organization ID** — a UUID like `01234567-89ab-cdef-0123-456789abcdef`. It is on the Turnkey dashboard **home page** (click-to-copy). Do NOT read an ID off a deployment or app detail page — that's a different UUID.
3. The user has (or can create) **Turnkey API credentials** for that org. `tvc login` will generate a keypair for you and walk the user through registering the public key on the dashboard.
4. The `tvc` binary is installed. Preferred: `cargo install tvc`. Verify with `tvc --version`.

If any of these are missing, stop and ask the user rather than guessing.

## 3. First-time setup: `tvc login`

Run `tvc login` interactively (do NOT pass `--non-interactive` on first setup —
the flow needs to prompt for the org ID and pause for dashboard registration).

```bash
tvc login
```

The flow:

1. If no orgs are configured yet, `tvc` prints the dashboard URL and prompts:
   - **Organization ID** — paste the UUID from the dashboard home page.
   - **Organization alias** — a short local name (default: `default`). This is what you'll pass to `--org` later.
2. `tvc` generates a P-256 API keypair and prints the public key.
3. `tvc` prompts you to add that public key to the dashboard: **Users → your user → New API Key → Advanced Settings → Generate API key via CLI → paste public key → Continue → Approve.** Press Enter when done.
4. `tvc` calls the Turnkey `whoami` endpoint to **verify** the org ID and credentials. If verification fails, no config is written and the error tells you which org ID failed.
5. On success, `tvc` writes:
   - `~/.config/turnkey/tvc.config.toml` — org registry and active org
   - `~/.config/turnkey/orgs/<alias>/api_key.json` — the P-256 API keypair
   - `~/.config/turnkey/orgs/<alias>/operator.json` — the operator (QOS) keypair used to approve deployment manifests

After that, subsequent `tvc` commands read those files automatically.

### CI / non-interactive use

For CI, skip `tvc login` and set these three env vars instead — they authenticate
directly without touching disk:

| Env var | Value |
|---|---|
| `TVC_ORG_ID` | the Turnkey organization UUID |
| `TVC_API_KEY_PUBLIC` | hex-encoded compressed P-256 public key |
| `TVC_API_KEY_PRIVATE` | hex-encoded P-256 private key |

Setting some but not all three is rejected. When set, env vars take precedence
over any local config file.

## 4. Common workflows

All commands below are real subcommands. Cross-check names with
`tvc --help` and `tvc <group> --help` if you're unsure.

### Create an app

An **app** is the long-lived container for a family of deployments. It owns the
quorum key and the set of operators that can approve manifests.

```bash
# Generate a template config
tvc app init --output my-app.json

# Edit my-app.json to fill in name, quorumPublicKey, operator keys, etc.
# The user typically needs to hand-edit this — do NOT guess these values.
# Pass --interactive to app init to be prompted for each field instead.

# Create the app on Turnkey
tvc app create --config-file my-app.json
```

### List apps

```bash
tvc app list                     # all apps in the active org
tvc app list --name my-app       # filter by name
```

### App live status

```bash
tvc app status --app-id <APP_UUID>
```

Returns the currently live deployment's runtime state from the cluster.

### Create a deployment

```bash
# Generate a template deploy config
tvc deploy init --output my-deploy.json

# Edit my-deploy.json (appId, pivotContainerImageUrl, expectedPivotDigest, etc.)

# Create the deployment
tvc deploy create --config-file my-deploy.json
```

`tvc deploy create --help` documents each flag override (`--app-id`,
`--qos-version`, `--pivot-image-url`, `--expected-pivot-digest`, etc.).

### Approve a deployment

Operators approve the QOS manifest before it goes live. The recommended flow
fetches the manifest and manifest ID automatically via `GetTvcDeployment`:

```bash
tvc deploy approve \
  --deploy-id <DEPLOYMENT_UUID> \
  --operator-id <OPERATOR_UUID>
```

Passing `--dangerous-skip-interactive` bypasses the section-by-section
confirmation prompts. Only do that when the human explicitly asked for it.

### Check deployment status

Two commands with different scopes — pick the right one:

- `tvc deploy status --deploy-id <ID>` — deployment record status (config, manifest, approvals).
- `tvc deploy get-status --deploy-id <ID>` — **live** runtime status polled from the cluster.

### Tail debug logs

Debug logs require debug mode at **both** the app level (created with
`--dangerous-enable-debug-mode-deployments`) **and** the deployment level
(created with `--dangerous-deploy-debug-mode`). Non-debug deployments cannot
expose logs retroactively.

```bash
tvc deploy debug-logs --deploy-id <DEPLOYMENT_UUID>              # one-shot
tvc deploy debug-logs --deploy-id <DEPLOYMENT_UUID> --poll       # follow
```

### Set the live deployment

```bash
tvc app set-live-deploy --app-id <APP_UUID> --deploy-id <DEPLOYMENT_UUID>
```

### Deletion

```bash
tvc deploy delete --deploy-id <DEPLOYMENT_UUID>
tvc app delete --app-id <APP_UUID>          # also deletes all its deployments
```

## 5. Configuration overrides

Global flags exist on `tvc` itself:

- `--non-interactive` (or `TVC_NON_INTERACTIVE=true`) — disables all prompts and fails fast on missing input. Set this whenever you're driving `tvc` from a script or another agent.
- `--message-format json` — emits newline-delimited JSON. Implies `--non-interactive`. Prefer this for machine-readable output when parsing the CLI's response.
- `--color auto|always|never` — ANSI color control.

Environment / URL overrides:

- `--api-base-url <URL>` (or `TVC_API_BASE_URL`) — override the Turnkey API endpoint. Defaults to `https://api.turnkey.com`. Available on `tvc login`; also honored by env-var auth in CI. Only needed for non-prod Turnkey environments.
- `TVC_ORG` — default value for `--org` on `tvc login`.
- Per-command env vars (`TVC_APP_ID`, `TVC_DEPLOY_ID`, `TVC_OPERATOR_ID`, `TVC_MANIFEST_ID`, `TVC_DEPLOY_CONFIG`, `TVC_APP_CONFIG`, …) — check each command's `--help` for its supported vars and precedence.

Precedence (highest wins): CLI flag > env var > config file value > built-in default.

## 6. Where things live

- **Config file:** `~/.config/turnkey/tvc.config.toml`
- **Per-org keys:** `~/.config/turnkey/orgs/<alias>/api_key.json` and `.../operator.json`
- **Reference apps and deploy configs:** <https://github.com/tkhq/tvc-examples>
- **Product docs and concepts:** <https://docs.turnkey.com/features/verifiable-cloud/overview>
- **Quickstart guide:** <https://docs.turnkey.com/getting-started/verifiable-cloud-quickstart>
- **Source of truth for command surface:** `tvc/src/commands/` in the `tkhq/rust-sdk` repo. If a command isn't there, it doesn't exist — don't invent flags.

## 7. Common gotchas

- **Org ID source.** The org ID must come from the Turnkey dashboard **home** page. IDs shown on deployment or app pages are different UUIDs and will fail the `tvc login` verification step.
- **First `tvc login` needs a TTY.** The flow pauses for dashboard registration. Do NOT wrap it in `--non-interactive` or a headless script on first setup.
- **Non-interactive login requires an existing API key.** In `--non-interactive` mode `tvc login` will not generate a new key; run interactive login once first, then automate afterwards.
- **Debug logs need double opt-in.** App created with `--dangerous-enable-debug-mode-deployments` **and** deployment created with `--dangerous-deploy-debug-mode`. Turning debug mode on permanently marks the app's quorum key as insecure — never do this on a production app.
- **`deploy status` vs `deploy get-status`.** The first shows the Turnkey deployment record; the second polls the live cluster. Same for `app status` (live cluster). Pick the one that answers the question you're actually asking.
- **Env-var auth is all-or-nothing.** Set `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` together or not at all. Partial sets are rejected with a message naming the missing vars.
- **`--api-base-url` is stored per org on `tvc login`.** Setting it on login updates the on-disk value; other commands read from the stored config, so you generally don't need to pass it again.
- **Config is only persisted after credential verification.** If `tvc login` fails at the `whoami` step, the config file is left untouched — no cleanup needed. Fix the org ID or credentials and re-run.
- **`--message-format json` implies `--non-interactive`.** JSON mode never prompts; it fails fast on missing required inputs. Provide every value up front.
