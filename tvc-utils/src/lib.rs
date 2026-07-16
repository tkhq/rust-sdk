//! Utilities for Turnkey Verifiable Cloud applications.
//!
//! The [`verify`] module verifies the TVC trust chain — an NSM attestation
//! document and a QuorumOS manifest envelope, as attached to responses by
//! `tvc-axum` — against caller-supplied expected values, and returns the
//! enclave's ephemeral public key for verifying response signatures.
//!
//! The [`fake`] module generates fake QuorumOS manifests for tests. It must
//! not be used in production.

#![deny(missing_docs)]

pub mod fake;
pub mod verify;

pub use fake::{
    FakeManifestBuilder, fake_keyed_member, fake_manifest, fake_manifest_envelope, fake_member,
};
pub use verify::{VerificationExpectations, VerifyError, verify_attestation_and_manifest};
