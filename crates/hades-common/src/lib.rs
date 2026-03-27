//! # hades-common
//!
//! Shared types, errors, and constants for the Hades Messaging protocol.
//! This crate contains foundational primitives used across all other crates.

pub mod constants;
pub mod error;
pub mod types;

pub use error::HadesError;
pub use types::*;
