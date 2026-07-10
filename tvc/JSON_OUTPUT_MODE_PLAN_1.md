# TVC JSON Output Mode Architecture

This document defines the output contract and implementation constraints for
TVC-116. It supplements the Cargo-inspired `Shell` and `Message` design with
TVC-specific rules needed for reliable automation and future command growth.

## 1. Treat stdout as a protocol

In human mode, commands may render friendly prose, status lines, and tables. In
JSON mode, stdout is a machine-readable protocol:

- Every non-empty stdout line must be a complete, valid JSON object.
- Human prose, progress text, tables, prompts, and incidental output must never
  reach stdout.
- Successful results and expected command failures must be emitted through the
  central output layer as structured messages.
- A structured failure is written to stdout and accompanied by a nonzero exit
  status.
- Developer diagnostics produced by `tracing` remain a separate channel on
  stderr. A last-resort failure to serialize or write the structured response
  may also be reported on stderr.

These rules make stdout safe to pipe directly into a JSONL consumer without
requiring heuristics to distinguish user-facing text from structured data.

## 2. Use JSON Lines consistently

`--message-format=json` uses JSON Lines for every TVC command:

- Each message is serialized as one compact JSON object followed by a newline.
- A one-shot command emits exactly one terminal success or error message.
- A long-running command may emit zero or more progress messages followed by
  exactly one terminal success or error message.
- Commands do not wrap messages in an array and do not pretty-print them.

Using one transport for both one-shot and streaming commands allows progress
events to be added later without changing how consumers parse stdout.

## 3. Design stable, evolvable message schemas

Every JSON message has a top-level, kebab-case `reason` discriminator. Reason
strings and existing field meanings are part of TVC's externally visible
automation contract.

- Prefer broad reasons that describe durable domain events, such as
  `deployment-result`, `deployment-progress`, or `command-error`.
- Do not encode implementation details, versions, or transient workflow steps
  into reason strings.
- Evolve messages additively. New optional fields and new reason values are
  compatible; removing fields, renaming reasons, or changing field types is
  breaking.
- Consumers must be allowed to ignore unknown fields and reason values.
- Error messages include a stable machine-readable code in addition to a human
  explanation and actionable flag hints where applicable.

Before implementation resumes after the rebase, survey every command's
success, progress, warning, and failure actions. Finalize the initial reason
catalog from that post-rebase command surface rather than preserving names
from the stale branch automatically.

## 4. Make output dependencies explicit

All user-visible output flows through an invocation-scoped `Shell` (or a small
CLI context that owns it):

- `Shell` owns stdout and stderr writers, message format, color policy, and
  verbosity.
- Command handlers receive the output dependency explicitly instead of using a
  global or opening standard streams themselves.
- Human and JSON rendering decisions are centralized in `Shell` and typed
  `Message` implementations.
- TVC command modules must not call `print!`, `println!`, `eprint!`, or
  `eprintln!` for user-visible output.
- Interactive input is accessed through the invocation context or an injected
  prompt policy so output mode and prompt behavior cannot diverge.
- Developer-only diagnostics continue to use `tracing`, not `Shell` messages.

Tests should substitute in-memory writers so command output can be asserted
without global stream redirection and without serializing test execution.

## Confirmed interactivity policy

JSON mode is always noninteractive, including when stdin is a TTY.

- If a command has a deterministic, documented fallback, it uses that fallback
  without prompting.
- If explicit input is required, the command immediately emits a structured
  JSON error and exits nonzero.
- Destructive confirmation must be supplied through an explicit
  noninteractive flag; JSON mode never assumes confirmation.
- No JSON-mode execution may block waiting for stdin.

Human mode retains the interactive behavior introduced by TVC-13: commands may
prompt when permitted and stdin is a TTY, while explicitly noninteractive or
non-TTY execution fails fast when required input is missing.

## Verification expectations

The implementation must include tests that demonstrate:

- Every JSON-mode stdout line parses independently as JSON.
- One-shot commands emit one terminal success or error message.
- JSON errors use stdout, preserve a nonzero exit status, and do not leak human
  error formatting to stderr.
- JSON mode never invokes prompts, even with a TTY-capable test input.
- Human mode preserves existing readable output and color behavior.
- In-memory `Shell` writers capture stdout and stderr independently.
- A guard test prevents direct standard-stream output from being introduced in
  TVC command modules.
