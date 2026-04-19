//! Gateway (WebSocket) transport for the Discord API.
//!
//! This module owns the WebSocket connection lifecycle:
//!   - connect → receive HELLO → send IDENTIFY
//!   - background heartbeat task
//!   - sequence number + session_id tracking
//!   - automatic reconnect + RESUME on disconnect
//!   - gateway send rate limiting (120 events / 60s)
//!
//! The rest of the codebase consumes a stream of [`GatewayEvent`] values
//! without ever touching the underlying WebSocket transport directly — when
//! we swap transports we only need to touch this file.

use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use async_channel::Receiver;
use async_channel::Sender;
use async_channel::bounded;
use async_lock::Mutex;
use beet::core::async_ext;
use beet::core::time_ext;
use beet::net::prelude::StreamExt;
use beet::net::prelude::sockets::CloseFrame;
use beet::net::prelude::sockets::Message;
use beet::net::prelude::sockets::Socket;
use beet::net::prelude::sockets::SocketRead;
use beet::net::prelude::sockets::SocketWrite;
use beet::prelude::BevyError;

use futures_lite::future::race;
use serde_json::json;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;

use crate::prelude::*;
use twilight_model::gateway::Intents;
use twilight_model::gateway::OpCode;
use twilight_model::gateway::event::DispatchEvent;
use twilight_model::gateway::event::GatewayEvent;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEFAULT_GATEWAY_URL: &str =
	"wss://gateway.discord.gg/?v=10&encoding=json";

/// Discord allows at most 120 gateway sends per 60 seconds.
const SEND_BUDGET_MAX: u32 = 120;
const SEND_BUDGET_WINDOW: Duration = Duration::from_secs(60);

/// Maximum number of reconnect attempts before giving up for a while.
const MAX_RECONNECT_ATTEMPTS: u32 = 8;

// ---------------------------------------------------------------------------
// Gateway send rate limiter
// ---------------------------------------------------------------------------

/// Simple sliding-window rate limiter for outbound gateway messages.
struct SendRateLimiter {
	/// Timestamps of recent sends (ring buffer style — we just keep the
	/// window's worth).
	timestamps: Vec<Instant>,
	budget: u32,
	window: Duration,
}

impl SendRateLimiter {
	fn new(budget: u32, window: Duration) -> Self {
		Self {
			timestamps: Vec::with_capacity(budget as usize),
			budget,
			window,
		}
	}

	/// Returns how long the caller must wait before the next send is allowed,
	/// or `None` if a send is allowed immediately.
	fn delay(&self) -> Option<Duration> {
		let now = Instant::now();
		let cutoff = now - self.window;

		// Count sends still within the window.
		let recent =
			self.timestamps.iter().filter(|&&t| t > cutoff).count() as u32;

		if recent < self.budget {
			return None;
		}

		// Find the oldest timestamp still in the window.
		let oldest = self
			.timestamps
			.iter()
			.filter(|&&t| t > cutoff)
			.copied()
			.min()?;

		// We can send again once `oldest` ages out of the window.
		let reset_at = oldest + self.window;
		if reset_at > now {
			Some(reset_at - now)
		} else {
			None
		}
	}

	/// Record that a send just happened.
	fn record(&mut self) {
		let now = Instant::now();
		let cutoff = now - self.window;
		self.timestamps.retain(|&t| t > cutoff);
		self.timestamps.push(now);
	}
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Options for connecting to the Discord gateway.
#[derive(Debug, Clone)]
pub struct GatewayConfig {
	pub token: String,
	/// Gateway intents bitflags (strongly typed).
	pub intents: Intents,
	/// Optional shard info: `[shard_id, num_shards]`.
	pub shard: Option<[u32; 2]>,
}

// ---------------------------------------------------------------------------
// Internal session state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct SessionState {
	/// From the READY event.
	session_id: Option<String>,
	/// Resume URL provided by Discord in the READY event.
	resume_gateway_url: Option<String>,
	/// Monotonically increasing sequence counter.
	sequence: Option<u64>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Start the gateway connection and return a receiver of [`GatewayEvent`]s.
///
/// The returned `GatewayHandle` can be used to send messages on the gateway
/// (e.g. request guild members, update presence).  The background tasks will
/// keep running until the handle is dropped or an unrecoverable error occurs.
pub struct GatewayHandle {
	/// Send arbitrary JSON payloads on the gateway (rate-limited).
	#[allow(dead_code)]
	pub sender: Sender<serde_json::Value>,
	/// Receive typed events.
	pub events: Receiver<GatewayEvent>,
}

impl GatewayConfig {
	/// Connect to the Discord gateway, returning a [`GatewayHandle`].
	///
	/// This spawns background tasks for:
	///   - reading from the WebSocket and parsing events
	///   - heartbeating at the interval Discord tells us
	///   - reconnecting + resuming on disconnects
	///   - rate-limiting outbound sends
	pub async fn connect(self) -> Result<GatewayHandle, String> {
		let (event_tx, event_rx) = bounded::<GatewayEvent>(256);
		let (send_tx, send_rx) = bounded::<serde_json::Value>(64);

		async_ext::spawn(gateway_driver(self, event_tx, send_rx)).detach();

		Ok(GatewayHandle {
			sender: send_tx,
			events: event_rx,
		})
	}
}


// ---------------------------------------------------------------------------
// The main driver loop (runs in a spawned task)
// ---------------------------------------------------------------------------

async fn gateway_driver(
	config: GatewayConfig,
	event_tx: Sender<GatewayEvent>,
	send_rx: Receiver<serde_json::Value>,
) {
	let session = Arc::new(Mutex::new(SessionState::default()));
	let mut reconnect_attempts: u32 = 0;

	loop {
		let url = DEFAULT_GATEWAY_URL.to_string();

		// Append query params if the resume URL doesn't already have them.
		let url = if url.contains("v=10") {
			url
		} else if url.contains('?') {
			format!("{}&v=10&encoding=json", url)
		} else {
			format!("{}?v=10&encoding=json", url)
		};

		info!(url = %url, "connecting to Discord gateway");

		// Check if we're attempting to resume this session.
		let attempting_resume = {
			let s = session.lock().await;
			s.session_id.is_some() && s.sequence.is_some()
		};

		let socket_result = Socket::connect(&url).await;

		let socket = match socket_result {
			Ok(s) => {
				reconnect_attempts = 0;
				s
			}
			Err(e) => {
				error!(error = %e, "failed to connect to gateway");

				// If we were trying to resume and it failed, clear session
				// state so we fall back to IDENTIFY on the next attempt.
				if attempting_resume {
					warn!(
						"resume failed, clearing session state to re-identify"
					);
					let mut s = session.lock().await;
					s.session_id = None;
					s.sequence = None;
					s.resume_gateway_url = None;
				}

				reconnect_attempts += 1;
				if reconnect_attempts > MAX_RECONNECT_ATTEMPTS {
					error!("exceeded max reconnect attempts, giving up");
					return;
				}
				let backoff = backoff_delay(reconnect_attempts);
				warn!(
					delay_ms = backoff.as_millis() as u64,
					attempt = reconnect_attempts,
					"backing off before reconnect"
				);
				time_ext::sleep(backoff).await;
				continue;
			}
		};

		info!("WebSocket connected");

		let (ws_write, mut ws_read) = socket.split();
		let ws_write = Arc::new(Mutex::new(ws_write));
		let rate_limiter = Arc::new(Mutex::new(SendRateLimiter::new(
			SEND_BUDGET_MAX,
			SEND_BUDGET_WINDOW,
		)));

		// ------------------------------------------------------------------
		// 1.  Read HELLO and extract heartbeat_interval
		// ------------------------------------------------------------------
		let heartbeat_interval =
			match read_hello_from_stream(&mut ws_read).await {
				Ok(interval) => interval,
				Err(e) => {
					error!(error = %e, "failed to read HELLO from gateway");
					reconnect_attempts += 1;
					let backoff = backoff_delay(reconnect_attempts);
					time_ext::sleep(backoff).await;
					continue;
				}
			};

		info!(interval_ms = heartbeat_interval, "received HELLO");

		// ------------------------------------------------------------------
		// 2.  Send IDENTIFY or RESUME
		// ------------------------------------------------------------------
		let should_resume = {
			let s = session.lock().await;
			s.session_id.is_some() && s.sequence.is_some()
		};

		if should_resume {
			let s = session.lock().await;
			let resume = json!({
				"op": OpCode::Resume,
				"d": {
					"token": config.token,
					"session_id": s.session_id.as_ref().unwrap(),
					"seq": s.sequence.unwrap(),
				}
			});
			drop(s);
			if let Err(e) =
				rate_limited_send(&ws_write, &rate_limiter, &resume).await
			{
				error!(error = %e, "failed to send RESUME");
				reconnect_attempts += 1;
				let backoff = backoff_delay(reconnect_attempts);
				time_ext::sleep(backoff).await;
				continue;
			}
			info!("sent RESUME");
		} else {
			let mut identify = json!({
				"op": OpCode::Identify,
				"d": {
					"token": config.token,
					"properties": {
						"os": "linux",
						"browser": "rust-bot",
						"device": "rust-bot"
					},
					"intents": config.intents,
				}
			});

			if let Some(shard) = config.shard {
				identify["d"]["shard"] = json!([shard[0], shard[1]]);
			}

			if let Err(e) =
				rate_limited_send(&ws_write, &rate_limiter, &identify).await
			{
				error!(error = %e, "failed to send IDENTIFY");
				reconnect_attempts += 1;
				let backoff = backoff_delay(reconnect_attempts);
				time_ext::sleep(backoff).await;
				continue;
			}
			info!("sent IDENTIFY");
		}

		// ------------------------------------------------------------------
		// 3.  Spawn heartbeat task
		// ------------------------------------------------------------------
		let hb_write = Arc::clone(&ws_write);
		let hb_session = Arc::clone(&session);
		let hb_rate_limiter = Arc::clone(&rate_limiter);
		let (hb_cancel_tx, hb_cancel_rx) = bounded::<()>(1);

		let heartbeat_handle = async_ext::spawn(async move {
			// Discord says we should send the first heartbeat after
			// `heartbeat_interval * jitter` where jitter ∈ [0, 1).
			let jitter = rand::random::<f64>();
			let first_delay = Duration::from_millis(
				(heartbeat_interval as f64 * jitter) as u64,
			);
			let cancelled = race(
				async {
					time_ext::sleep(first_delay).await;
					false
				},
				async {
					let _ = hb_cancel_rx.recv().await;
					true
				},
			)
			.await;
			if cancelled {
				return;
			}

			let interval_dur = Duration::from_millis(heartbeat_interval);
			loop {
				let cancelled = race(
					async {
						time_ext::sleep(interval_dur).await;
						false
					},
					async {
						let _ = hb_cancel_rx.recv().await;
						true
					},
				)
				.await;
				if cancelled {
					debug!("heartbeat task cancelled");
					return;
				}

				let seq = {
					let s = hb_session.lock().await;
					s.sequence
				};
				let heartbeat = json!({"op": OpCode::Heartbeat, "d": seq});

				if let Err(e) =
					rate_limited_send(&hb_write, &hb_rate_limiter, &heartbeat)
						.await
				{
					warn!(error = %e, "heartbeat send failed, stopping heartbeat task");
					return;
				}
				debug!("sent heartbeat (seq={:?})", seq);
			}
		});

		// ------------------------------------------------------------------
		// 4.  Main read loop
		// ------------------------------------------------------------------
		let disconnect_reason = read_loop(
			&mut ws_read,
			&ws_write,
			&rate_limiter,
			&event_tx,
			&session,
			&config,
			&send_rx,
		)
		.await;

		// ------------------------------------------------------------------
		// 5.  Cleanup — cancel heartbeat, decide whether to reconnect
		// ------------------------------------------------------------------
		let _ = hb_cancel_tx.send(()).await;
		drop(heartbeat_handle);

		// Try to close the WS gracefully.
		{
			let mut w = ws_write.lock().await;
			let _ = w.send(Message::Close(None)).await;
		}

		match disconnect_reason {
			DisconnectReason::ShouldResume => {
				info!("will attempt RESUME");
				// session state is preserved; loop will send RESUME.
			}
			DisconnectReason::ShouldReidentify => {
				info!("session invalidated, will re-IDENTIFY");
				let mut s = session.lock().await;
				s.session_id = None;
				s.sequence = None;
				s.resume_gateway_url = None;
			}
			DisconnectReason::Fatal => {
				error!("fatal gateway error, shutting down");
				return;
			}
			DisconnectReason::EventChannelClosed => {
				info!("event channel closed, shutting down gateway driver");
				return;
			}
		}

		reconnect_attempts += 1;
		if reconnect_attempts > MAX_RECONNECT_ATTEMPTS {
			error!("exceeded max reconnect attempts, giving up");
			return;
		}
		let backoff = backoff_delay(reconnect_attempts);
		warn!(
			delay_ms = backoff.as_millis() as u64,
			attempt = reconnect_attempts,
			"reconnecting after backoff"
		);
		time_ext::sleep(backoff).await;
	}
}

// ---------------------------------------------------------------------------
// Disconnect reason
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum DisconnectReason {
	ShouldResume,
	ShouldReidentify,
	Fatal,
	EventChannelClosed,
}

// ---------------------------------------------------------------------------
// Read loop
// ---------------------------------------------------------------------------

async fn read_loop(
	ws_read: &mut SocketRead,
	ws_write: &Arc<Mutex<SocketWrite>>,
	rate_limiter: &Arc<Mutex<SendRateLimiter>>,
	event_tx: &Sender<GatewayEvent>,
	session: &Arc<Mutex<SessionState>>,
	_config: &GatewayConfig,
	send_rx: &Receiver<serde_json::Value>,
) -> DisconnectReason {
	loop {
		enum Sel {
			Send(serde_json::Value),
			Ws(Option<Result<Message, BevyError>>),
		}

		// When send_rx is closed (sender dropped), fall through to the ws branch only.
		let sel = race(
			async {
				match send_rx.recv().await {
					Ok(payload) => Sel::Send(payload),
					Err(_) => Sel::Ws(futures_lite::future::pending().await),
				}
			},
			async { Sel::Ws(ws_read.next().await) },
		)
		.await;

		match sel {
			// Outbound sends from the bot logic (e.g. update presence).
			Sel::Send(payload) => {
				if let Err(e) =
					rate_limited_send(ws_write, rate_limiter, &payload).await
				{
					warn!(error = %e, "failed to send user payload on gateway");
				}
			}

			// Inbound messages from Discord.
			Sel::Ws(msg) => {
				let msg = match msg {
					Some(Ok(m)) => m,
					Some(Err(e)) => {
						warn!(error = %e, "WebSocket read error");
						return DisconnectReason::ShouldResume;
					}
					None => {
						info!("WebSocket stream ended");
						return DisconnectReason::ShouldResume;
					}
				};

				match msg {
					Message::Text(text) => {
						// Pre-parse to extract sequence number before full deser.
						// We still need the raw payload for sequence tracking.
						if let Ok(envelope) =
							serde_json::from_str::<GatewayPayload>(&text)
						{
							if let Some(s) = envelope.s {
								let mut sess = session.lock().await;
								sess.sequence = Some(s);
							}
						}

						let event = match parse_gateway_event(&text) {
							Ok(ev) => ev,
							Err(e) => {
								warn!(error = %e, "failed to parse gateway event");
								continue;
							}
						};

						// Handle session-relevant events internally.
						match &event {
							GatewayEvent::Dispatch(
								_,
								DispatchEvent::Ready(ready),
							) => {
								let mut sess = session.lock().await;
								sess.session_id =
									Some(ready.session_id.clone());
								sess.resume_gateway_url =
									Some(ready.resume_gateway_url.clone());
								info!(
									session_id = %ready.session_id,
									user = %ready.user.name,
									"gateway READY"
								);
							}

							GatewayEvent::Heartbeat => {
								// Discord is asking us to heartbeat immediately (op 1).
								let seq = {
									let s = session.lock().await;
									s.sequence
								};
								let heartbeat =
									json!({"op": OpCode::Heartbeat, "d": seq});
								if let Err(e) = rate_limited_send(
									ws_write,
									rate_limiter,
									&heartbeat,
								)
								.await
								{
									warn!(error = %e, "failed to send requested heartbeat");
								}
								debug!("sent requested heartbeat");
								// Don't forward to bot — it's internal plumbing.
								continue;
							}

							GatewayEvent::HeartbeatAck => {
								debug!("heartbeat acknowledged");
							}

							GatewayEvent::Reconnect => {
								info!("gateway requested reconnect (op 7)");
								return DisconnectReason::ShouldResume;
							}

							GatewayEvent::InvalidateSession(resumable) => {
								warn!(resumable, "session invalidated (op 9)");
								if *resumable {
									time_ext::sleep(Duration::from_secs(2))
										.await;
									return DisconnectReason::ShouldResume;
								} else {
									time_ext::sleep(Duration::from_secs(3))
										.await;
									return DisconnectReason::ShouldReidentify;
								}
							}

							GatewayEvent::Hello(_) => {
								// Hello is handled earlier in the connection setup;
								// if we see it here it's unexpected but harmless.
								debug!("unexpected HELLO in read loop");
							}

							_ => {}
						}

						// Forward to bot.
						if event_tx.send(event).await.is_err() {
							info!("event channel closed by consumer");
							return DisconnectReason::EventChannelClosed;
						}
					}

					Message::Close(frame) => {
						let code = frame.as_ref().map(|f| f.code);
						warn!(close_code = ?code, "WebSocket closed by server");

						if let Some(CloseFrame { code: raw, .. }) = frame {
							let action = CloseAction::from_code(raw);
							match action {
								CloseAction::Fatal => {
									error!(
										close_code = raw,
										"fatal gateway close code"
									);
									return DisconnectReason::Fatal;
								}
								CloseAction::Reidentify => {
									warn!(
										close_code = raw,
										"session invalidated by close code"
									);
									return DisconnectReason::ShouldReidentify;
								}
								CloseAction::Resume => {
									info!(
										close_code = raw,
										"resumable close code"
									);
									return DisconnectReason::ShouldResume;
								}
							}
						}

						return DisconnectReason::ShouldResume;
					}

					// Ping / Pong / Binary — ignore.
					_ => {}
				}
			}
		}
	}
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read the HELLO payload from an already-split stream reference.
async fn read_hello_from_stream(
	stream: &mut SocketRead,
) -> Result<u64, String> {
	let msg = async_ext::timeout(Duration::from_secs(30), stream.next())
		.await
		.map_err(|_| "timed out waiting for HELLO".to_string())?
		.ok_or_else(|| "stream ended before HELLO".to_string())?
		.map_err(|e| format!("WS error reading HELLO: {}", e))?;

	let text = match msg {
		Message::Text(t) => t,
		other => {
			return Err(format!(
				"expected text message for HELLO, got {:?}",
				other
			));
		}
	};

	let payload: GatewayPayload = serde_json::from_str(&text)
		.map_err(|e| format!("failed to parse HELLO: {}", e))?;

	if payload.op != OpCode::Hello {
		return Err(format!("expected op HELLO, got op {:?}", payload.op));
	}

	let interval = payload
		.d
		.as_ref()
		.and_then(|d| d.get("heartbeat_interval"))
		.and_then(|v| v.as_u64())
		.ok_or_else(|| "HELLO missing heartbeat_interval".to_string())?;

	Ok(interval)
}

/// Send a JSON payload on the WebSocket, respecting the send rate limiter.
async fn rate_limited_send(
	ws_write: &Arc<Mutex<SocketWrite>>,
	rate_limiter: &Arc<Mutex<SendRateLimiter>>,
	payload: &serde_json::Value,
) -> Result<(), String> {
	// Wait until we have budget.
	loop {
		let delay = {
			let rl = rate_limiter.lock().await;
			rl.delay()
		};
		match delay {
			Some(d) => {
				debug!(
					delay_ms = d.as_millis() as u64,
					"gateway send rate-limited, waiting"
				);
				time_ext::sleep(d).await;
			}
			None => break,
		}
	}

	// Record the send.
	{
		let mut rl = rate_limiter.lock().await;
		rl.record();
	}

	let text = serde_json::to_string(payload).map_err(|e| e.to_string())?;

	let mut w = ws_write.lock().await;
	w.send(Message::Text(text))
		.await
		.map_err(|e| format!("WS send error: {}", e))
}

/// Exponential backoff with jitter, capped at 60 s.
fn backoff_delay(attempt: u32) -> Duration {
	let base_ms = 1000u64 * 2u64.saturating_pow(attempt.min(6));
	let jitter = (rand::random::<f64>() * 0.5 + 0.75) * base_ms as f64;
	Duration::from_millis(jitter.min(60_000.0) as u64)
}
