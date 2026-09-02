# tvc app init

## Purpose
Writes an app-config JSON file for `tvc app create` to consume. By default it is a
placeholder template (`<FILL_IN_...>` markers) the user edits by hand; with
`--interactive` it walks prompts for the placeholder fields and writes a filled config.
Run it once per new app, before `app create`.

## Inputs
| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| output path | `-o, --output <PATH>` | `TVC_APP_CONFIG_OUT` | — | `app.json` | no |
| interactive fill | `--interactive` | — | — | false | n/a |
| operator public key (template seed) | — | — | active org's default operator (implicit) | `<FILL_IN_OPERATOR_PUBLIC_KEY>` | yes, only with `--interactive` |
| app name | — | — | — | `<FILL_IN_APP_NAME>` | yes, only with `--interactive` |
| manifest set name | — | — | — | `<FILL_IN_MANIFEST_SET_NAME>` | yes, only with `--interactive` |
| quorum public key | — | — | — | `KNOWN_QUORUM_KEY` (well-known, insecure) | never |
| manifest threshold / operator count | — | — | — | 1 / one operator `operator-1` | never |
| share set | — | — | — | omitted → dev-known share set at create time | never |
| enableEgress, dangerousEnableDebugModeDeployments | — | — | — | false | never |

Resolution order holds for `--output` (flag > env > default). The operator-key seed is
config-state only — no flag or env can override which operator seeds the template
(`tvc/src/operator.rs:282-304`). Globals are inherited normally; `--interactive` is
rejected when non-interactive is in effect (flag, `TVC_NON_INTERACTIVE`, or JSON mode,
which forces non-interactive via `tvc/src/output.rs:209-216`).

## Interactive behavior
Without `--interactive` there are no prompts at all; the command is fully
non-interactive-safe and JSON-mode-safe.

With `--interactive` (`tvc/src/commands/app/init.rs:36-42`):
- Under `--non-interactive` / `TVC_NON_INTERACTIVE` / `--message-format json`: bails
  immediately with "--interactive conflicts with --non-interactive or
  TVC_NON_INTERACTIVE=true" (`tvc/src/prompts.rs:29-31`; test
  `tvc/tests/non_interactive.rs:234-249`).
- Otherwise requires a real TTY on stdin (`tvc/src/prompts.rs:33-38`).
- Prompt order (`tvc/src/config/app.rs:114-141`, placeholder-driven): app name →
  manifest set name → each placeholder operator's public key (the saved operator key is
  offered as the prompt default). Share-set prompts exist in the code but are
  unreachable from a fresh template (share_set_params is None). Empty input bails with
  "`<prompt>` cannot be empty" (`tvc/src/prompts.rs:42-48`).
- Never prompted, even interactively: quorum public key, threshold, operator count,
  share set, egress, debug mode.

## Outputs
Human mode (`tvc/src/commands/app/init.rs:83-104`): "Created app config template:
<path>" + "Edit the file to fill in your values, then run: tvc app create --config-file
<path>"; interactive variant drops the "edit" line and says "Created app config:
<path>".

JSON mode: one NDJSON line, reason `app_config_created`, fields `command` ("app init"),
`path`, `template`, `interactive` (test `tvc/tests/message_format.rs:60-86`). The
`interactive: true` shape is unreachable in JSON mode (JSON forces non-interactive,
which conflicts with `--interactive`), and `template` is always `!interactive`.

## Side effects
- Writes the output JSON file; refuses to overwrite an existing file
  (`tvc/src/commands/app/init.rs:48-50`).
- Reads `~/.config/turnkey/` config; dispatch creates a default config file if absent —
  this happens for all commands before the command body runs (`tvc/src/cli.rs:219-223`).
- Best-effort read of the default operator public key: local backend reads the sole
  registered key file from disk; yubikey backend reads the cached registry key (no
  device access); hosted backend joins stored key halves (`tvc/src/operator.rs:282-304`).
- No network calls, no Turnkey activities, no device interaction.

## Failure modes
All error paths are plain anyhow errors → JSON `code: command_error`, exit 1
(`tvc/src/errors.rs:93-102` recognizes only `MissingResource` and client errors):
- Output file already exists (`tvc/src/commands/app/init.rs:48-50`).
- `--interactive` under non-interactive mode; `--interactive` without a TTY
  (`tvc/src/prompts.rs:29-38`).
- Empty prompt submission; serialize/write failures.
Clap-level failures (unknown flags) are `usage_error`, exit 2, per the global contract.

## Gaps

1. **[capability] The operator-key seed is locked to the org default backend's sole
operator; the user cannot pick which configured operator seeds the config.**
`default_operator_public_key` consults only `org.default_operator_kind` and then the
*sole* record of that kind — `select_local_operator` errors on multiple locals
(`tvc/src/config/turnkey/mod.rs:452-456`), so with two local operators the prefill
silently vanishes and the placeholder appears instead (`tvc/src/operator.rs:282-304`).
There is no `--operator`/`--operator-public-key` flag and no selection prompt; contrast
`app create`, which enumerates every known candidate and prompts to choose
(`tvc/src/commands/app/create.rs:71-96`). This is exactly the "state/default silently
constrains an explicit choice" shape from the audit brief; the only escape is pasting a
key by hand in `--interactive` mode.

2. **[capability] The quorum public key cannot be supplied and silently defaults to the
well-known insecure key, even in interactive mode.** The template hard-codes
`KNOWN_QUORUM_KEY` (`tvc/src/config/app.rs:73`), documented in code as "for applications
that do not need secure quorum keys" (`tvc/src/config/app.rs:65-66`). It is not a
`<FILL_IN` placeholder, so `fill_interactively` never prompts for it
(`tvc/src/config/app.rs:114-141`), no flag exists, and neither the help text nor the
success message warns that the default quorum key is publicly known. The interactive
path then declares the config done ("Run: tvc app create --config-file ...",
`tvc/src/commands/app/init.rs:86-91`) with the insecure key baked in — despite
`keys generate-local-quorum-key` existing to mint a real one.

3. **[capability] No path — flag or prompt — to a production share set; the template
commits the app to the dev-known share set.** The template writes
`shareSetParams: null` (`tvc/src/config/app.rs:88`), which `app create` resolves to
`dev-known-share-set` built from `KNOWN_SHARE_SET_KEYS`, whose secrets are assumed
well-known (`tvc/src/config/app.rs:52-63,94-107,213-223`). `--interactive` never asks
(only pre-existing placeholders are filled), so a secure share set requires hand-writing
the JSON shape with no scaffold. Same applies to threshold (fixed 1), operator count
(fixed 1, named "operator-1"), and enableEgress — interactive mode cannot change any of
them (`tvc/src/config/app.rs:70-91`).

4. **[consistency] No `--from-app` seeding, unlike `deploy init --from-deployment`.**
`deploy init` fetches an existing deployment and copies every recoverable field
(`tvc/src/commands/deploy/init.rs:41-42,88-99`); `app init` has no equivalent to
regenerate a config from an existing app even though the API exposes app data (used by
`app list`/`app status`).

5. **[consistency] Fixed default filename guarantees a collision on the second run.**
`app init` defaults to `app.json` and bails if it exists
(`tvc/src/commands/app/init.rs:23,48-50`); `deploy init` defaults to a timestamped
`deploy-<ts>.json` that never collides (`tvc/src/commands/deploy/init.rs:71-74`).
Neither offers `--force`, but only `app init`'s default makes the error the norm.
(`keys init-local-quorum-key` shares app init's fixed-name shape,
`tvc/src/commands/keys/init_local_quorum_key.rs:22-31`.)

6. **[docs] README documents a `--name` flag that does not exist.** `tvc/README.md:46`
shows `tvc app init --name my-app --output my-app.json`; `Args` has only `--output` and
`--interactive` (`tvc/src/commands/app/init.rs:17-32`), so the documented invocation
exits 2 with a usage error. Ironically the app name is the first thing the template
makes you fill in, so a `--name` flag is also a plausible [capability] add.

7. **[consistency] `--interactive` exists on `app init` and `deploy init` but not on
`keys init-local-quorum-key`, and has no env var anywhere.** The three template
generators diverge: quorum-key init has no interactive fill at all
(`tvc/src/commands/keys/init_local_quorum_key.rs:16-26`), and `--interactive` lacks an
env/config equivalent while its sibling input `--output` has `TVC_APP_CONFIG_OUT`
(`tvc/src/commands/app/init.rs:18-31`). Minor, but breaks the flag>env>config
uniformity the LONG_ABOUT advertises (`tvc/src/cli.rs:18-23`).

8. **[consistency] Usage-shaped errors classify as `command_error`/exit 1 instead of
`usage_error`/exit 2.** The `--interactive` vs non-interactive conflict is a plain
`bail!` in command code (`tvc/src/prompts.rs:29-31`) rather than a clap
`conflicts_with`, and semantic failures like "File already exists" cannot reach
`invalid_input` because `ErrorCode::InvalidInput` is dead code — nothing produces it
(`tvc/src/errors.rs:54-56`, `tvc/src/errors.rs:93-102`). JSON consumers therefore see
the fallback code for what the taxonomy in `tvc/src/cli.rs:54-64` describes as distinct
classes. Shared with `deploy init`.

9. **[bug?] Interactive fill accepts any non-empty text as an operator public key.**
`fill_interactively` uses `required_text` with no format check
(`tvc/src/config/app.rs:122-127`), even though an `OperatorPublicKey` parser exists and
is used by `app create`'s reuse logic (`tvc/src/commands/app/create.rs:140`). A typo'd
key survives `app create`'s validation too (placeholder/threshold checks only,
`tvc/src/config/app.rs:160-211`) and is first rejected server-side.
