//! Utilities for Turnkey Verifiable Cloud applications.
//!
//! The [`fake`] module generates fake QuorumOS manifests for tests. It must
//! not be used in production.

#![deny(missing_docs)]

pub mod fake;

pub use fake::{FakeManifestBuilder, fake_manifest, fake_manifest_envelope, fake_member};
