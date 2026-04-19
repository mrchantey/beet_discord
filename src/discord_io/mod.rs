//! Discord I/O layer: gateway (WebSocket) transport, HTTP REST client,
//! event-handlers, and the Bevy bot wiring.
//!
//! This entire module is compiled only when the `io` feature is enabled,
//! keeping the default build to just [`crate::discord_helpers`].

mod gateway;
pub use gateway::*;
mod gateway_listener;
pub use gateway_listener::*;
mod http;
pub use http::*;
