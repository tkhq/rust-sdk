# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.7.1](https://github.com/tkhq/rust-sdk/compare/tvc-v0.7.0...tvc-v0.7.1) - 2026-06-11

### Added

- *(tvc)* show approval completion after quorum
- *(tvc)* debug mode in app and deployment intents ([#132](https://github.com/tkhq/rust-sdk/pull/132))
- *(tvc)* implement tvc app list command
- *(tvc)* default login API URL to prod
- *(tvc)* interactive prompts and non-interactive guard
- *(tvc)* interactive prompts and non-interactive guard
- *(tvc)* interactive prompts and non-interactive guard
- *(tvc)* interactive prompts and non-interactive guard
- *(tvc)* add deploy post-share command

### Fixed

- *(tvc)* remove manual update to changelog

### Other

- Merge pull request #159 from tkhq/richard/eng-3889-clarify-youre-done-once-you-post-approvals
- Add else message when deployment approved but quorum not reached
- Reword quorum reached message
- Make non-interactive mode explicit in tvc
- Merge pull request #156 from tkhq/zeke/rename-external-connect
- *(tvc)* refactor now-unsafe test code ([#157](https://github.com/tkhq/rust-sdk/pull/157))
- Fix env-var guard
- Update to edition 2024
- *(tvc)* fix cli ([#155](https://github.com/tkhq/rust-sdk/pull/155))
- Merge pull request #151 from tkhq/am/feat/list_apps
- *(tvc)* add unit and integration tests for app list command
- *(tvc)* update changelog for app list command
- *(tvc)* add render_app helper to list apps command
- Merge pull-request #145
- Merge pull request #141 from tkhq/richard/tvc-58-add-client-side-debug-logging-to-tvc-cli

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
