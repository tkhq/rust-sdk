# tvc version

*Depends on: Part 00 (00_preliminaries.md). Conformance unit: the version command.*

## Purpose (informative)

Print the tvc CLI release version: the crate's `CARGO_PKG_VERSION`, baked in at compile time.
Run it to check which binary you have, for example after the backend rejects a request with `ClientVersionTooOld`.

## Acceptance scenario (normative)

The value `0.15.0` is the crate version on this branch (tvc/Cargo.toml:3).

| Step | Action | Expected observation |
|---|---|---|
| 1 | Set `HOME=/home/tvc-fresh`, an empty directory. Run `tvc version`. | stdout is `0.15.0` followed by one newline. Exit code 0. |
| 2 | Run `tvc --message-format=json version`. | stdout is one NDJSON object with `reason` = `version` and `version` = `0.15.0`. Exit code 0. |

All steps pass in one run from a clean start.

## Inputs (normative)

| Input | Flag | Env | Config key | Default | Prompted |
|---|---|---|---|---|---|
| (no command-specific inputs) | (none) | (none) | (none) | (none) | no |
| message format (global) | `--message-format` | (none) | (none) | `human` | no |

The command defines no configuration values, so the Part 00 resolution order has nothing to resolve.
The reported version is a compile-time constant (tvc/src/commands/version.rs:21).

## Interactive behavior (normative)

The command MUST NOT prompt.
Interactive mode and non-interactive mode MUST produce the same output.
INV-G1 applies in JSON mode with no extra effect, because no prompt path exists (tvc/src/commands/version.rs:19-23).

## Outputs (normative)

- In human mode stdout MUST carry the version string and one trailing newline, for example `0.15.0`. Pinned by tvc/src/commands/version.rs:13-17 and tvc/tests/message_format.rs:137-143.
- In JSON mode stdout MUST carry one NDJSON object with `reason` = `version` and `version` set to the crate version. The serde tag on `Outcome` supplies `reason` (tvc/src/outcome.rs:30; variant at tvc/src/outcome.rs:67). Asserted by tvc/tests/message_format.rs:146-160.
- The exit code MUST be 0 on success (tvc/src/cli.rs:120).

## Side effects (normative)

The command body MUST NOT write any file.
The command MUST NOT send a network request and MUST NOT access a device.
Dispatch loads the tvc config before the command runs, per INV-G4 (tvc/src/cli.rs:215-240).
When the file is absent, dispatch creates it (tvc/src/cli.rs:219-223).
Gap 1 tracks this dispatch behavior.

## Failure modes (normative)

The command body is infallible (tvc/src/commands/version.rs:19-23).
Every failure MUST come from the tvc config load in dispatch.

| Condition | Observation | Citation |
|---|---|---|
| `HOME` unset | `code` = `command_error`; exit code 1 | tvc/src/cli.rs:215 |
| tvc config unreadable, or malformed TOML | `code` = `command_error`; exit code 1 | tvc/src/cli.rs:225-230 |
| tvc config absent and its directory unwritable | `code` = `command_error`; exit code 1 | tvc/src/cli.rs:219-222 |

## Test vectors (normative)

GV-2 and GV-3 (00_preliminaries.md) pin the happy path in both output modes.
The vectors below add only observations that GV-2 and GV-3 do not cover.
Every observation is deterministic for one build of the binary; comparison excludes no fields.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | `HOME=/home/tvc-fresh`, an empty directory | `tvc version` | stdout is `0.15.0` plus one trailing newline and nothing else; exit code 0; after the run `/home/tvc-fresh/.config/turnkey/tvc.config.toml` exists (tvc/src/commands/version.rs:13-17; tvc/tests/message_format.rs:137-143; tvc/src/cli.rs:219-223). |
| V-2 | `HOME=/home/tvc-fresh`, an empty directory | `tvc --message-format=json version` | One NDJSON object with `reason` = `version` and `version` = `0.15.0`; exit code 0 (tvc/src/outcome.rs:30; tvc/tests/message_format.rs:146-160). |
| V-3 | `HOME` unset | `tvc --message-format=json version` | One NDJSON object with `reason` = `command_error`, `code` = `command_error`, `message` contains `HOME environment variable not set`; exit code 1 (tvc/src/cli.rs:215; tvc/src/output.rs:335-341). |
| V-4 | The tvc config contains the malformed TOML `not = [toml` | `tvc --message-format=json version` | One NDJSON object with `reason` = `command_error`, `code` = `command_error`, `message` contains `failed to parse config file`; exit code 1 (tvc/src/cli.rs:229-230). |
| V-5 | Any environment | `tvc --version --message-format=json` | One NDJSON object with `reason` = `command_error`, `code` = `usage_error`, `message` contains `unexpected argument '--version'`; exit code 2 (tvc/src/cli.rs:68-70; tvc/src/cli.rs:160-176). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The reported version MUST equal the crate version compiled into the binary. | `env!("CARGO_PKG_VERSION")` resolves at compile time (tvc/src/commands/version.rs:21). Behavioral check: tvc/tests/message_format.rs:142 and tvc/tests/message_format.rs:159 compare against the same constant. |
| INV-2 | `version::run` MUST NOT return an error. | The body is one infallible `Ok` expression (tvc/src/commands/version.rs:19-23). |
| INV-3 | The command body MUST NOT prompt and MUST NOT perform I/O. | `run()` takes no context, client, or filesystem handle, so no I/O interface is reachable (tvc/src/commands/version.rs:19). |

## Gaps (informative)

1. **[bug?] Dispatch forces a tvc config load, and possibly a write, before `tvc version` runs.**
   A version check can therefore fail, or mutate the machine.
   `Commands::run` exempts only `yubikey create-certs` from the load (tvc/src/cli.rs:206-213).
   Every other command, `Version` among them (tvc/src/cli.rs:317), goes through the load at tvc/src/cli.rs:215-240.
   The load creates the tvc config when the file is absent (tvc/src/cli.rs:219-223).
   Consequences: `tvc version` errors when the environment lacks `HOME`, and when the tvc config holds malformed TOML.
   On a fresh machine it silently writes the tvc config as a side effect of a version query.
   This conflicts with the repo rule that offline paths do not depend on unrelated config (tvc/CLAUDE.md, "I/O, errors, and compatibility").
   The `create-certs` early return is the established fix pattern.
   Corroborating hazard: the version integration tests do not override `HOME` (tvc/tests/message_format.rs:136-160).
   Contrast the login test's `.env("HOME", temp.path())` at tvc/tests/message_format.rs:119.
   A test run on a fresh dev machine would create the tvc config in the developer's home directory.

2. **[capability] The conventional version flag is a usage error.**
   tvc offers no `--version` and no `-V`.
   The `Cli` derive never sets `#[command(version)]` (tvc/src/cli.rs:68-70), so clap does not generate the flag.
   `tvc --version` fails as an unknown argument (exit code 2, `usage_error` in JSON mode; V-5).
   Nearly every CLI supports the flag form.
   Support here is one attribute, and it would sidestep gap 1 because clap prints the version before dispatch.

3. **[docs] `handle_parse_error` covers a `--version` flag that does not exist.**
   The comment claims `--help` and `--version` surface as `Err` (tvc/src/cli.rs:149-150).
   The match arm handles `ErrorKind::DisplayVersion` (tvc/src/cli.rs:158).
   With no `version` attribute on `Cli` that error kind is unreachable, so the comment misstates current behavior.
   A fix for gap 2 makes both statements true; without it, the comment ought to call the arm defensive.

No other gaps: the command is a fixed lookup with no domain inputs, so the operator-selection and resolution-order gap classes do not apply.
