#![doc = include_str!("../README.md")]

/// CLI argument parsing and top-level command dispatch.
pub mod cli;
/// Subcommand implementations for the `auth` binary.
pub mod commands;
/// Auth configuration resolution and persistence helpers.
pub mod config;
/// SSH wire-format helpers for public keys, signatures, and agent messages.
pub mod ssh;
/// Turnkey-backed signing client helpers.
pub mod turnkey;
