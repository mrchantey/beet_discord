//! Lightweight gateway types borrowed from `twilight-gateway`.
//!
//! We can't depend on `twilight-gateway` directly (it pulls in `tokio`,
//! `tokio-websockets`, etc.), but we *can* re-export the pure data types
//! from `twilight-model` and adapt a few patterns from twilight-gateway
//! that only depend on those model types.
//!
//! Everything here is intentionally thin — just types, constants, and
//! conversions. The actual WebSocket transport lives in
//! [`crate::discord_io::gateway`].

use std::time::Duration;
use std::time::Instant;

// =========================================================================
// Private imports from twilight-model used within this module
// (intentionally not re-exported — callers import twilight_model directly)
// =========================================================================

use twilight_model::gateway::event::GatewayEvent;
use twilight_model::gateway::event::GatewayEventDeserializer;
use twilight_model::gateway::CloseCode;
use twilight_model::gateway::OpCode;

// =========================================================================
// Gateway payload envelope
// =========================================================================

use serde::Deserialize;
use serde::Serialize;

/// Raw gateway payload envelope.
///
/// Every message on the Discord WebSocket is wrapped in this structure.
/// Twilight handles this inside `twilight-gateway`, but since we run our
/// own transport we need the raw envelope.  The `op` field is now the
/// strongly-typed [`OpCode`] instead of a bare `u8`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayPayload {
	pub op: OpCode,
	pub d: Option<serde_json::Value>,
	pub s: Option<u64>,
	pub t: Option<String>,
}

// =========================================================================
// Close-code helpers
// =========================================================================

/// Classifies a raw WebSocket close code into an action the gateway
/// driver should take.
///
/// Uses [`CloseCode`] from twilight-model where possible, falling back
/// to a sensible default for codes Discord doesn't officially define.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseAction {
	/// Session is still alive — reconnect and send RESUME.
	Resume,
	/// Session was invalidated — reconnect and send IDENTIFY.
	Reidentify,
	/// Unrecoverable error — stop the gateway driver.
	Fatal,
}

impl CloseAction {
	/// Determine the appropriate action for a raw close code received
	/// from Discord.
	pub fn from_code(raw: u16) -> Self {
		match CloseCode::try_from(raw) {
			Ok(code) => {
				if code.can_reconnect() {
					// InvalidSequence and SessionTimedOut mean the session
					// is gone — we need to re-IDENTIFY rather than RESUME.
					match code {
						CloseCode::InvalidSequence
						| CloseCode::SessionTimedOut => Self::Reidentify,
						_ => Self::Resume,
					}
				} else {
					Self::Fatal
				}
			}
			// Unknown code — try to resume as a safe default.
			Err(_) => Self::Resume,
		}
	}
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
	use super::*;
	use twilight_model::gateway::Intents;
	use twilight_model::gateway::OpCode;

	#[test]
	fn gateway_payload_deserializes() {
		let json = r#"{"op":0,"d":null,"s":1,"t":"READY"}"#;
		let payload: GatewayPayload = serde_json::from_str(json).unwrap();
		assert_eq!(payload.op, OpCode::Dispatch);
		assert_eq!(payload.s, Some(1));
		assert_eq!(payload.t.as_deref(), Some("READY"));
	}

	#[test]
	fn gateway_payload_hello() {
		let json =
			r#"{"op":10,"d":{"heartbeat_interval":41250},"s":null,"t":null}"#;
		let payload: GatewayPayload = serde_json::from_str(json).unwrap();
		assert_eq!(payload.op, OpCode::Hello);
	}

	#[test]
	fn close_action_fatal_codes() {
		assert_eq!(CloseAction::from_code(4004), CloseAction::Fatal);
		assert_eq!(CloseAction::from_code(4010), CloseAction::Fatal);
		assert_eq!(CloseAction::from_code(4011), CloseAction::Fatal);
		assert_eq!(CloseAction::from_code(4012), CloseAction::Fatal);
		assert_eq!(CloseAction::from_code(4013), CloseAction::Fatal);
		assert_eq!(CloseAction::from_code(4014), CloseAction::Fatal);
	}

	#[test]
	fn close_action_reidentify_codes() {
		assert_eq!(CloseAction::from_code(4007), CloseAction::Reidentify);
		assert_eq!(CloseAction::from_code(4009), CloseAction::Reidentify);
	}

	#[test]
	fn close_action_resume_codes() {
		assert_eq!(CloseAction::from_code(4000), CloseAction::Resume);
		assert_eq!(CloseAction::from_code(4001), CloseAction::Resume);
		assert_eq!(CloseAction::from_code(4002), CloseAction::Resume);
		assert_eq!(CloseAction::from_code(4003), CloseAction::Resume);
		assert_eq!(CloseAction::from_code(4005), CloseAction::Resume);
		assert_eq!(CloseAction::from_code(4008), CloseAction::Resume);
	}

	#[test]
	fn close_action_unknown_code_resumes() {
		assert_eq!(CloseAction::from_code(5000), CloseAction::Resume);
		assert_eq!(CloseAction::from_code(1001), CloseAction::Resume);
	}

	#[test]
	fn intents_bitflags() {
		let intents = Intents::GUILDS
			| Intents::GUILD_MEMBERS
			| Intents::GUILD_PRESENCES
			| Intents::GUILD_MESSAGES
			| Intents::MESSAGE_CONTENT;
		assert!(intents.contains(Intents::GUILDS));
		assert!(intents.contains(Intents::GUILD_MEMBERS));
		assert!(intents.contains(Intents::GUILD_PRESENCES));
		assert!(intents.contains(Intents::GUILD_MESSAGES));
		assert!(intents.contains(Intents::MESSAGE_CONTENT));
		assert!(!intents.contains(Intents::DIRECT_MESSAGES));
	}
}

// =========================================================================
// Session (adapted from twilight-gateway/src/session.rs)
// =========================================================================

/// Gateway session information for a shard's active connection.
///
/// A session is a stateful identifier on Discord's end for running a shard.
/// It is used for maintaining an authenticated Websocket connection based on
/// an identifier. While a session is only connected to one shard, one shard
/// can have more than one session: if a shard shuts down its connection and
/// starts a new session, then the previous session will be kept alive for a
/// short time.
///
/// # Reusing sessions
///
/// Sessions are able to be reused across connections to Discord. If an
/// application's process needs to be restarted, then this session
/// information—which can be (de)serialized via serde—can be stored, the
/// application restarted, and then used again.
///
/// If the delay between disconnecting from the gateway and reconnecting isn't
/// too long and Discord hasn't invalidated the session, then the session will
/// be reused by Discord. As a result, any events that were "missed" while
/// restarting and reconnecting will be played back, meaning the application
/// won't have missed any events. If the delay has been too long, then a new
/// session will be initialized, resulting in those events being missed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Session {
	/// ID of the gateway session.
	id: Box<str>,
	/// Sequence of the most recently received gateway event.
	///
	/// The first sequence of a session is always 1.
	sequence: u64,
}

impl Session {
	/// Create new configuration for resuming a gateway session.
	pub fn new(sequence: u64, session_id: String) -> Self {
		Self {
			sequence,
			id: session_id.into_boxed_str(),
		}
	}

	/// ID of the session being resumed.
	///
	/// Session IDs are obtained by shards via sending an `Identify` command,
	/// and in return the session ID is provided via the `Ready` event.
	pub fn id(&self) -> &str { &self.id }

	/// Current sequence of the connection.
	///
	/// Number of the events that have been received during this session. A
	/// larger number typically correlates that the shard has been connected
	/// with this session for a longer time, while a smaller number typically
	/// correlates to meaning that it's been connected with this session for a
	/// shorter duration of time.
	pub fn sequence(&self) -> u64 { self.sequence }

	/// Set the sequence, returning the previous sequence.
	pub fn set_sequence(&mut self, sequence: u64) -> u64 {
		std::mem::replace(&mut self.sequence, sequence)
	}
}

// =========================================================================
// Latency (adapted from twilight-gateway/src/latency.rs)
// =========================================================================

/// Shard gateway connection latency.
///
/// Measures the difference between sending a heartbeat and receiving an
/// acknowledgement, also known as a heartbeat period. Spurious heartbeat
/// acknowledgements are ignored.
#[derive(Clone, Debug)]
pub struct Latency {
	/// Sum of recorded latencies.
	latency_sum: Duration,
	/// Number of recorded heartbeat periods.
	periods: u32,
	/// When the last heartbeat received an acknowledgement.
	received: Option<Instant>,
	/// List of most recent latencies.
	recent: [Duration; Self::RECENT_LEN],
	/// When the last heartbeat was sent.
	sent: Option<Instant>,
}

impl Latency {
	/// Number of recent latencies to store.
	const RECENT_LEN: usize = 5;

	/// Create a new instance for tracking shard latency.
	pub fn new() -> Self {
		Self {
			latency_sum: Duration::ZERO,
			periods: 0,
			received: None,
			recent: [Duration::MAX; Self::RECENT_LEN],
			sent: None,
		}
	}

	/// Average latency.
	///
	/// For example, a reasonable value for this may be between 10 to 100
	/// milliseconds depending on the network connection and physical location.
	///
	/// Returns [`None`] if no heartbeat periods have been recorded.
	pub fn average(&self) -> Option<Duration> {
		self.latency_sum.checked_div(self.periods)
	}

	/// Number of recorded heartbeat periods.
	pub fn periods(&self) -> u32 { self.periods }

	/// Most recent latencies from newest to oldest.
	pub fn recent(&self) -> &[Duration] {
		// We use the sentinel value of Duration::MAX since using
		// `Duration::ZERO` would cause tests depending on elapsed time on fast
		// CPUs to flake.
		let maybe_zero_idx = self
			.recent
			.iter()
			.position(|duration| *duration == Duration::MAX);

		&self.recent[0..maybe_zero_idx.unwrap_or(Self::RECENT_LEN)]
	}

	/// When the last heartbeat received an acknowledgement.
	pub fn received(&self) -> Option<Instant> { self.received }

	/// When the last heartbeat was sent.
	pub fn sent(&self) -> Option<Instant> { self.sent }

	/// Record that a heartbeat acknowledgement was received, completing the
	/// period.
	///
	/// The current time is subtracted against when the last heartbeat
	/// was sent to calculate the heartbeat period's latency.
	///
	/// # Panics
	///
	/// Panics if the period is already complete or has not begun.
	#[track_caller]
	pub fn record_received(&mut self) {
		debug_assert!(
			self.received.is_none(),
			"period completed multiple times"
		);

		let now = Instant::now();
		let period_latency = now - self.sent.expect("period has not begun");
		self.received = Some(now);
		self.periods += 1;

		self.latency_sum += period_latency;
		self.recent.copy_within(..Self::RECENT_LEN - 1, 1);
		self.recent[0] = period_latency;
	}

	/// Record that a heartbeat was sent, beginning a new period.
	///
	/// The current time is stored to be used in [`record_received`](Self::record_received).
	pub fn record_sent(&mut self) {
		self.received = None;
		self.sent = Some(Instant::now());
	}
}

impl Default for Latency {
	fn default() -> Self { Self::new() }
}

// =========================================================================
// parse_gateway_event (adapted from twilight-gateway/src/json.rs)
// =========================================================================

/// Parse a raw JSON gateway message into a twilight [`GatewayEvent`](GatewayEvent).
///
/// This adapts `twilight-gateway`'s JSON parsing to work without the full
/// twilight-gateway deserializer infrastructure. It uses
/// [`GatewayEventDeserializer`] from `twilight-model` directly.
pub fn parse_gateway_event(json: &str) -> Result<GatewayEvent, String> {
	let deserializer = GatewayEventDeserializer::from_json(json)
		.ok_or_else(|| "failed to pre-parse gateway event".to_string())?;

	let mut json_deserializer = serde_json::Deserializer::from_str(json);

	use serde::de::DeserializeSeed;
	deserializer
		.deserialize(&mut json_deserializer)
		.map_err(|e| format!("failed to deserialize gateway event: {}", e))
}

// =========================================================================
// Tests (continued)
// =========================================================================

#[cfg(test)]
mod tests_session {
	use super::*;

	#[test]
	fn session_new_and_getters() {
		let session = Session::new(42, "my-session-id".to_owned());
		assert_eq!(session.id(), "my-session-id");
		assert_eq!(session.sequence(), 42);
	}

	#[test]
	fn session_set_sequence() {
		let mut session = Session::new(1, String::new());
		let old = session.set_sequence(2);
		assert_eq!(old, 1);
		assert_eq!(session.sequence(), 2);

		let old = session.set_sequence(10);
		assert_eq!(old, 2);
		assert_eq!(session.sequence(), 10);
	}

	#[test]
	fn session_serde_roundtrip() {
		let session = Session::new(100, "abc123".to_owned());
		let json = serde_json::to_string(&session).unwrap();
		let deserialized: Session = serde_json::from_str(&json).unwrap();
		assert_eq!(session, deserialized);
	}
}

#[cfg(test)]
mod tests_latency {
	use super::*;

	#[test]
	fn latency_new_defaults() {
		let latency = Latency::new();
		assert_eq!(latency.periods(), 0);
		assert!(latency.average().is_none());
		assert!(latency.received().is_none());
		assert!(latency.sent().is_none());
		assert!(latency.recent().is_empty());
	}

	#[test]
	fn latency_record_period() {
		let mut latency = Latency::new();

		latency.record_sent();
		assert!(latency.sent().is_some());
		assert!(latency.received().is_none());
		assert_eq!(latency.periods(), 0);

		latency.record_received();
		assert!(latency.received().is_some());
		assert_eq!(latency.periods(), 1);
		assert_eq!(latency.recent().len(), 1);
		assert!(latency.average().is_some());
	}

	#[test]
	fn latency_recent_ordering() {
		let mut latency = Latency::new();

		// Record several periods
		for _ in 0..3 {
			latency.record_sent();
			latency.record_received();
		}

		assert_eq!(latency.periods(), 3);
		assert_eq!(latency.recent().len(), 3);
	}

	#[test]
	#[should_panic(expected = "period has not begun")]
	fn latency_record_received_without_sent() {
		let mut latency = Latency::new();
		latency.record_received();
	}
}

#[cfg(test)]
mod tests_parse {
	use super::*;

	#[test]
	fn parse_hello_event() {
		let json =
			r#"{"op":10,"d":{"heartbeat_interval":41250},"s":null,"t":null}"#;
		let event = parse_gateway_event(json).unwrap();
		match event {
			GatewayEvent::Hello(hello) => {
				assert_eq!(hello.heartbeat_interval, 41250);
			}
			other => panic!("expected Hello, got {:?}", other),
		}
	}

	#[test]
	fn parse_heartbeat_ack_event() {
		let json = r#"{"op":11,"d":null,"s":null,"t":null}"#;
		let event = parse_gateway_event(json).unwrap();
		assert!(matches!(event, GatewayEvent::HeartbeatAck));
	}

	#[test]
	fn parse_reconnect_event() {
		let json = r#"{"op":7,"d":null,"s":null,"t":null}"#;
		let event = parse_gateway_event(json).unwrap();
		assert!(matches!(event, GatewayEvent::Reconnect));
	}

	#[test]
	fn parse_invalid_session_event() {
		let json = r#"{"op":9,"d":false,"s":null,"t":null}"#;
		let event = parse_gateway_event(json).unwrap();
		assert!(matches!(event, GatewayEvent::InvalidateSession(false)));
	}

	#[test]
	fn parse_invalid_json_fails() {
		let result = parse_gateway_event("not json at all");
		assert!(result.is_err());
	}

	#[test]
	fn parse_missing_opcode_fails() {
		let json = r#"{"d":null,"s":null,"t":null}"#;
		let result = parse_gateway_event(json);
		assert!(result.is_err());
	}
}
