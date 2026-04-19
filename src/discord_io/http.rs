//! Transport-abstracted HTTP client for the Discord REST API.
//!
//! All outbound HTTP calls go through [`DiscordHttpClient`] so that auth
//! headers, rate-limit back-off, and error handling live in one place.
//! The beet `Request` / `Response` types are an implementation detail —
//! swapping HTTP backends only requires touching this module.
//!
//! ## Usage
//!
//! Every API call is modelled as a type that implements
//! [`IntoDiscordRequest`].  The single entry-point is
//! [`DiscordHttpClient::send`]:
//!
//! ```ignore
//! let msg = CreateMessage::new(channel_id).content("Hello!");
//! let created: Message = http.send(msg).await?;
//! ```

use crate::prelude::*;
use async_lock::Mutex;
use beet::core::time_ext;
use beet::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;
use tracing::warn;
use twilight_model::channel::message::Message;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const BASE_URL: &str = "https://discord.com/api/v10";
const USER_AGENT: &str =
	"BeetFramework (https://github.com/mrchantey/beet, 0.1)";

// ---------------------------------------------------------------------------
// Rate-limit tracker (per-bucket)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct BucketState {
	remaining: u32,
	resets_at: Instant,
}

#[derive(Debug, Default, Clone)]
struct RateLimiter {
	/// Route-key → bucket id mapping.
	route_buckets: HashMap<String, String>,
	/// Bucket id → state.
	buckets: HashMap<String, BucketState>,
	/// Global rate-limit: if set, no requests may be sent until this instant.
	global_until: Option<Instant>,
}

impl RateLimiter {
	fn new() -> Self {
		Self {
			route_buckets: HashMap::new(),
			buckets: HashMap::new(),
			global_until: None,
		}
	}

	/// Returns how long we should wait before sending a request on `route_key`,
	/// or `None` if we can send immediately.
	fn delay_for(&self, route_key: &str) -> Option<Duration> {
		// Global rate limit takes priority.
		if let Some(until) = self.global_until {
			let now = Instant::now();
			if until > now {
				return Some(until - now);
			}
		}

		let bucket_id = self.route_buckets.get(route_key)?;
		let state = self.buckets.get(bucket_id)?;

		if state.remaining == 0 {
			let now = Instant::now();
			if state.resets_at > now {
				return Some(state.resets_at - now);
			}
		}

		None
	}

	/// Update internal state from response headers.
	fn update(&mut self, route_key: &str, info: &RateLimitInfo) {
		if info.is_global {
			if let Some(reset_after) = info.reset_after {
				self.global_until =
					Some(Instant::now() + Duration::from_secs_f64(reset_after));
			}
		}

		if let Some(ref bucket) = info.bucket {
			self.route_buckets
				.insert(route_key.to_string(), bucket.clone());

			let reset_instant = if let Some(reset_after) = info.reset_after {
				Instant::now() + Duration::from_secs_f64(reset_after)
			} else {
				Instant::now() + Duration::from_secs(1)
			};

			self.buckets.insert(bucket.clone(), BucketState {
				remaining: info.remaining.unwrap_or(1),
				resets_at: reset_instant,
			});
		}
	}
}

// ---------------------------------------------------------------------------
// Parse rate-limit headers from response parts
// ---------------------------------------------------------------------------

fn parse_rate_limit_headers(parts: &ResponseParts) -> RateLimitInfo {
	let remaining = parts
		.headers
		.first_raw("x-ratelimit-remaining")
		.and_then(|s: &str| s.parse::<u32>().ok());

	let reset_at = parts
		.headers
		.first_raw("x-ratelimit-reset")
		.and_then(|s: &str| s.parse::<f64>().ok());

	let reset_after = parts
		.headers
		.first_raw("x-ratelimit-reset-after")
		.and_then(|s: &str| s.parse::<f64>().ok());

	let bucket = parts
		.headers
		.first_raw("x-ratelimit-bucket")
		.map(|s: &str| s.to_string());

	let is_global = parts
		.headers
		.first_raw("x-ratelimit-global")
		.map(|s: &str| s == "true")
		.unwrap_or(false);

	RateLimitInfo {
		remaining,
		reset_at,
		reset_after,
		bucket,
		is_global,
	}
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum HttpError {
	/// Non-success status from Discord.
	Api {
		status: StatusCode,
		body: String,
		route: String,
	},
	/// Transport / network error.
	Transport(String),
	/// Serialisation error.
	Serde(String),
}

impl std::fmt::Display for HttpError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			HttpError::Api {
				status,
				body,
				route,
			} => {
				write!(f, "Discord API error {} on {}: {}", status, route, body)
			}
			HttpError::Transport(e) => write!(f, "HTTP transport error: {}", e),
			HttpError::Serde(e) => write!(f, "Serialisation error: {}", e),
		}
	}
}

impl std::error::Error for HttpError {}

impl From<JsonError> for HttpError {
	fn from(e: JsonError) -> Self { HttpError::Serde(e.0) }
}


// ---------------------------------------------------------------------------
// DiscordHttpClient
// ---------------------------------------------------------------------------

/// A thin, rate-limit–aware HTTP client for the Discord REST API.
///
/// Cheap to clone (internals are behind `Arc`).
///
/// ## Usage
///
/// All API calls go through [`send`](Self::send):
///
/// ```ignore
/// let msg = CreateMessage::new(channel_id).content("Hello!");
/// let created: Message = http.send(msg).await?;
/// ```
#[derive(Clone, Component, Get)]
pub struct DiscordHttpClient {
	token: String,
	limiter: Arc<Mutex<RateLimiter>>,
}

impl DiscordHttpClient {
	/// Create a new client with the given bot token.
	pub fn new(token: impl Into<String>) -> Self {
		Self {
			token: token.into(),
			limiter: Arc::new(Mutex::new(RateLimiter::new())),
		}
	}

	// ------------------------------------------------------------------
	// Public: the single send method
	// ------------------------------------------------------------------

	/// Send any request that implements [`IntoDiscordRequest`].
	///
	/// Auth and user-agent headers are attached automatically. Rate-limit
	/// back-off and retry logic are handled internally.
	///
	/// ```ignore
	/// // Create a message
	/// let msg: Message = http.send(
	///     CreateMessage::new(channel_id).content("hello")
	/// ).await?;
	///
	/// // Trigger typing
	/// http.send(CreateTypingTrigger::new(channel_id)).await?;
	///
	/// // Get guild info
	/// let guild: Guild = http.send(GetGuild::new(guild_id)).await?;
	/// ```
	pub async fn send<R: IntoDiscordRequest>(
		&self,
		request: R,
	) -> Result<R::Output, HttpError> {
		let req = request.into_discord_request()?;
		let bytes = self.raw_request(&req).await?;
		R::parse_response(&bytes).map_err(Into::into)
	}

	// ------------------------------------------------------------------
	// Higher-level helpers (compose multiple requests)
	// ------------------------------------------------------------------

	/// Count messages in a channel by paginating backwards. Caps at 10 000.
	pub async fn count_messages(
		&self,
		channel_id: Id<ChannelMarker>,
	) -> Result<usize, HttpError> {
		let mut count = 0usize;
		let mut before: Option<Id<twilight_model::id::marker::MessageMarker>> =
			None;
		let max_pages = 100;

		for _ in 0..max_pages {
			let mut req = GetChannelMessages::new(channel_id).limit(100);
			if let Some(b) = before {
				req = req.before(b);
			}

			let messages: Vec<Message> = self.send(req).await?;

			if messages.is_empty() {
				break;
			}

			count += messages.len();
			before = messages.last().map(|m| m.id);

			if messages.len() < 100 {
				break;
			}
		}

		Ok(count)
	}

	/// Get the very first message ever sent in a channel.
	pub async fn get_first_message(
		&self,
		channel_id: Id<ChannelMarker>,
	) -> Result<Message, HttpError> {
		let messages: Vec<Message> = self
			.send(
				GetChannelMessages::new(channel_id)
					.after(Id::new(1))
					.limit(1),
			)
			.await?;

		messages.into_iter().next().ok_or_else(|| HttpError::Api {
			status: StatusCode::NOT_FOUND,
			body: "No messages found in this channel.".to_string(),
			route: format!("GET /channels/{}/messages", channel_id),
		})
	}

	// ------------------------------------------------------------------
	// Internal: build a beet Request with auth + user-agent
	// ------------------------------------------------------------------

	fn build_base_request(&self, method: HttpMethod, url: &str) -> Request {
		let mut req = Request::new(method, url);
		req.headers
			.set_raw("authorization", format!("Bot {}", self.token));
		req.headers.set_raw("user-agent", USER_AGENT);
		req
	}

	// ------------------------------------------------------------------
	// Internal: low-level dispatch with rate-limit handling
	// ------------------------------------------------------------------

	/// Execute a [`DiscordRequest`] with rate-limit back-off and retry.
	///
	/// Returns the raw response bytes on success.
	async fn raw_request(
		&self,
		req: &DiscordRequest,
	) -> Result<Vec<u8>, HttpError> {
		let max_retries = 5;
		let route_key = &req.route_key;

		for attempt in 0..=max_retries {
			// Pre-request: wait if the rate limiter says so.
			{
				let limiter = self.limiter.lock().await;
				if let Some(delay) = limiter.delay_for(route_key) {
					let delay = delay.min(Duration::from_secs(60));
					drop(limiter);
					debug!(
						route = route_key.as_str(),
						delay_ms = delay.as_millis() as u64,
						"rate-limit pre-emptive backoff"
					);
					time_ext::sleep(delay).await;
				}
			}

			let url =
				format!("{}/{}", BASE_URL, req.path.trim_start_matches('/'));

			let http_req = match &req.body {
				RequestBody::None => self.build_base_request(req.method, &url),
				RequestBody::Json(value) => {
					let base = self.build_base_request(req.method, &url);
					base.with_json_body(value)
						.map_err(|e| HttpError::Serde(e.to_string()))?
				}
				RequestBody::Raw { content_type, data } => {
					let mut base = self.build_base_request(req.method, &url);
					base.headers.set_raw("content-type", content_type);
					base.with_body(data.clone())
				}
			};

			let resp = http_req
				.send()
				.await
				.map_err(|e| HttpError::Transport(e.to_string()))?;

			let status = resp.status();
			let rl_info = parse_rate_limit_headers(resp.response_parts());

			// Update the limiter regardless of status.
			{
				let mut limiter = self.limiter.lock().await;
				limiter.update(route_key, &rl_info);
			}

			if status == StatusCode::TOO_MANY_REQUESTS {
				let retry_after = rl_info.reset_after.unwrap_or(1.0);
				let delay = Duration::from_secs_f64(retry_after.min(60.0));
				warn!(
					route = route_key.as_str(),
					attempt,
					retry_after_s = retry_after,
					global = rl_info.is_global,
					"rate-limited by Discord, backing off"
				);

				if rl_info.is_global {
					let mut limiter = self.limiter.lock().await;
					limiter.global_until = Some(Instant::now() + delay);
				}

				if attempt < max_retries {
					time_ext::sleep(delay).await;
					continue;
				}
			}

			let resp_bytes = resp
				.bytes()
				.await
				.map_err(|e: BevyError| HttpError::Transport(e.to_string()))?;

			if status.is_ok() {
				return Ok(resp_bytes.to_vec());
			}

			let body_str = String::from_utf8_lossy(&resp_bytes).to_string();
			return Err(HttpError::Api {
				status,
				body: body_str,
				route: route_key.to_string(),
			});
		}

		Err(HttpError::Api {
			status: StatusCode::TOO_MANY_REQUESTS,
			body: "rate-limited after max retries".to_string(),
			route: route_key.to_string(),
		})
	}
}

impl std::fmt::Debug for DiscordHttpClient {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("DiscordHttpClient")
			.field("token", &"<redacted>")
			.finish()
	}
}
