//! Typed helpers for Discord-specific metadata stored in
//! `serde_json::Map<String, serde_json::Value>` fields on `Post`, `Thread`,
//! and `Actor` components.
//!
//! Each struct uses `Option` fields so that missing keys deserialize as `None`,
//! and only `Some` values are written back into the map.

use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

// ---------------------------------------------------------------------------
// DiscordStatus enum
// ---------------------------------------------------------------------------

/// Status of a post relative to the Discord HTTP API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscordStatus {
	Sending,
	Complete,
}

// ---------------------------------------------------------------------------
// Per-domain metadata structs
// ---------------------------------------------------------------------------

/// Metadata keys stored on a [`Post`](beet::prelude::Post).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscordPostMeta {
	/// Whether this post has been sent / is being sent to Discord.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub discord_status: Option<DiscordStatus>,
	/// Discord message IDs that correspond to this post (a single post may be
	/// split across several Discord messages).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub discord_messages: Option<Vec<u64>>,
}

/// Metadata keys stored on a [`Thread`](beet::prelude::Thread).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscordThreadMeta {
	/// The Discord channel / thread ID backing this thread.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub discord_channel: Option<u64>,
}

/// Metadata keys stored on an [`Actor`](beet::prelude::Actor).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscordActorMeta {
	/// The Discord user ID that owns this actor.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub discord_user: Option<u64>,
}

// ---------------------------------------------------------------------------
// Read / write helpers
// ---------------------------------------------------------------------------

/// Extension trait for reading and writing typed metadata structs into a
/// `serde_json::Map<String, Value>`.
pub trait DiscordMeta: Sized {
	/// Deserialize `Self` from the given metadata map.
	///
	/// Keys that are absent from the map will become `None` on the struct.
	fn read_from(metadata: &Map<String, Value>) -> Self;

	/// Merge all `Some` fields of `self` into the metadata map.
	///
	/// Existing keys not covered by this struct are left untouched.
	fn write_to(&self, metadata: &mut Map<String, Value>);
}

/// Blanket-ish helper – we implement the trait for each of our three structs
/// via a macro to avoid repetition.
macro_rules! impl_discord_meta {
	($ty:ty) => {
		impl DiscordMeta for $ty {
			fn read_from(metadata: &Map<String, Value>) -> Self {
				// Build a temporary Value::Object and deserialize from it.
				let obj = Value::Object(metadata.clone());
				serde_json::from_value(obj).unwrap_or_default()
			}

			fn write_to(&self, metadata: &mut Map<String, Value>) {
				// Serialize self to a Value::Object, then merge each key.
				if let Ok(Value::Object(map)) = serde_json::to_value(self) {
					for (k, v) in map {
						metadata.insert(k, v);
					}
				}
			}
		}
	};
}

impl_discord_meta!(DiscordPostMeta);
impl_discord_meta!(DiscordThreadMeta);
impl_discord_meta!(DiscordActorMeta);
