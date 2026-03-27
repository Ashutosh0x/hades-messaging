//! # hades-proto
//!
//! Protocol message definitions for Hades Messaging wire format.
//!
//! All messages are serde-serializable for transport over WebSocket.
//! The `.proto` file in `proto/hades.proto` serves as the canonical
//! schema reference; these Rust types mirror it without requiring `protoc`.

pub mod messages;

pub use messages::*;
