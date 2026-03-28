//! # hades-relay
//!
//! Zero-knowledge relay server for Hades Messaging.
//!
//! The relay is intentionally dumb — it routes sealed envelopes between
//! clients without ever seeing plaintext, sender identity, or metadata
//! beyond circuit-level routing.
//!
//! ## Components
//!
//! - **Server**: Axum-based WebSocket server with TLS
//! - **Session**: Per-connection state machine
//! - **Router**: Circuit-based message routing
//! - **Prekey Store**: Manages uploaded prekey bundles
//! - **Rate Limiter**: Per-identity request throttling
//! - **Metrics**: Prometheus endpoint for operational monitoring

pub mod auth;
pub mod config;
pub mod message_queue;
pub mod prekey_store;
pub mod rate_limit;
pub mod router;
pub mod server;
pub mod session;
