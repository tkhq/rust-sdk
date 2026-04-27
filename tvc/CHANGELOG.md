# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Interactive mode: every command that reads a config file or needs a user
  choice now prompts in a TTY when the input is missing, and prints a
  copy-pasteable replay command at the end of each successful run.
- `TVC_NON_INTERACTIVE=1` environment variable. When set, every command that
  would otherwise prompt errors immediately with a clear message
- `--interactive` flag on `tvc deploy init` and `tvc app init`. Walks the user
  through each placeholder field and writes a filled config instead of a
  template. Conflicts with `TVC_NON_INTERACTIVE=1`.
- `tvc deploy approve` now offers a select prompt for `--operator-id` when
  more than one operator ID is saved locally and stdin is a TTY. Single saved
  ID and non-interactive paths keep their previous "auto-pick first" behavior.
- PTY-based integration tests (`tests/pty.rs`, gated `#[cfg(unix)]`) that
  drive the real binary through a pseudo-terminal

### Changed

- **Breaking:** `tvc deploy create` and `tvc app create` now take the config
  file as `-c <PATH>` / `--config-file <PATH>` instead of a positional
  argument

### Removed

- Hand-rolled prompt fallbacks (`prompt`, `prompt_with_default`, hand-rolled
  `confirm`) inside `login.rs` and `deploy/approve.rs`, refactored to a thin
  `prompts::*` wrapper around [`inquire`].

## 0.6.2 - 2026-04-09

### Other

- Added `tvc` to `rust-sdk` release version group

## 0.1.0-alpha.1 - 2026-04-03

### Added

- Initial alpha release of the TVC CLI
- `tvc login` for Turnkey authentication
- `tvc app init/create` for app management
- `tvc deploy init/create/approve` for deployment workflows
- `tvc deploy status` for checking deployment status
