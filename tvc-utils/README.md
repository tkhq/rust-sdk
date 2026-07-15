# tvc-utils

Test utilities for Turnkey Verifiable Cloud applications.

Provides helpers to build fake QuorumOS manifest envelopes (manifest schema
v2) for tests: a one-liner `fake_manifest_envelope()` with sensible defaults
and a `FakeManifestBuilder` for configuring the namespace, manifest/share
sets, quorum key, PCRs, and pivot configuration.

**DO NOT USE IN PRODUCTION.** The generated manifests carry made-up
measurements and no approvals; they exist only so tests can exercise code
paths that consume a `VersionedManifestEnvelope`.
