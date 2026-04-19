//! Additional channel, message, and reaction request types for the Discord
//! REST API.
//!
//! These complement the core types in [`super::requests`] with less commonly
//! used endpoints.

use crate::prelude::*;
use beet::prelude::*;
use twilight_model::channel::Channel;
use twilight_model::channel::FollowedChannel;
use twilight_model::channel::message::Message;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::marker::GenericMarker;
use twilight_model::id::marker::MessageMarker;
use twilight_model::id::marker::UserMarker;
use twilight_model::user::User;

// ===========================================================================
// Messages
// ===========================================================================

// ---- BulkDeleteMessages ---------------------------------------------------

/// Bulk-delete messages in a channel (2–100 messages, < 14 days old).
#[derive(Debug, Clone, Serialize)]
pub struct BulkDeleteMessages {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	pub messages: Vec<Id<MessageMarker>>,
}

impl BulkDeleteMessages {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		messages: Vec<Id<MessageMarker>>,
	) -> Self {
		Self {
			channel_id,
			messages,
		}
	}
}

impl IntoDiscordRequest for BulkDeleteMessages {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/messages/bulk-delete", self.channel_id);
		let route_key =
			format!("POST /channels/{}/messages/bulk-delete", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ---- CrosspostMessage -----------------------------------------------------

/// Crosspost (publish) a message in an Announcement channel to followers.
#[derive(Debug, Clone)]
pub struct CrosspostMessage {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
}

impl CrosspostMessage {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		message_id: Id<MessageMarker>,
	) -> Self {
		Self {
			channel_id,
			message_id,
		}
	}
}

impl IntoDiscordRequest for CrosspostMessage {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/messages/{}/crosspost",
			self.channel_id, self.message_id
		);
		let route_key =
			format!("POST /channels/{}/messages/crosspost", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Message, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Channels
// ===========================================================================

// ---- DeleteChannel --------------------------------------------------------

/// Delete a channel, or close a private message.
#[derive(Debug, Clone)]
pub struct DeleteChannel {
	channel_id: Id<ChannelMarker>,
}

impl DeleteChannel {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for DeleteChannel {
	type Output = Channel;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}", self.channel_id);
		let route_key = format!("DELETE /channels/{}", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Delete,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Channel, JsonError> {
		parse_json(bytes)
	}
}

// ---- UpdateChannel --------------------------------------------------------

/// Modify a channel's settings (name, topic, NSFW flag, etc.).
#[derive(Debug, Clone, Serialize)]
pub struct UpdateChannel {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub topic: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub nsfw: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rate_limit_per_user: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bitrate: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user_limit: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub position: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parent_id: Option<Option<Id<ChannelMarker>>>,
}

impl UpdateChannel {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self {
		Self {
			channel_id,
			name: None,
			topic: None,
			nsfw: None,
			rate_limit_per_user: None,
			bitrate: None,
			user_limit: None,
			position: None,
			parent_id: None,
		}
	}

	/// Set the channel name (1–100 characters).
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	/// Set the channel topic (0–4096 characters for forum channels, 0–1024 for others).
	pub fn topic(mut self, topic: impl Into<String>) -> Self {
		self.topic = Some(topic.into());
		self
	}

	/// Set whether the channel is NSFW.
	pub fn nsfw(mut self, nsfw: bool) -> Self {
		self.nsfw = Some(nsfw);
		self
	}

	/// Set the slowmode rate limit per user in seconds (0–21600).
	pub fn rate_limit_per_user(mut self, seconds: u16) -> Self {
		self.rate_limit_per_user = Some(seconds);
		self
	}

	/// Set the bitrate for a voice channel (8000–…).
	pub fn bitrate(mut self, bitrate: u32) -> Self {
		self.bitrate = Some(bitrate);
		self
	}

	/// Set the user limit for a voice channel (0 = unlimited, 1–99).
	pub fn user_limit(mut self, limit: u16) -> Self {
		self.user_limit = Some(limit);
		self
	}

	/// Set the sorting position of the channel.
	pub fn position(mut self, position: u16) -> Self {
		self.position = Some(position);
		self
	}

	/// Set the parent category, or `None` to remove from a category.
	pub fn parent_id(mut self, parent_id: Option<Id<ChannelMarker>>) -> Self {
		self.parent_id = Some(parent_id);
		self
	}
}

impl IntoDiscordRequest for UpdateChannel {
	type Output = Channel;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}", self.channel_id);
		let route_key = format!("PATCH /channels/{}", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Channel, JsonError> {
		parse_json(bytes)
	}
}

// ---- FollowNewsChannel ----------------------------------------------------

/// Follow an Announcement channel to send messages to a target channel.
#[derive(Debug, Clone, Serialize)]
pub struct FollowNewsChannel {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	pub webhook_channel_id: Id<ChannelMarker>,
}

impl FollowNewsChannel {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		webhook_channel_id: Id<ChannelMarker>,
	) -> Self {
		Self {
			channel_id,
			webhook_channel_id,
		}
	}
}

impl IntoDiscordRequest for FollowNewsChannel {
	type Output = FollowedChannel;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/followers", self.channel_id);
		let route_key = format!("POST /channels/{}/followers", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<FollowedChannel, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetChannelInvites ----------------------------------------------------

/// Get all invites for a channel.
#[derive(Debug, Clone)]
pub struct GetChannelInvites {
	channel_id: Id<ChannelMarker>,
}

impl GetChannelInvites {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for GetChannelInvites {
	type Output = Vec<serde_json::Value>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/invites", self.channel_id);
		let route_key = format!("GET /channels/{}/invites", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(
		bytes: &[u8],
	) -> Result<Vec<serde_json::Value>, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteChannelPermission ----------------------------------------------

/// Delete a channel permission overwrite for a user or role.
#[derive(Debug, Clone)]
pub struct DeleteChannelPermission {
	channel_id: Id<ChannelMarker>,
	overwrite_id: Id<GenericMarker>,
}

impl DeleteChannelPermission {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		overwrite_id: Id<GenericMarker>,
	) -> Self {
		Self {
			channel_id,
			overwrite_id,
		}
	}
}

impl IntoDiscordRequest for DeleteChannelPermission {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/permissions/{}",
			self.channel_id, self.overwrite_id
		);
		let route_key =
			format!("DELETE /channels/{}/permissions", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Delete,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ---- UpdateChannelPermission ----------------------------------------------

/// Edit a channel permission overwrite for a user or role.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateChannelPermission {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	#[serde(skip)]
	overwrite_id: Id<GenericMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub allow: Option<Permissions>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub deny: Option<Permissions>,
	/// 0 for role, 1 for member.
	#[serde(rename = "type")]
	pub kind: u8,
}

impl UpdateChannelPermission {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		overwrite_id: Id<GenericMarker>,
		kind: u8,
	) -> Self {
		Self {
			channel_id,
			overwrite_id,
			allow: None,
			deny: None,
			kind,
		}
	}

	/// Set the allowed permission bitfield.
	pub fn allow(mut self, allow: Permissions) -> Self {
		self.allow = Some(allow);
		self
	}

	/// Set the denied permission bitfield.
	pub fn deny(mut self, deny: Permissions) -> Self {
		self.deny = Some(deny);
		self
	}
}

impl IntoDiscordRequest for UpdateChannelPermission {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/permissions/{}",
			self.channel_id, self.overwrite_id
		);
		let route_key =
			format!("PUT /channels/{}/permissions", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Put,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ---- GetChannelWebhooks ---------------------------------------------------

/// Get all webhooks for a channel.
#[derive(Debug, Clone)]
pub struct GetChannelWebhooks {
	channel_id: Id<ChannelMarker>,
}

impl GetChannelWebhooks {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for GetChannelWebhooks {
	type Output = Vec<serde_json::Value>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/webhooks", self.channel_id);
		let route_key = format!("GET /channels/{}/webhooks", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(
		bytes: &[u8],
	) -> Result<Vec<serde_json::Value>, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Reactions
// ===========================================================================

// ---- GetReactions ---------------------------------------------------------

/// Get users who reacted with a specific emoji on a message.
#[derive(Debug, Clone)]
pub struct GetReactions {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
	emoji: String,
	limit: Option<u16>,
	after: Option<Id<UserMarker>>,
}

impl GetReactions {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		message_id: Id<MessageMarker>,
		emoji: impl Into<String>,
	) -> Self {
		Self {
			channel_id,
			message_id,
			emoji: emoji.into(),
			limit: None,
			after: None,
		}
	}

	/// Maximum number of users to return (1–100, default 25).
	pub fn limit(mut self, limit: u16) -> Self {
		self.limit = Some(limit.min(100));
		self
	}

	/// Get users after this user ID (pagination).
	pub fn after(mut self, user_id: Id<UserMarker>) -> Self {
		self.after = Some(user_id);
		self
	}
}

impl IntoDiscordRequest for GetReactions {
	type Output = Vec<User>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let encoded = url_encode_emoji(&self.emoji);
		let mut query_parts = Vec::new();
		if let Some(limit) = self.limit {
			query_parts.push(format!("limit={}", limit));
		}
		if let Some(after) = self.after {
			query_parts.push(format!("after={}", after));
		}
		let query = if query_parts.is_empty() {
			String::new()
		} else {
			format!("?{}", query_parts.join("&"))
		};
		let path = format!(
			"channels/{}/messages/{}/reactions/{}{}",
			self.channel_id, self.message_id, encoded, query
		);
		let route_key =
			format!("GET /channels/{}/messages/reactions", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<User>, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteUserReaction ---------------------------------------------------

/// Remove a specific user's reaction from a message.
#[derive(Debug, Clone)]
pub struct DeleteUserReaction {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
	emoji: String,
	user_id: Id<UserMarker>,
}

impl DeleteUserReaction {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		message_id: Id<MessageMarker>,
		emoji: impl Into<String>,
		user_id: Id<UserMarker>,
	) -> Self {
		Self {
			channel_id,
			message_id,
			emoji: emoji.into(),
			user_id,
		}
	}
}

impl IntoDiscordRequest for DeleteUserReaction {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let encoded = url_encode_emoji(&self.emoji);
		let path = format!(
			"channels/{}/messages/{}/reactions/{}/{}",
			self.channel_id, self.message_id, encoded, self.user_id
		);
		let route_key =
			format!("DELETE /channels/{}/messages/reactions", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Delete,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ---- DeleteAllReactionsForEmoji -------------------------------------------

/// Remove all reactions of a specific emoji from a message.
#[derive(Debug, Clone)]
pub struct DeleteAllReactionsForEmoji {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
	emoji: String,
}

impl DeleteAllReactionsForEmoji {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		message_id: Id<MessageMarker>,
		emoji: impl Into<String>,
	) -> Self {
		Self {
			channel_id,
			message_id,
			emoji: emoji.into(),
		}
	}
}

impl IntoDiscordRequest for DeleteAllReactionsForEmoji {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let encoded = url_encode_emoji(&self.emoji);
		let path = format!(
			"channels/{}/messages/{}/reactions/{}",
			self.channel_id, self.message_id, encoded
		);
		let route_key =
			format!("DELETE /channels/{}/messages/reactions", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Delete,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
	use super::*;
	use twilight_model::id::Id;

	#[test]
	fn bulk_delete_messages_serializes_body() {
		let req = BulkDeleteMessages::new(Id::new(111), vec![
			Id::new(1),
			Id::new(2),
			Id::new(3),
		]);
		let dr = req.into_discord_request().unwrap();
		assert_eq!(dr.path, "channels/111/messages/bulk-delete");
		assert_eq!(dr.route_key, "POST /channels/111/messages/bulk-delete");
		match &dr.body {
			RequestBody::Json(v) => {
				let arr = v["messages"].as_array().unwrap();
				assert_eq!(arr.len(), 3);
			}
			_ => panic!("expected Json body"),
		}
	}

	#[test]
	fn crosspost_message_into_request() {
		let req = CrosspostMessage::new(Id::new(111), Id::new(222));
		let dr = req.into_discord_request().unwrap();
		assert_eq!(dr.path, "channels/111/messages/222/crosspost");
		assert!(matches!(dr.body, RequestBody::None));
	}

	#[test]
	fn delete_channel_into_request() {
		let req = DeleteChannel::new(Id::new(111));
		let dr = req.into_discord_request().unwrap();
		assert_eq!(dr.path, "channels/111");
		assert_eq!(dr.route_key, "DELETE /channels/111");
	}

	#[test]
	fn update_channel_builder() {
		let req = UpdateChannel::new(Id::new(111))
			.name("new-name")
			.topic("new topic")
			.nsfw(true);
		let dr = req.into_discord_request().unwrap();
		assert_eq!(dr.path, "channels/111");
		assert_eq!(dr.route_key, "PATCH /channels/111");
		match &dr.body {
			RequestBody::Json(v) => {
				assert_eq!(v["name"], "new-name");
				assert_eq!(v["topic"], "new topic");
				assert_eq!(v["nsfw"], true);
				// Fields not set should be absent
				assert!(v.get("bitrate").is_none());
			}
			_ => panic!("expected Json body"),
		}
	}

	#[test]
	fn follow_news_channel_into_request() {
		let req = FollowNewsChannel::new(Id::new(111), Id::new(222));
		let dr = req.into_discord_request().unwrap();
		assert_eq!(dr.path, "channels/111/followers");
		match &dr.body {
			RequestBody::Json(v) => {
				assert!(v["webhook_channel_id"].as_str().is_some());
			}
			_ => panic!("expected Json body"),
		}
	}

	#[test]
	fn get_channel_invites_into_request() {
		let req = GetChannelInvites::new(Id::new(111));
		let dr = req.into_discord_request().unwrap();
		assert_eq!(dr.path, "channels/111/invites");
	}

	#[test]
	fn delete_channel_permission_into_request() {
		let req = DeleteChannelPermission::new(Id::new(111), Id::new(222));
		let dr = req.into_discord_request().unwrap();
		assert_eq!(dr.path, "channels/111/permissions/222");
		assert_eq!(dr.route_key, "DELETE /channels/111/permissions");
	}

	#[test]
	fn update_channel_permission_into_request() {
		let req = UpdateChannelPermission::new(Id::new(111), Id::new(222), 1)
			.allow(Permissions::SEND_MESSAGES);
		let dr = req.into_discord_request().unwrap();
		assert_eq!(dr.path, "channels/111/permissions/222");
		assert_eq!(dr.route_key, "PUT /channels/111/permissions");
		match &dr.body {
			RequestBody::Json(v) => {
				assert_eq!(v["type"], 1);
				assert!(v.get("allow").is_some());
				// deny was not set, should be absent
				assert!(v.get("deny").is_none());
			}
			_ => panic!("expected Json body"),
		}
	}

	#[test]
	fn get_channel_webhooks_into_request() {
		let req = GetChannelWebhooks::new(Id::new(111));
		let dr = req.into_discord_request().unwrap();
		assert_eq!(dr.path, "channels/111/webhooks");
	}

	#[test]
	fn get_reactions_with_pagination() {
		let req = GetReactions::new(Id::new(111), Id::new(222), "👍")
			.limit(50)
			.after(Id::new(333));
		let dr = req.into_discord_request().unwrap();
		assert!(dr.path.contains("/reactions/%F0%9F%91%8D"));
		assert!(dr.path.contains("limit=50"));
		assert!(dr.path.contains("after=333"));
		assert_eq!(dr.route_key, "GET /channels/111/messages/reactions");
	}

	#[test]
	fn get_reactions_no_params() {
		let req = GetReactions::new(Id::new(111), Id::new(222), "blobcat:999");
		let dr = req.into_discord_request().unwrap();
		assert!(dr.path.contains("/reactions/blobcat:999"));
		assert!(!dr.path.contains('?'));
	}

	#[test]
	fn delete_user_reaction_into_request() {
		let req = DeleteUserReaction::new(
			Id::new(111),
			Id::new(222),
			"👍",
			Id::new(333),
		);
		let dr = req.into_discord_request().unwrap();
		assert!(dr.path.contains("/reactions/%F0%9F%91%8D/333"));
		assert_eq!(dr.route_key, "DELETE /channels/111/messages/reactions");
	}

	#[test]
	fn delete_all_reactions_for_emoji_into_request() {
		let req = DeleteAllReactionsForEmoji::new(
			Id::new(111),
			Id::new(222),
			"blobcat:999",
		);
		let dr = req.into_discord_request().unwrap();
		assert!(dr.path.ends_with("/reactions/blobcat:999"));
		assert_eq!(dr.route_key, "DELETE /channels/111/messages/reactions");
	}
}
