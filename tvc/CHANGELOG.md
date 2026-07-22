# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.12.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.11.0...tvc-v0.12.0) - 2026-07-22

### Added

- *(tvc)* add outcomes
- *(tvc)* add outcomes
- *(tvc)* add outcomes
- *(tvc)* [**breaking**] add versioned operator registry
- *(tvc)* [**breaking**] reuse operator by default in app create
- *(tvc)* add hosted quorum key create command
- *(tvc)* add json output shell
- *(tvc)* add json output shell
- *(tvc)* add json output shell
- *(tvc)* add json output shell
- *(tvc)* add json output shell
- *(tvc)* add json output
- *(tvc)* route deploy debug-logs output through Shell (TVC-116)
- *(tvc)* route command output through Shell (TVC-116)
- *(tvc)* add json output shell plumbing

### Fixed

- *(tvc)* correct deploy create next-steps hints

### Other

- *(tvc)* [**breaking**] explicitly rename local key commands
- Remove generics that aren't necessary
- Merge pull request #198 from tkhq/richard/tvc-124-tvc-deploy-create-next-steps-output-include-operator-id-drop

## [0.11.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.10.0...tvc-v0.11.0) - 2026-07-15

### Added

- *(tvc)* [**breaking**] --operator-seed takes the seed value; add --operator-seed-path
- *(tvc)* validate app-id at deploy create
- *(tvc)* seed deploy init config from an existing deployment

### Other

- Merge pull request #184 from tkhq/am/feat/remove-delete-org-profiles
- Update to qos 0.12.1 and default to it for deploys
- Add ticket to TODO

## [0.10.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.9.0...tvc-v0.10.0) - 2026-07-07

### Other

- add debug_mode bool to straggling commands

## [0.9.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.8.0...tvc-v0.9.0) - 2026-07-03

### Added

- Added `tvc deploy debug-logs` to fetch deployment debug logs ([#169](https://github.com/tkhq/rust-sdk/pull/169)).

### Fixed

- Clarified `tvc login` API key and operator key labels, dashboard v2 instructions, and environment-specific dashboard links.

## [0.8.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.7.0...tvc-v0.8.0) - 2026-06-30

### Added

- Added debug mode for app and deployment intents ([#132](https://github.com/tkhq/rust-sdk/pull/132)).
- Added the `tvc app list` command.
- Added the `tvc deploy post-share` command.
- Added interactive prompts with a non-interactive guard for CLI commands.
- Added egress visibility to deployment provisioning details.
- `tvc deploy provisioning-details` now includes PCR16 and PCR17 in its
  attestation summary output with manifest/key commitment labels.

### Changed

- **Breaking:** Renamed the `external_connectivity` config field to `enable_egress`
  to match the API.
- The login API URL now defaults to production.
- Deployment approval, provisioning, and share re-encryption flows now parse
  versioned QOS manifests and manifest envelopes.
- Provisioning verification now checks manifest envelope approvals, PCR0-PCR3,
  and the PCR16 setup manifest/key commitment against the approved manifest
  hash.
- Share re-encryption now signs the versioned manifest envelope hash for share
  approvals.

### Fixed

- Fixed the TVC CLI ([#155](https://github.com/tkhq/rust-sdk/pull/155)).

## [0.7.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.6.2...tvc-v0.7.0) - 2026-05-19

### Added

- Added offline quorum-key generation and share re-encryption commands for TVC provisioning workflows.
- Added `tvc deploy provisioning-details` to display deployment provisioning details in a human-readable format.
- Added non-interactive `tvc deploy create` inputs through flags and environment variables for CI workflows.
- Added TVC app and deployment lifecycle commands for deleting, restoring, and setting the live deployment.
- Added support for custom app share sets during app creation and approval.

### Changed

- **Breaking:** Updated TVC CLI commands to use named flags and consistent environment variable names.

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
