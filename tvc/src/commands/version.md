# tvc version

## Purpose

Print the tvc CLI release version (the crate's `CARGO_PKG_VERSION`, baked in at
compile time). Run it to check which binary you have — e.g. after the backend
rejects a request with `ClientVersionTooOld`.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| (none command-specific) | — | — | — | — | no |
| message format (global) | `--message-format` | — | — | `human` | no |

No deviation from the resolution order — there is nothing to resolve. The
version value itself is not an input; it is fixed at build time
(`tvc/src/commands/version.rs:21`).

## Interactive behavior

None. Never prompts in either mode; `--non-interactive` and JSON mode change
nothing about its behavior.

## Outputs

- Human mode: the bare version string plus newline, e.g. `0.15.0`
  (`tvc/src/commands/version.rs:13-17`, asserted by
  `tvc/tests/message_format.rs:137-143`).
- JSON mode: one NDJSON line `{"reason":"version","version":"0.15.0"}` —
  `reason` comes from the `Outcome::Version` serde tag
  (`tvc/src/outcome.rs:67`, asserted by `tvc/tests/message_format.rs:146-160`).

Exit code 0 on success.

## Side effects

None intended, but dispatch imposes some (see Gaps): before `version::run()`
is called, `Commands::run` loads `~/.config/turnkey/tvc.config.toml` — and if
the file does not exist, **creates it** with defaults
(`tvc/src/cli.rs:215-240`, save at `tvc/src/cli.rs:222`). No network, no
device, no other file I/O.

## Failure modes

The command body is infallible (`tvc/src/commands/version.rs:19-23`). All
failure paths come from the forced config load in dispatch:

- `HOME` unset → `command_error`, exit 1 (`tvc/src/cli.rs:215`).
- Config file unreadable or malformed TOML → `command_error`, exit 1
  (`tvc/src/cli.rs:225-230`).
- Config file absent and its directory unwritable → `command_error`, exit 1
  (`tvc/src/cli.rs:222`).

## Gaps

1. **[bug?] Dispatch forces a config load — and a config write — before
   `tvc version` runs, so a version check can fail, or mutate the machine.**
   `Commands::run` exempts only `yubikey create-certs` from config loading
   (`tvc/src/cli.rs:206-212`); every other command, `Version` included
   (`tvc/src/cli.rs:317`), goes through the load at `tvc/src/cli.rs:215-240`,
   which creates `~/.config/turnkey/tvc.config.toml` when absent
   (`tvc/src/cli.rs:219-223`). Consequences: `tvc version` errors when `HOME`
   is unset or the config is malformed, and on a fresh machine silently writes
   a config file as a side effect of asking for a version number — violating
   the repo's own rule that offline paths must not depend on unrelated config
   (`tvc/CLAUDE.md`, "I/O, errors, and compatibility"). The create-certs
   early-return is the established fix pattern. Corroborating hazard: the
   version integration tests do not override `HOME`
   (`tvc/tests/message_format.rs:136-160`, contrast the login test's
   `.env("HOME", temp.path())` at `tvc/tests/message_format.rs:119`), so a
   test run on a fresh dev machine would create a real config file in the
   developer's home directory.

2. **[capability] No `tvc --version` / `-V` — the conventional flag is a
   usage error.** The `Cli` derive never sets `#[command(version)]`
   (`tvc/src/cli.rs:68-70`), so clap does not generate the flag;
   `tvc --version` fails as an unknown argument (exit 2, `usage_error` in JSON
   mode). Nearly every CLI supports the flag form; supporting it here is one
   attribute (and it would sidestep gap 1, since clap prints it before
   dispatch).

3. **[docs] `handle_parse_error` documents and handles a `--version` flag
   that does not exist.** The doc comment claims "`--help`/`-h` and
   `--version` also surface as `Err`" (`tvc/src/cli.rs:149-150`) and the match
   arm handles `ErrorKind::DisplayVersion` (`tvc/src/cli.rs:158`), but with no
   `version` attribute on `Cli` that error kind is unreachable — the comment
   misstates current behavior. Fixing gap 2 makes both true; otherwise the
   comment should say the arm is defensive.

No other gaps: the command is a fixed lookup with no domain inputs, so the
operator-selection / resolution-order gap class does not apply.
