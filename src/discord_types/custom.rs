//! Types that don't exist in twilight-model.
//!
//! These are specific to our implementation and cover outbound message bodies,
//! rate-limit tracking, and the [`IntoDiscordRequest`] trait that bridges
//! typed request structs with the transport layer in [`crate::discord_io`].
//!
//! Gateway event types (including presence updates) are now provided directly
//! by `twilight-model`.

use beet::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::marker::MessageMarker;

// ---------------------------------------------------------------------------
// DiscordRequest / RequestBody
// ---------------------------------------------------------------------------

/// A fully-specified Discord REST API request, ready for the transport layer
/// to add auth headers and dispatch.
#[derive(Debug)]
pub struct DiscordRequest {
	pub method: HttpMethod,
	/// Path relative to the API base URL, e.g. `"channels/123/messages"`.
	pub path: String,
	/// Route template used for per-route rate-limit bucketing,
	/// e.g. `"POST /channels/123/messages"`.
	pub route_key: String,
	pub body: RequestBody,
}

/// The body payload of a [`DiscordRequest`].
#[derive(Debug)]
pub enum RequestBody {
	/// No body (typical for GET / DELETE).
	None,
	/// A JSON-encoded body.
	Json(serde_json::Value),
	/// A raw body with an explicit `Content-Type` header
	/// (e.g. `multipart/form-data`).
	Raw { content_type: String, data: Vec<u8> },
}

// ---------------------------------------------------------------------------
// JsonError – lightweight error for request/response (de)serialisation
// ---------------------------------------------------------------------------

/// Error produced when serialising a request body or deserialising a
/// response body.  Converted into the transport-layer `HttpError` by the
/// `From` impl in `discord_io::http`.
#[derive(Debug)]
pub struct JsonError(pub String);

impl std::fmt::Display for JsonError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0)
	}
}

impl std::error::Error for JsonError {}

// ---------------------------------------------------------------------------
// IntoDiscordRequest trait
// ---------------------------------------------------------------------------

/// Types that can be converted into a Discord REST API request.
///
/// Auth and user-agent headers are appended automatically by
/// [`DiscordHttpClient::send`](crate::discord_io::DiscordHttpClient::send)
/// before the request is dispatched.
///
/// # Examples
///
/// ```ignore
/// // CreateMessage stores the channel_id and body fields.
/// let msg = CreateMessage::new(channel_id)
///     .content("Hello!")
///     .reply_to(some_message_id);
///
/// let response: Message = http.send(msg).await?;
/// ```
pub trait IntoDiscordRequest {
	/// The type returned in the response body (e.g. `Message`, `()`, …).
	type Output;

	/// Build the [`DiscordRequest`] from this value.
	fn into_discord_request(self) -> Result<DiscordRequest, JsonError>;

	/// Parse raw response bytes into [`Self::Output`].
	fn parse_response(bytes: &[u8]) -> Result<Self::Output, JsonError>;
}

// ---------------------------------------------------------------------------
// Helper functions (DRY utilities for IntoDiscordRequest impls)
// ---------------------------------------------------------------------------

/// Serialise a value into a [`RequestBody::Json`].
pub fn json_body<T: Serialize>(value: &T) -> Result<RequestBody, JsonError> {
	serde_json::to_value(value)
		.map(RequestBody::Json)
		.map_err(|e| JsonError(e.to_string()))
}

/// Deserialise a JSON response body.
pub fn parse_json<T: serde::de::DeserializeOwned>(
	bytes: &[u8],
) -> Result<T, JsonError> {
	serde_json::from_slice(bytes).map_err(|e| {
		let raw = String::from_utf8_lossy(bytes);
		JsonError(format!("{}: {}", e, &raw[..raw.len().min(200)]))
	})
}

/// "Parse" an empty response (204 No Content, etc.).
pub fn parse_empty(_bytes: &[u8]) -> Result<(), JsonError> { Ok(()) }

// ---------------------------------------------------------------------------
// Multipart helpers
// ---------------------------------------------------------------------------

/// Generate a boundary string for multipart requests without requiring
/// the `rand` crate (which is behind the `io` feature).
pub fn generate_boundary() -> String {
	use std::time::SystemTime;
	use std::time::UNIX_EPOCH;
	let nanos = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_nanos();
	// Mix in the address of a stack variable for extra entropy.
	let stack_addr = &nanos as *const _ as usize;
	format!("BeetBoundary{:016x}{:x}", nanos, stack_addr)
}

/// Build a `multipart/form-data` body as raw bytes.
///
/// Produces parts for an optional `payload_json` text field and a
/// required file part named `"file"`.
pub fn build_multipart(
	boundary: &str,
	content: Option<&str>,
	filename: &str,
	file_data: &[u8],
) -> Vec<u8> {
	let mut buf: Vec<u8> = Vec::new();

	// Optional payload_json field.
	if let Some(text) = content {
		let payload = serde_json::json!({ "content": text }).to_string();
		buf.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
		buf.extend_from_slice(
			b"Content-Disposition: form-data; name=\"payload_json\"\r\n",
		);
		buf.extend_from_slice(b"Content-Type: application/json\r\n\r\n");
		buf.extend_from_slice(payload.as_bytes());
		buf.extend_from_slice(b"\r\n");
	}

	// File part.
	buf.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
	buf.extend_from_slice(
		format!(
			"Content-Disposition: form-data; name=\"file\"; \
			 filename=\"{}\"\r\n",
			filename
		)
		.as_bytes(),
	);
	buf.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
	buf.extend_from_slice(file_data);
	buf.extend_from_slice(b"\r\n");

	// Closing boundary.
	buf.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

	buf
}

/// Percent-encode a unicode emoji for use in reaction URLs.
/// Custom emoji in `name:id` format are returned as-is.
pub fn url_encode_emoji(emoji: &str) -> String {
	if emoji.contains(':') {
		// Custom emoji — no encoding needed.
		emoji.to_string()
	} else {
		use std::fmt::Write;
		let mut encoded = String::new();
		for byte in emoji.as_bytes() {
			write!(encoded, "%{:02X}", byte).unwrap();
		}
		encoded
	}
}

// ---------------------------------------------------------------------------
// Outbound message reference (used by CreateMessage body)
// ---------------------------------------------------------------------------

/// Simplified message reference for outbound messages.
///
/// Uses typed `Id<T>` markers for compile-time safety.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateMessageReference {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub message_id: Option<Id<MessageMarker>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub channel_id: Option<Id<ChannelMarker>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub guild_id: Option<Id<GuildMarker>>,
	#[serde(default)]
	pub fail_if_not_exists: bool,
}

// ---------------------------------------------------------------------------
// Rate-limit info parsed from response headers
// ---------------------------------------------------------------------------

/// Rate-limit metadata extracted from Discord REST API response headers.
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
	pub remaining: Option<u32>,
	#[allow(dead_code)]
	pub reset_at: Option<f64>,
	pub reset_after: Option<f64>,
	pub bucket: Option<String>,
	pub is_global: bool,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn json_body_serialises_value() {
		let body = json_body(&serde_json::json!({"content": "hi"})).unwrap();
		match body {
			RequestBody::Json(v) => {
				assert_eq!(v["content"], "hi");
			}
			_ => panic!("expected Json variant"),
		}
	}

	#[test]
	fn parse_json_deserialises() {
		let bytes = br#"{"name":"test"}"#;
		let val: serde_json::Value = parse_json(bytes).unwrap();
		assert_eq!(val["name"], "test");
	}

	#[test]
	fn parse_json_error_includes_snippet() {
		let bytes = b"not json";
		let err = parse_json::<serde_json::Value>(bytes).unwrap_err();
		assert!(err.0.contains("not json"), "error should include raw body");
	}

	#[test]
	fn parse_empty_always_succeeds() {
		assert!(parse_empty(b"").is_ok());
		assert!(parse_empty(b"some bytes").is_ok());
	}

	#[test]
	fn generate_boundary_not_empty() {
		let b = generate_boundary();
		assert!(b.starts_with("BeetBoundary"));
		assert!(b.len() > 20);
	}

	#[test]
	fn build_multipart_produces_valid_body() {
		let boundary = "TestBoundary";
		let body =
			build_multipart(boundary, Some("hello"), "test.txt", b"data");
		let body_str = String::from_utf8_lossy(&body);
		assert!(body_str.contains("--TestBoundary\r\n"));
		assert!(body_str.contains("payload_json"));
		assert!(body_str.contains("\"content\":\"hello\""));
		assert!(body_str.contains("filename=\"test.txt\""));
		assert!(body_str.contains("--TestBoundary--\r\n"));
	}

	#[test]
	fn build_multipart_without_content() {
		let boundary = "TestBoundary";
		let body = build_multipart(boundary, None, "img.png", b"\x89PNG");
		let body_str = String::from_utf8_lossy(&body);
		assert!(!body_str.contains("payload_json"));
		assert!(body_str.contains("filename=\"img.png\""));
	}

	#[test]
	fn url_encode_emoji_unicode() {
		// "👍" is U+1F44D → UTF-8 bytes: F0 9F 91 8D
		let encoded = url_encode_emoji("👍");
		assert_eq!(encoded, "%F0%9F%91%8D");
	}

	#[test]
	fn url_encode_emoji_custom() {
		let encoded = url_encode_emoji("blobcat:123456789");
		assert_eq!(encoded, "blobcat:123456789");
	}

	#[test]
	fn create_message_reference_serialises() {
		let reference = CreateMessageReference {
			message_id: Some(Id::new(12345)),
			channel_id: None,
			guild_id: None,
			fail_if_not_exists: false,
		};
		let json = serde_json::to_string(&reference).unwrap();
		assert!(json.contains("\"message_id\":\"12345\""));
		assert!(!json.contains("channel_id"));
		assert!(!json.contains("guild_id"));
	}
}
