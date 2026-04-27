# `tvc`

CLI for [Turnkey Verifiable Cloud](https://turnkey.com) — see [this guide](https://docs.turnkey.com/getting-started/verifiable-cloud-quickstart) for end-to-end example usage.

## Installation

```bash
cargo install tvc
```

## Overview

`tvc` is interactive by default. Most commands prompt for missing inputs when
stdin is a TTY, walk you through filling out config files, and print a
copy-pasteable replay command at the end so you can re-run the same flow
non-interactively. Every prompt has a corresponding flag, so scripts and CI
never have to deal with prompts.

## Quick start (interactive)

```bash
# Authenticate. First run walks you through org setup; subsequent runs use
# `--org <alias>` from your config.
tvc login

# Generate and fill in an app config in one go.
tvc app init --interactive --output my-app.json
tvc app create -c my-app.json

# Generate and fill in a deployment config in one go.
tvc deploy init --interactive --output my-deploy.json
tvc deploy create -c my-deploy.json

# Approve the deployment's manifest. Walks the four sections of the manifest
# and asks for confirmation on each.
tvc deploy approve --deploy-id <DEPLOYMENT_UUID>
```

If you'd rather edit a JSON template by hand:

```bash
tvc app init --output my-app.json     # writes a placeholder template
$EDITOR my-app.json
tvc app create -c my-app.json
```

`tvc app create` / `tvc deploy create` will also walk you through any
remaining `<FILL_IN_*>` placeholders interactively if a TTY is available.

## Non-interactive mode (CI / scripts)

For CI and automation, set `TVC_NON_INTERACTIVE=1`:

```bash
export TVC_NON_INTERACTIVE=1

tvc login --org prod
tvc app create -c ./my-app.json
tvc deploy create -c ./my-deploy.json
tvc deploy approve \
  --deploy-id "$DEPLOY_ID" \
  --operator-id "$OPERATOR_ID" \
  --dangerous-skip-interactive
```

With `TVC_NON_INTERACTIVE=1`:

- Every command that would otherwise prompt errors immediately with a clear
  `flag X is required in non-interactive mode` message — instead of stalling
  on a prompt that nobody will answer.
- `--interactive` on `tvc deploy init` / `tvc app init` is rejected (would
  conflict).
- Existing flag-driven invocations behave identically to before.

## Replay command banner

After every successful interactive run, `tvc` prints a banner with the
equivalent flag-based command:

```
─────────────────────────────────────
  To skip prompts next time, run:

  tvc deploy approve \
    --deploy-id deploy_abc123 \
    --operator-id op_111 \
    --dangerous-skip-interactive
─────────────────────────────────────
```

Contract:

- **Scalar flags are always printed**, even when the value matched the default.
  This future-proofs your saved command against default changes.
- **Boolean flags** appear when `true`, are omitted when `false` (matches Unix
  CLI conventions).
- **Secret values** (e.g. `--operator-seed`) appear as `<PATH>` placeholders;
  the real value is never echoed.

## Common flows

### Create an app

```bash
# Option A — interactive walk:
tvc app init --interactive --output my-app.json
tvc app create -c my-app.json

# Option B — edit a template by hand:
tvc app init --output my-app.json
# Edit my-app.json to fill in required values (quorumPublicKey, operator keys, etc.)
tvc app create -c my-app.json
```

### Create and approve a deployment

```bash
# Generate and fill in the deployment config.
tvc deploy init --interactive --output my-deploy.json
tvc deploy create -c my-deploy.json

# Recommended: approve by deployment ID (fetches manifest + manifest_id from API).
tvc deploy approve \
  --deploy-id <DEPLOYMENT_UUID> \
  --operator-id <OPERATOR_UUID>     # from your `tvc app create` output

# Alternative: approve from a local manifest file.
tvc deploy approve \
  --manifest manifest.json \
  --manifest-id <MANIFEST_UUID> \    # from your `tvc deploy create` output
  --operator-id <OPERATOR_UUID>

# Save a single provisioning bundle JSON for later re-encryption / submission flows.
tvc deploy provisioning-details \
  --deploy-id <DEPLOYMENT_UUID> \
  --provision-bundle-out provisioning-bundle.json
```

## Testing

The `tvc` test suite has three layers:

- **Unit tests** in `src/` modules — fast, deterministic checks for pure
  logic: config validation (`has_placeholders`), replay-banner rendering,
  manifest section rendering, shell quoting.
- **Integration tests** in `tests/*.rs` driven by `assert_cmd` — exercise
  flag-driven paths end-to-end through the binary without a TTY.
- **PTY tests** in `tests/pty.rs` — drive the real `tvc` binary inside a
  pseudo-terminal via [`rexpect`](https://crates.io/crates/rexpect). This is
  how we test the actual interactive code path: inquire's TTY rendering, raw
  key handling, and `Select` widgets that piped stdin can't drive.

PTY tests cover:

- `approve_walks_all_four_sections_with_yeses` — full manifest-approval flow
  with section-by-section confirmation and the replay banner.
- `approve_bails_when_user_rejects_pivot` — early-bail behavior when the user
  says "no" mid-approval.
- `login_with_empty_org_id_bails` — empty Organization ID is rejected with the
  expected error.

Run all tests:

```bash
cargo test -p tvc
```

PTY tests are gated `#[cfg(unix)]` because `rexpect` uses Unix PTYs. On
Windows, `tvc` itself works fine (inquire uses ConPTY under the hood), but
our test harness doesn't cover that platform. CI runs on `ubuntu-latest` and
exercises the full suite there.
