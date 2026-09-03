# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.15.1](https://github.com/tkhq/rust-sdk/compare/tvc-v0.15.0...tvc-v0.15.1) - 2026-09-03

### Added

- *(tvc)* add skills save command to install bundled agent skills
- *(tvc)* add tvc-deployments agent skill

### Other

- *(tvc)* rustfmt skills_save tests
- *(tvc)* point AGENTS.md at the bundled agent skills and their sync rules
- rm mention of merging imports from AGENTS.md
- *(tvc)* force use of modern file layout

## [0.15.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.14.0...tvc-v0.15.0) - 2026-08-28

### Added

- *(tvc)* add YubiKey certificate creation
- *(tvc)* add --kind yubikey to operator create
- *(tvc)* offer YubiKey operators during interactive new-org login
- *(tvc)* create organizations with a YubiKey operator record
- *(tvc)* add the YubiKey enrollment core
- *(tvc)* choose the connected YubiKey without a selector
- *(tvc)* disambiguate multiple YubiKey operators by serial
- *(tvc)* add keys refresh-yubikey to resync the registry key cache
- *(tvc)* route local share re-encryption through the operator pair port
- *(tvc)* route manifest approval through a yubikey default operator
- *(tvc)* add yubikey as a configurable default operator kind
- *(tvc)* add the YubiKey operator pair and serial resolution
- *(tvc)* add sign and key-agreement primitives to the YubiKey device boundary
- *(tvc)* add YubiKey registry, provisioning, and key management
- *(tvc)* add deploy replicas option
- *(tvc)* log in to hosted-default orgs with a backend-shaped outcome
- *(tvc)* offer registered hosted operators in app create and approve defaults
- *(tvc)* resolve the sole hosted operator when the default backend is hosted
- *(tvc)* offer operator key backup during login onboarding
- *(tvc)* add keys backup-operator-key command
- *(tvc)* collapse client-version-too-old error and instrument the CLI callstack
- *(tvc)* classify backend client-version rejection and render upgrade hint

### Fixed

- *(tvc)* Check the manifest before trying to do approval
- *(tvc)* match operator keys before app reuse
- *(tvc)* confirm certless YubiKey reprovisioning
- *(tvc)* resolve YubiKey inputs before side effects
- *(tvc)* YubiKey parity for profile deletion and operator-key messaging
- *(tvc)* fence the multi-candidate reuse prompt and pin the selection paths
- *(tvc)* explain the local-only key commands to hosted-operator orgs

### Other

- *(tvc)* stop managing YubiKey device state
- *(tvc)* cover multi-record YubiKey login selection end to end
- *(tvc)* fake-test keys refresh-yubikey and fix its save remediation
- *(tvc)* select the share-decryption backend before reading inputs
- *(tvc)* make operator resolution a Config method
- *(tvc)* type the key-agreement points and preserve qos_p256 errors
- *(tvc)* extract the Pair port for decrypt-capable operators
- *(tvc)* Remove bad `.expect`s
- *(tvc)* address remaining PR 254 feedback
- *(tvc)* followups from PR 254 review
- *(tvc)* Add qos_client as a dependency
- Merge pull request #250 from tkhq/swag/client-transport
- sync protos from tkhq/mono fd98e55aa01e (make -C proto sync/rust-sdk)
- Merge pull request #234 from tkhq/zeke/deployment-approval-prompts
- Improve tvc key file output handling
- Update qos_* versions
- *(tvc)* load config once at dispatch and pass it into commands
- *(tvc)* simplify return types
- *(tvc)* remove single-use functions
- *(tvc)* seal operator backends behind a Signer port
- *(tvc)* parse stored operator public keys into a typed composite
- Merge pull request #228 from tkhq/richard/eng-4082-ensure-tvc-cli-is-on-a-recent-enough-version

### Added

- *(tvc)* expose deployment replica count on `tvc deploy create` via the `replicas` config field and `--replicas`/`TVC_REPLICAS` override

## [0.14.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.13.1...tvc-v0.14.0) - 2026-08-05

### Added

- *(tvc)* cryptographically validate manifest approvals
- *(tvc)* add version subcommand
- *(tvc)* stamp X-TVC-CLIENT-VERSION on every API request

### Other

- Sync protos from mono and map google.protobuf.Timestamp to pbjson-types for serde support
- Merge pull request #230 from tkhq/richard/eng-4274-cryptographically-validate-manifest-approvals-before-writing
- Merge pull request #229 from tkhq/richard/parse-dont-validate
- *(tvc)* followups from PR 188
- *(tvc)* followups from PR 188
- *(tvc)* Fix docs
- *(tvc)* serde-tagged outcome vocabulary and Run trait pilot

## [0.13.1](https://github.com/tkhq/rust-sdk/compare/tvc-v0.13.0...tvc-v0.13.1) - 2026-07-29

### Other

- Merge pull request #219 from tkhq/richard/agents.md
- *(tvc)* add wallet_id to operator create output
- *(tvc)* followup agents.md updates

## [0.13.0](https://github.com/tkhq/rust-sdk/compare/tvc-v0.12.0...tvc-v0.13.0) - 2026-07-28

### Added

- *(tvc)* add deploy provision for hosted provisioning
- *(tvc)* extend create-quorum-key to accept operator-ids; also refactor to use PubKey struct
- *(tvc)* support hosted operators in deploy approve
- *(tvc)* add hosted operator create command

### Other

- *(tvc)* error taxonomy surfaced to user
- *(tvc)* type UUID-backed CLI IDs as uuid::Uuid
- *(tvc)* update AGENTS.md from related reviews
- Merge pull request #205 from tkhq/annie/tvc-191-cli-add-hosted-operator-creation-and-deploy-approval

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
