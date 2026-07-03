# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.9.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.8.0...tvc-v0.9.0) - 2026-07-03

### Added

- *(tvc)* add deploy debug-logs command ([#169](https://github.com/tkhq/rust-sdk/pull/169))

### Fixed

- *(tvc)* remove extra word in API key lines
- *(tvc)* clarify API key vs operator key labels in login output
- *(tvc)* derive org ID welcome link from selected environment
- *(tvc)* derive API key registration dashboard URL from environment
- *(tvc)* formatting

### Other

- *(tvc)* hoist turnkey imports to top of tests module
- *(tvc)* avoid unnecessary allocation in dashboard_base_url
- *(tvc)* update login API key instructions for dashboard v2
- Merge pull request #175 from tkhq/avi/followup

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
