//! Types copy-pasted from `twilight-http` (`twilight-http/src/api_error.rs`).
//!
//! These are reproduced here so we can deserialize Discord API error responses
//! without pulling in the full `twilight-http` crate and its heavy dependency
//! tree (hyper, rustls, tokio, etc.). Only `serde`, `serde_json`, and `std`
//! are required.

use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum ApiError {
	General(GeneralApiError),
	/// Request has been ratelimited.
	Ratelimited(RatelimitedApiError),
	/// Something was wrong with the input when sending a message.
	Message(MessageApiError),
}

impl Display for ApiError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::General(inner) => Display::fmt(inner, f),
			Self::Message(inner) => Display::fmt(inner, f),
			Self::Ratelimited(inner) => Display::fmt(inner, f),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct GeneralApiError {
	pub code: u64,
	pub message: String,
}

impl Display for GeneralApiError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("Error code ")?;
		Display::fmt(&self.code, f)?;
		f.write_str(": ")?;

		f.write_str(&self.message)
	}
}

/// Sending a message failed because the provided fields contained invalid
/// input.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct MessageApiError {
	/// Fields within a provided embed were invalid.
	pub embed: Option<Vec<MessageApiErrorEmbedField>>,
}

impl Display for MessageApiError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("message fields invalid: ")?;

		if let Some(embed) = &self.embed {
			f.write_str("embed (")?;

			let field_count = embed.len().saturating_sub(1);

			for (idx, field) in embed.iter().enumerate() {
				Display::fmt(field, f)?;

				if idx == field_count {
					f.write_str(", ")?;
				}
			}

			f.write_str(")")?;
		}

		Ok(())
	}
}

/// Field within a [`MessageApiError`] [embed] list.
///
/// [embed]: MessageApiError::embed
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum MessageApiErrorEmbedField {
	/// Something was wrong with the provided fields.
	Fields,
	/// The provided timestamp wasn't a valid RFC3339 string.
	Timestamp,
}

impl Display for MessageApiErrorEmbedField {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Fields => "fields",
			Self::Timestamp => "timestamp",
		})
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RatelimitedApiError {
	/// Whether the ratelimit is a global ratelimit.
	pub global: bool,
	/// Human readable message provided by the API.
	pub message: String,
	/// Amount of time to wait before retrying.
	pub retry_after: f64,
}

impl Display for RatelimitedApiError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("Got ")?;

		if self.global {
			f.write_str("global ")?;
		}

		f.write_str("ratelimited for ")?;
		Display::fmt(&self.retry_after, f)?;

		f.write_str("s")
	}
}

impl Eq for RatelimitedApiError {}

impl PartialEq for RatelimitedApiError {
	fn eq(&self, other: &Self) -> bool {
		self.global == other.global && self.message == other.message
	}
}

#[cfg(test)]
mod tests {
	use super::ApiError;
	use super::GeneralApiError;
	use super::MessageApiError;
	use super::MessageApiErrorEmbedField;
	use super::RatelimitedApiError;

	#[test]
	fn general_api_error_roundtrip() {
		let expected = GeneralApiError {
			code: 10001,
			message: "Unknown account".to_owned(),
		};

		let json = serde_json::to_string(&expected).unwrap();
		let deserialized: GeneralApiError =
			serde_json::from_str(&json).unwrap();
		assert_eq!(expected, deserialized);

		// Also verify from raw JSON
		let raw = r#"{"code":10001,"message":"Unknown account"}"#;
		let from_raw: GeneralApiError = serde_json::from_str(raw).unwrap();
		assert_eq!(expected, from_raw);
	}

	#[test]
	fn general_api_error_display() {
		let error = GeneralApiError {
			code: 10001,
			message: "Unknown account".to_owned(),
		};
		assert_eq!(error.to_string(), "Error code 10001: Unknown account");
	}

	#[test]
	fn api_error_general_variant() {
		let raw = r#"{"code":10001,"message":"Unknown account"}"#;
		let error: ApiError = serde_json::from_str(raw).unwrap();
		assert!(matches!(error, ApiError::General(_)));

		if let ApiError::General(inner) = &error {
			assert_eq!(inner.code, 10001);
			assert_eq!(inner.message, "Unknown account");
		}

		// Round-trip
		let json = serde_json::to_string(&error).unwrap();
		let round_tripped: ApiError = serde_json::from_str(&json).unwrap();
		assert_eq!(error, round_tripped);
	}

	#[test]
	fn api_error_message_variant() {
		let expected = ApiError::Message(MessageApiError {
			embed: Some(vec![
				MessageApiErrorEmbedField::Fields,
				MessageApiErrorEmbedField::Timestamp,
			]),
		});

		let json = serde_json::to_string(&expected).unwrap();
		let deserialized: ApiError = serde_json::from_str(&json).unwrap();
		assert_eq!(expected, deserialized);

		// Also verify from raw JSON
		let raw = r#"{"embed":["fields","timestamp"]}"#;
		let from_raw: ApiError = serde_json::from_str(raw).unwrap();
		assert_eq!(expected, from_raw);
	}

	#[test]
	fn message_api_error_display() {
		let error = MessageApiError {
			embed: Some(vec![
				MessageApiErrorEmbedField::Fields,
				MessageApiErrorEmbedField::Timestamp,
			]),
		};
		let display = error.to_string();
		assert!(display.contains("message fields invalid:"));
		assert!(display.contains("embed ("));
		assert!(display.contains("fields"));
		assert!(display.contains("timestamp"));
	}

	#[test]
	fn ratelimited_api_error_roundtrip() {
		let expected = RatelimitedApiError {
			global: true,
			message: "You are being rate limited.".to_owned(),
			retry_after: 6.457,
		};

		let json = serde_json::to_string(&expected).unwrap();
		let deserialized: RatelimitedApiError =
			serde_json::from_str(&json).unwrap();
		assert_eq!(expected, deserialized);

		// Also verify from raw JSON
		let raw = r#"{"global":true,"message":"You are being rate limited.","retry_after":6.457}"#;
		let from_raw: RatelimitedApiError = serde_json::from_str(raw).unwrap();
		assert_eq!(expected, from_raw);
	}

	#[test]
	fn ratelimited_api_error_display() {
		let error = RatelimitedApiError {
			global: true,
			message: "You are being rate limited.".to_owned(),
			retry_after: 6.457,
		};
		assert_eq!(error.to_string(), "Got global ratelimited for 6.457s");

		let non_global = RatelimitedApiError {
			global: false,
			message: "You are being rate limited.".to_owned(),
			retry_after: 0.5,
		};
		assert_eq!(non_global.to_string(), "Got ratelimited for 0.5s");
	}

	/// Assert that deserializing an [`ApiError::Ratelimited`] variant uses
	/// the correct variant.
	///
	/// Tests for [#1302], which was due to a previously ordered variant having
	/// higher priority for untagged deserialization.
	///
	/// [#1302]: https://github.com/twilight-rs/twilight/issues/1302
	#[test]
	fn api_error_variant_ratelimited() {
		let raw = r#"{"global":false,"message":"You are being rate limited.","retry_after":0.362}"#;
		let error: ApiError = serde_json::from_str(raw).unwrap();

		let expected = ApiError::Ratelimited(RatelimitedApiError {
			global: false,
			message: "You are being rate limited.".to_owned(),
			retry_after: 0.362,
		});

		assert_eq!(expected, error);

		// Round-trip
		let json = serde_json::to_string(&error).unwrap();
		let round_tripped: ApiError = serde_json::from_str(&json).unwrap();
		assert_eq!(expected, round_tripped);
	}

	#[test]
	fn embed_field_serde() {
		let fields = MessageApiErrorEmbedField::Fields;
		let json = serde_json::to_string(&fields).unwrap();
		assert_eq!(json, r#""fields""#);

		let timestamp = MessageApiErrorEmbedField::Timestamp;
		let json = serde_json::to_string(&timestamp).unwrap();
		assert_eq!(json, r#""timestamp""#);

		let from_str: MessageApiErrorEmbedField =
			serde_json::from_str(r#""fields""#).unwrap();
		assert_eq!(from_str, MessageApiErrorEmbedField::Fields);

		let from_str: MessageApiErrorEmbedField =
			serde_json::from_str(r#""timestamp""#).unwrap();
		assert_eq!(from_str, MessageApiErrorEmbedField::Timestamp);
	}
}
