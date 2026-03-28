//! # hades-onion
//!
//! Onion routing layer for Hades Messaging.
//!
//! Provides Tor-inspired multi-hop routing to hide client IP addresses
//! from the relay server. Each circuit passes through 3 relay nodes,
//! with layered encryption peeled at each hop.
//!
//! ## Components
//!
//! - **Circuit**: Multi-hop encrypted tunnel
//! - **Onion Encrypt**: Layered encryption/decryption
//! - **Cell**: Fixed-size transport cells
//! - **Relay Node**: Individual hop in a circuit
//! - **Guard**: Guard node selection and rotation

pub mod bridge_rotation;
pub mod cell;
pub mod circuit;
pub mod cover_traffic;
pub mod guard;
pub mod onion_encrypt;
pub mod pluggable_transport;
pub mod relay_node;
pub mod arti_client;
