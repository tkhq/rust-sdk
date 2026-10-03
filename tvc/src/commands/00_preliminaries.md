# tvc CLI specifications: Part 00 (Preliminaries)

*Depends on: nothing. Every per-command specification depends on this part.*

## Purpose (informative)

The documents in this directory specify the observable behavior of each `tvc` command.
Each specification pins the implementation on this branch.
The quality bar: the implementation and its tests agree with these documents on every observable behavior the vectors cover.

## Requirement language (normative)

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY follow RFC 2119.
A sentence without one of these keywords is informative.
A normative sentence states what the implementation does today and what its tests keep true.
A Gaps entry proposes a change. Gaps sections are informative.

## Terminology (normative)

One word per concept. Synonyms are defects. Every specification uses these terms exactly.

| Term | Definition |
|---|---|
| **tvc config** | The file at `~/.config/turnkey/tvc.config.toml` (tvc/src/config/turnkey.rs:33). It stores orgs, operator records, and the active org. |
| **command config** | A file that one command consumes through `--config-file`. |
| **org** | One Turnkey organization entry in the tvc config. |
| **active org** | The org that a command acts on when no other org is chosen. |
| **profile** | A saved login: an org entry plus its local API key files. |
| **operator record** | One operator entry stored for an org in the tvc config. |
| **operator kind** | The variant of an operator record: `hosted`, `local`, or `yubikey`. |
| **default operator kind** | The per-org tvc config field that selects the operator kind when a command offers no explicit choice. |
| **interactive mode** | stdin is a TTY, `--non-interactive` is absent, and the output format is human. A command MAY prompt. |
| **non-interactive mode** | Prompts are disabled. A missing required input is an error. |
| **JSON mode** | `--message-format json`. Output is NDJSON. JSON mode implies non-interactive mode (INV-G1). |
| **human mode** | `--message-format human`, the default output format. |
| **activity** | One Turnkey API state change that a command submits. |
| **vector** | One table row that maps concrete inputs to an exact expected observation. |
| **acceptance scenario** | The end-to-end step table that defines success for one command. |

## Global CLI contract (normative)

### Value resolution

A command MUST resolve each configuration value from the highest ranked source (tvc/src/cli.rs:19).

1. Command line flag.
2. Environment variable.
3. Config file value.
4. Built-in default.

Two exceptions apply (tvc/src/cli.rs:25).

- `--pivot-args` replaces the config file list. It does not append.
- The debug mode flags only turn debug mode on. To disable debug mode, remove it from the config file and omit the flag.

### Error taxonomy

In JSON mode a failed command MUST emit one JSON object with a `reason` field and a `code` field (tvc/src/cli.rs:50).

| Code | Meaning |
|---|---|
| `missing_required_input` | A required value was absent in non-interactive mode. |
| `usage_error` | Argument parsing failed. |
| `invalid_input` | Semantic validation failed in the command. |
| `unauthorized` | HTTP 401 or 403. |
| `not_found` | HTTP 404, or a resource that resolved to empty. |
| `api_error` | Another non-success HTTP status, or a failed or unexpected activity. |
| `approval_required` | The activity needs more approvals. |
| `network_error` | The request never reached the server. |
| `command_error` | The fallback for everything else. |
| `client_version_too_old` | Present in `ErrorCode` (tvc/src/errors.rs:65) and absent from the CLI help taxonomy. |

### Exit codes

| Exit code | Meaning | Mechanism |
|---|---|---|
| 0 | Success. | `Cli::run` (tvc/src/cli.rs:120) |
| 1 | Runtime error. | `Cli::run` (tvc/src/cli.rs:138) |
| 2 | Usage error. | clap default; the JSON path exits through `handle_parse_error` (tvc/src/cli.rs:145) |

### Global vectors

Vector comparison excludes nondeterministic fields: timestamps, API assigned identifiers, and durations.

| # | Invocation | Expected observation |
|---|---|---|
| GV-1 | `tvc definitely-not-a-command --message-format json` | stdout carries one JSON object with `code` = `usage_error`; exit code 2 (tvc/src/cli.rs:160). |
| GV-2 | `tvc version` | One human line with the crate version; exit code 0. |
| GV-3 | `tvc version --message-format json` | One JSON object with `reason` = `version`; exit code 0 (tvc/tests/message_format.rs:136). |
| GV-4 | `tvc --help` | clap help text, never JSON; exit code 0 (tvc/src/cli.rs:156). |

## Global invariants (normative)

Each invariant names the mechanism that enforces it.

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-G1 | In JSON mode a command MUST NOT prompt. | `Ctx::new` forces the non-interactive flag when the output format is JSON (tvc/src/output.rs:210). Behavioral check: tvc/src/output.rs:536. |
| INV-G2 | The process exit code MUST be 0, 1, or 2. | `Cli::run` maps success to 0 and runtime failure to 1 (tvc/src/cli.rs:111). Parse failures exit with code 2 (tvc/src/cli.rs:145). |
| INV-G3 | In JSON mode every runtime error MUST surface as exactly one NDJSON object with `reason` and `code`. | The output boundary renders errors through `ErrorMessage::from_error` (tvc/src/cli.rs:130). |
| INV-G4 | Every command except `yubikey create-certs` MUST load the tvc config before it runs, and MUST create the file when it is absent. | Dispatch order in `Commands::run` (tvc/src/cli.rs:206). |

## Per-command specification structure (normative)

Every per-command specification MUST contain these sections, in this order.

| # | Section | Label |
|---|---|---|
| 1 | H1 title: `# tvc <command>` | |
| 2 | Dependency line: `*Depends on: Part 00 (00_preliminaries.md). Conformance unit: the <command> command.*` | |
| 3 | `## Purpose` | informative |
| 4 | `## Acceptance scenario` | normative |
| 5 | `## Inputs` | normative |
| 6 | `## Interactive behavior` | normative |
| 7 | `## Outputs` | normative |
| 8 | `## Side effects` | normative |
| 9 | `## Failure modes` | normative |
| 10 | `## Test vectors` | normative |
| 11 | `## Invariants` | normative |
| 12 | `## Gaps` | informative |

The acceptance scenario is a table of concrete steps and expected observations, followed by a pass criterion.
Test vectors are rows named `V-n`. Each vector gives concrete input values and cites the code or test that pins the observation.
A citation names a file and a line or line range.
A bare file name is the command's own source file, or the unique file of that name under `tvc/src/` or `tvc/tests/`.
A relative path resolves under `tvc/src/` when it does not resolve from the repository root.
Invariants are rows named `INV-n`. Each invariant names its enforcing mechanism.
Prose follows the tk-brain skill `writing-executable-specifications` (04-skills/eng/writing-executable-specifications/SKILL.md): ASD-STE100, RFC 2119, no em or en dashes, no negative parallelism.
Validate each changed specification with the skill script `scripts/check-prose.py`. Zero errors before merge.
