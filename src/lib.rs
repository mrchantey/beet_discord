//! High-level entry point for the Discord bot.
//!
//! The crate is split into two layers:
//!
//! - **Always compiled:** [`discord_types`] (custom types, builders, and
//!   extension traits layered on top of `twilight-model`) and [`tw_gateway`]
//!   (lightweight gateway envelope and session types).
//! - **`io` feature only:** [`discord_io`] (WebSocket gateway, HTTP REST
//!   client, event handlers, and Bevy bot wiring).
//!
//! # Importing twilight types
//!
//! This crate does **not** re-export anything from `twilight-model`. Import
//! twilight types directly:
//!
//! ```ignore
//! use twilight_model::channel::message::Message;
//! use twilight_model::id::{Id, marker::ChannelMarker};
//! ```
//!
//! `use crate::prelude::*;` (or `use hello_discord::prelude::*;`) brings in
//! all types and extension traits *defined in this crate*.

#[cfg(feature = "io")]
pub mod discord_io;
pub mod discord_types;
pub mod beet_thread;
#[cfg(feature = "io")]
pub mod types;

pub mod prelude {
	#[cfg(feature = "io")]
	pub use crate::discord_io::*;
	pub use crate::discord_types::CommandExt;
	pub use crate::discord_types::*;
	pub use crate::beet_thread::*;
	#[cfg(feature = "io")]
	pub use crate::types::*;
	pub use twilight_model::application::interaction::Interaction;
	pub use twilight_model::channel::Channel;
	pub use twilight_model::channel::message::Message;
	pub use twilight_model::gateway::payload::incoming::GuildCreate;
	pub use twilight_model::gateway::payload::incoming::PresenceUpdate;
	pub use twilight_model::gateway::payload::incoming::Ready;
	pub use twilight_model::id::Id;
	pub use twilight_model::id::marker::ChannelMarker;
	pub use twilight_model::id::marker::UserMarker;
	pub use twilight_model::user::User;
}
