//! Miscellaneous Discord REST API request types: webhooks, threads, invites,
//! emojis, stickers, stage instances, scheduled events, templates,
//! auto-moderation, voice, polls, and guild welcome/onboarding.

use crate::prelude::*;
use beet::prelude::*;
use twilight_model::channel::Channel;
use twilight_model::channel::message::Message;
use twilight_model::channel::message::component::Component;
use twilight_model::channel::message::embed::Embed;
use twilight_model::channel::thread::ThreadMember;
use twilight_model::channel::thread::ThreadsListing;
use twilight_model::guild::Emoji;
use twilight_model::guild::auto_moderation::AutoModerationRule;
use twilight_model::guild::invite::Invite;
use twilight_model::guild::scheduled_event::GuildScheduledEvent;
use twilight_model::guild::scheduled_event::GuildScheduledEventUser;
use twilight_model::guild::template::Template;
use twilight_model::id::Id;

use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::marker::EmojiMarker;
use twilight_model::id::marker::GenericMarker;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::marker::MessageMarker;
use twilight_model::id::marker::RoleMarker;
use twilight_model::id::marker::ScheduledEventMarker;
use twilight_model::id::marker::UserMarker;
use twilight_model::id::marker::WebhookMarker;
use twilight_model::voice::VoiceRegion;

// ===========================================================================
// Webhooks
// ===========================================================================

// ---- GetWebhook -----------------------------------------------------------

/// Get a webhook by ID.
#[derive(Debug, Clone)]
pub struct GetWebhook {
	webhook_id: Id<WebhookMarker>,
}

impl GetWebhook {
	pub fn new(webhook_id: Id<WebhookMarker>) -> Self { Self { webhook_id } }
}

impl IntoDiscordRequest for GetWebhook {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("webhooks/{}", self.webhook_id);
		let route_key = format!("GET /webhooks/{}", self.webhook_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateWebhook --------------------------------------------------------

/// Create a webhook in a channel.
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhook {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub avatar: Option<String>,
}

impl CreateWebhook {
	pub fn new(channel_id: Id<ChannelMarker>, name: impl Into<String>) -> Self {
		Self {
			channel_id,
			name: name.into(),
			avatar: None,
		}
	}

	/// Set the avatar (base64-encoded data URI).
	pub fn avatar(mut self, avatar: impl Into<String>) -> Self {
		self.avatar = Some(avatar.into());
		self
	}
}

impl IntoDiscordRequest for CreateWebhook {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/webhooks", self.channel_id);
		let route_key = format!("POST /channels/{}/webhooks", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteWebhook --------------------------------------------------------

/// Delete a webhook by ID.
#[derive(Debug, Clone)]
pub struct DeleteWebhook {
	webhook_id: Id<WebhookMarker>,
}

impl DeleteWebhook {
	pub fn new(webhook_id: Id<WebhookMarker>) -> Self { Self { webhook_id } }
}

impl IntoDiscordRequest for DeleteWebhook {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("webhooks/{}", self.webhook_id);
		let route_key = format!("DELETE /webhooks/{}", self.webhook_id);
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

// ---- UpdateWebhook --------------------------------------------------------

/// Update a webhook's settings.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateWebhook {
	#[serde(skip)]
	webhook_id: Id<WebhookMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub avatar: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub channel_id: Option<Id<ChannelMarker>>,
}

impl UpdateWebhook {
	pub fn new(webhook_id: Id<WebhookMarker>) -> Self {
		Self {
			webhook_id,
			name: None,
			avatar: None,
			channel_id: None,
		}
	}

	/// Set the new webhook name.
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	/// Set the new avatar (base64-encoded data URI).
	pub fn avatar(mut self, avatar: impl Into<String>) -> Self {
		self.avatar = Some(avatar.into());
		self
	}

	/// Move the webhook to a different channel.
	pub fn channel_id(mut self, channel_id: Id<ChannelMarker>) -> Self {
		self.channel_id = Some(channel_id);
		self
	}
}

impl IntoDiscordRequest for UpdateWebhook {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("webhooks/{}", self.webhook_id);
		let route_key = format!("PATCH /webhooks/{}", self.webhook_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- ExecuteWebhook -------------------------------------------------------

/// Execute a webhook (send a message via webhook).
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteWebhook {
	#[serde(skip)]
	webhook_id: Id<WebhookMarker>,
	#[serde(skip)]
	webhook_token: String,
	#[serde(skip)]
	wait_: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embeds: Option<Vec<Embed>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<Vec<Component>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub username: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub avatar_url: Option<String>,
}

impl ExecuteWebhook {
	pub fn new(
		webhook_id: Id<WebhookMarker>,
		webhook_token: impl Into<String>,
	) -> Self {
		Self {
			webhook_id,
			webhook_token: webhook_token.into(),
			wait_: false,
			content: None,
			embeds: None,
			components: None,
			username: None,
			avatar_url: None,
		}
	}

	/// Set the text content.
	pub fn content(mut self, text: impl Into<String>) -> Self {
		self.content = Some(text.into());
		self
	}

	/// Append an embed.
	pub fn embed(mut self, embed: Embed) -> Self {
		self.embeds.get_or_insert_with(Vec::new).push(embed);
		self
	}

	/// Append a component row.
	pub fn component_row(mut self, row: Component) -> Self {
		self.components.get_or_insert_with(Vec::new).push(row);
		self
	}

	/// Override the webhook's default username.
	pub fn username(mut self, username: impl Into<String>) -> Self {
		self.username = Some(username.into());
		self
	}

	/// Override the webhook's default avatar URL.
	pub fn avatar_url(mut self, url: impl Into<String>) -> Self {
		self.avatar_url = Some(url.into());
		self
	}

	/// If true, the response will include a Message object.
	pub fn wait_(mut self, wait: bool) -> Self {
		self.wait_ = wait;
		self
	}
}

impl IntoDiscordRequest for ExecuteWebhook {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let query = if self.wait_ { "?wait=true" } else { "" };
		let path = format!(
			"webhooks/{}/{}{}",
			self.webhook_id, self.webhook_token, query
		);
		let route_key = format!(
			"POST /webhooks/{}/{}",
			self.webhook_id, self.webhook_token
		);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetWebhookMessage ----------------------------------------------------

/// Get a message sent by a webhook.
#[derive(Debug, Clone)]
pub struct GetWebhookMessage {
	webhook_id: Id<WebhookMarker>,
	webhook_token: String,
	message_id: Id<MessageMarker>,
}

impl GetWebhookMessage {
	pub fn new(
		webhook_id: Id<WebhookMarker>,
		webhook_token: impl Into<String>,
		message_id: Id<MessageMarker>,
	) -> Self {
		Self {
			webhook_id,
			webhook_token: webhook_token.into(),
			message_id,
		}
	}
}

impl IntoDiscordRequest for GetWebhookMessage {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}/messages/{}",
			self.webhook_id, self.webhook_token, self.message_id
		);
		let route_key = format!(
			"GET /webhooks/{}/{}/messages",
			self.webhook_id, self.webhook_token
		);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Message, JsonError> {
		parse_json(bytes)
	}
}

// ---- UpdateWebhookMessage -------------------------------------------------

/// Edit a message sent by a webhook.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateWebhookMessage {
	#[serde(skip)]
	webhook_id: Id<WebhookMarker>,
	#[serde(skip)]
	webhook_token: String,
	#[serde(skip)]
	message_id: Id<MessageMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embeds: Option<Vec<Embed>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<Vec<Component>>,
}

impl UpdateWebhookMessage {
	pub fn new(
		webhook_id: Id<WebhookMarker>,
		webhook_token: impl Into<String>,
		message_id: Id<MessageMarker>,
	) -> Self {
		Self {
			webhook_id,
			webhook_token: webhook_token.into(),
			message_id,
			content: None,
			embeds: None,
			components: None,
		}
	}

	/// Set the new text content.
	pub fn content(mut self, text: impl Into<String>) -> Self {
		self.content = Some(text.into());
		self
	}

	/// Append an embed.
	pub fn embed(mut self, embed: Embed) -> Self {
		self.embeds.get_or_insert_with(Vec::new).push(embed);
		self
	}

	/// Append a component row.
	pub fn component_row(mut self, row: Component) -> Self {
		self.components.get_or_insert_with(Vec::new).push(row);
		self
	}
}

impl IntoDiscordRequest for UpdateWebhookMessage {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}/messages/{}",
			self.webhook_id, self.webhook_token, self.message_id
		);
		let route_key = format!(
			"PATCH /webhooks/{}/{}/messages",
			self.webhook_id, self.webhook_token
		);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Message, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteWebhookMessage -------------------------------------------------

/// Delete a message sent by a webhook.
#[derive(Debug, Clone)]
pub struct DeleteWebhookMessage {
	webhook_id: Id<WebhookMarker>,
	webhook_token: String,
	message_id: Id<MessageMarker>,
}

impl DeleteWebhookMessage {
	pub fn new(
		webhook_id: Id<WebhookMarker>,
		webhook_token: impl Into<String>,
		message_id: Id<MessageMarker>,
	) -> Self {
		Self {
			webhook_id,
			webhook_token: webhook_token.into(),
			message_id,
		}
	}
}

impl IntoDiscordRequest for DeleteWebhookMessage {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}/messages/{}",
			self.webhook_id, self.webhook_token, self.message_id
		);
		let route_key = format!(
			"DELETE /webhooks/{}/{}/messages",
			self.webhook_id, self.webhook_token
		);
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
// Threads
// ===========================================================================

// ---- GetActiveThreads -----------------------------------------------------

/// Get all active threads in a guild.
#[derive(Debug, Clone)]
pub struct GetActiveThreads {
	guild_id: Id<GuildMarker>,
}

impl GetActiveThreads {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetActiveThreads {
	type Output = ThreadsListing;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/threads/active", self.guild_id);
		let route_key = format!("GET /guilds/{}/threads/active", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<ThreadsListing, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateThread ---------------------------------------------------------

/// Create a new thread in a channel (not from a message).
#[derive(Debug, Clone, Serialize)]
pub struct CreateThread {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub auto_archive_duration: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub invitable: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rate_limit_per_user: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "type")]
	pub kind: Option<u16>,
}

impl CreateThread {
	pub fn new(channel_id: Id<ChannelMarker>, name: impl Into<String>) -> Self {
		Self {
			channel_id,
			name: name.into(),
			auto_archive_duration: None,
			invitable: None,
			rate_limit_per_user: None,
			kind: None,
		}
	}

	/// Set auto-archive duration in minutes (60, 1440, 4320, 10080).
	pub fn auto_archive_duration(mut self, minutes: u16) -> Self {
		self.auto_archive_duration = Some(minutes);
		self
	}

	/// Whether non-moderators can add other non-moderators (private threads).
	pub fn invitable(mut self, invitable: bool) -> Self {
		self.invitable = Some(invitable);
		self
	}

	/// Slowmode rate limit per user in seconds.
	pub fn rate_limit_per_user(mut self, seconds: u16) -> Self {
		self.rate_limit_per_user = Some(seconds);
		self
	}

	/// Thread type (10 = announcement, 11 = public, 12 = private).
	pub fn kind(mut self, kind: u16) -> Self {
		self.kind = Some(kind);
		self
	}
}

impl IntoDiscordRequest for CreateThread {
	type Output = Channel;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/threads", self.channel_id);
		let route_key = format!("POST /channels/{}/threads", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Channel, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateThreadFromMessage ----------------------------------------------

/// Create a thread from an existing message.
#[derive(Debug, Clone, Serialize)]
pub struct CreateThreadFromMessage {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	#[serde(skip)]
	message_id: Id<MessageMarker>,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub auto_archive_duration: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rate_limit_per_user: Option<u16>,
}

impl CreateThreadFromMessage {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		message_id: Id<MessageMarker>,
		name: impl Into<String>,
	) -> Self {
		Self {
			channel_id,
			message_id,
			name: name.into(),
			auto_archive_duration: None,
			rate_limit_per_user: None,
		}
	}

	/// Set auto-archive duration in minutes.
	pub fn auto_archive_duration(mut self, minutes: u16) -> Self {
		self.auto_archive_duration = Some(minutes);
		self
	}

	/// Slowmode rate limit per user in seconds.
	pub fn rate_limit_per_user(mut self, seconds: u16) -> Self {
		self.rate_limit_per_user = Some(seconds);
		self
	}
}

impl IntoDiscordRequest for CreateThreadFromMessage {
	type Output = Channel;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/messages/{}/threads",
			self.channel_id, self.message_id
		);
		let route_key =
			format!("POST /channels/{}/messages/threads", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Channel, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateForumThread ----------------------------------------------------

/// Create a thread in a forum or media channel (includes initial message).
#[derive(Debug, Clone)]
pub struct CreateForumThread {
	channel_id: Id<ChannelMarker>,
	name: String,
	auto_archive_duration: Option<u16>,
	content: Option<String>,
	embeds: Option<Vec<Embed>>,
}

impl CreateForumThread {
	pub fn new(channel_id: Id<ChannelMarker>, name: impl Into<String>) -> Self {
		Self {
			channel_id,
			name: name.into(),
			auto_archive_duration: None,
			content: None,
			embeds: None,
		}
	}

	/// Set auto-archive duration in minutes.
	pub fn auto_archive_duration(mut self, minutes: u16) -> Self {
		self.auto_archive_duration = Some(minutes);
		self
	}

	/// Set the initial message content.
	pub fn content(mut self, text: impl Into<String>) -> Self {
		self.content = Some(text.into());
		self
	}

	/// Append an embed to the initial message.
	pub fn embed(mut self, embed: Embed) -> Self {
		self.embeds.get_or_insert_with(Vec::new).push(embed);
		self
	}
}

impl IntoDiscordRequest for CreateForumThread {
	type Output = Channel;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/threads", self.channel_id);
		let route_key = format!("POST /channels/{}/threads", self.channel_id);

		// Build the JSON manually to nest the `message` subobject.
		let mut body = serde_json::Map::new();
		body.insert("name".to_string(), serde_json::Value::String(self.name));
		if let Some(dur) = self.auto_archive_duration {
			body.insert(
				"auto_archive_duration".to_string(),
				serde_json::Value::Number(dur.into()),
			);
		}
		let mut message = serde_json::Map::new();
		if let Some(content) = self.content {
			message.insert(
				"content".to_string(),
				serde_json::Value::String(content),
			);
		}
		if let Some(embeds) = self.embeds {
			message.insert(
				"embeds".to_string(),
				serde_json::to_value(&embeds)
					.map_err(|e| JsonError(e.to_string()))?,
			);
		}
		body.insert("message".to_string(), serde_json::Value::Object(message));

		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: RequestBody::Json(serde_json::Value::Object(body)),
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Channel, JsonError> {
		parse_json(bytes)
	}
}

// ---- JoinThread -----------------------------------------------------------

/// Join a thread (add current user).
#[derive(Debug, Clone)]
pub struct JoinThread {
	channel_id: Id<ChannelMarker>,
}

impl JoinThread {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for JoinThread {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/thread-members/@me", self.channel_id);
		let route_key =
			format!("PUT /channels/{}/thread-members/@me", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Put,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ---- LeaveThread ----------------------------------------------------------

/// Leave a thread (remove current user).
#[derive(Debug, Clone)]
pub struct LeaveThread {
	channel_id: Id<ChannelMarker>,
}

impl LeaveThread {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for LeaveThread {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/thread-members/@me", self.channel_id);
		let route_key =
			format!("DELETE /channels/{}/thread-members/@me", self.channel_id);
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

// ---- AddThreadMember ------------------------------------------------------

/// Add a user to a thread.
#[derive(Debug, Clone)]
pub struct AddThreadMember {
	channel_id: Id<ChannelMarker>,
	user_id: Id<UserMarker>,
}

impl AddThreadMember {
	pub fn new(channel_id: Id<ChannelMarker>, user_id: Id<UserMarker>) -> Self {
		Self {
			channel_id,
			user_id,
		}
	}
}

impl IntoDiscordRequest for AddThreadMember {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/thread-members/{}",
			self.channel_id, self.user_id
		);
		let route_key =
			format!("PUT /channels/{}/thread-members", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Put,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ---- RemoveThreadMember ---------------------------------------------------

/// Remove a user from a thread.
#[derive(Debug, Clone)]
pub struct RemoveThreadMember {
	channel_id: Id<ChannelMarker>,
	user_id: Id<UserMarker>,
}

impl RemoveThreadMember {
	pub fn new(channel_id: Id<ChannelMarker>, user_id: Id<UserMarker>) -> Self {
		Self {
			channel_id,
			user_id,
		}
	}
}

impl IntoDiscordRequest for RemoveThreadMember {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/thread-members/{}",
			self.channel_id, self.user_id
		);
		let route_key =
			format!("DELETE /channels/{}/thread-members", self.channel_id);
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

// ---- GetThreadMember ------------------------------------------------------

/// Get a single thread member.
#[derive(Debug, Clone)]
pub struct GetThreadMember {
	channel_id: Id<ChannelMarker>,
	user_id: Id<UserMarker>,
}

impl GetThreadMember {
	pub fn new(channel_id: Id<ChannelMarker>, user_id: Id<UserMarker>) -> Self {
		Self {
			channel_id,
			user_id,
		}
	}
}

impl IntoDiscordRequest for GetThreadMember {
	type Output = ThreadMember;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/thread-members/{}",
			self.channel_id, self.user_id
		);
		let route_key =
			format!("GET /channels/{}/thread-members", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<ThreadMember, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetThreadMembers -----------------------------------------------------

/// List all members of a thread.
#[derive(Debug, Clone)]
pub struct GetThreadMembers {
	channel_id: Id<ChannelMarker>,
}

impl GetThreadMembers {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for GetThreadMembers {
	type Output = Vec<ThreadMember>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/thread-members", self.channel_id);
		let route_key =
			format!("GET /channels/{}/thread-members", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<ThreadMember>, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Invites
// ===========================================================================

// ---- GetInvite ------------------------------------------------------------

/// Get an invite by its code.
#[derive(Debug, Clone)]
pub struct GetInvite {
	code: String,
}

impl GetInvite {
	pub fn new(code: impl Into<String>) -> Self { Self { code: code.into() } }
}

impl IntoDiscordRequest for GetInvite {
	type Output = Invite;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("invites/{}", self.code);
		let route_key = "GET /invites".to_string();
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Invite, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateInvite ---------------------------------------------------------

/// Create an invite for a channel.
#[derive(Debug, Clone, Serialize)]
pub struct CreateInvite {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_age: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_uses: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub temporary: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub unique: Option<bool>,
}

impl CreateInvite {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self {
		Self {
			channel_id,
			max_age: None,
			max_uses: None,
			temporary: None,
			unique: None,
		}
	}

	/// Max age in seconds (0 = never, default 86400).
	pub fn max_age(mut self, max_age: u32) -> Self {
		self.max_age = Some(max_age);
		self
	}

	/// Max number of uses (0 = unlimited, default 0).
	pub fn max_uses(mut self, max_uses: u16) -> Self {
		self.max_uses = Some(max_uses);
		self
	}

	/// Whether the invite grants temporary membership.
	pub fn temporary(mut self, temporary: bool) -> Self {
		self.temporary = Some(temporary);
		self
	}

	/// If true, try to reuse a similar invite (or create a new one).
	pub fn unique(mut self, unique: bool) -> Self {
		self.unique = Some(unique);
		self
	}
}

impl IntoDiscordRequest for CreateInvite {
	type Output = Invite;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/invites", self.channel_id);
		let route_key = format!("POST /channels/{}/invites", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Invite, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteInvite ---------------------------------------------------------

/// Delete an invite by its code. Returns the deleted invite.
#[derive(Debug, Clone)]
pub struct DeleteInvite {
	code: String,
}

impl DeleteInvite {
	pub fn new(code: impl Into<String>) -> Self { Self { code: code.into() } }
}

impl IntoDiscordRequest for DeleteInvite {
	type Output = Invite;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("invites/{}", self.code);
		let route_key = "DELETE /invites".to_string();
		Ok(DiscordRequest {
			method: HttpMethod::Delete,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Invite, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildInvites ------------------------------------------------------

/// List all invites for a guild.
#[derive(Debug, Clone)]
pub struct GetGuildInvites {
	guild_id: Id<GuildMarker>,
}

impl GetGuildInvites {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildInvites {
	type Output = Vec<Invite>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/invites", self.guild_id);
		let route_key = format!("GET /guilds/{}/invites", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Invite>, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Emojis
// ===========================================================================

// ---- GetGuildEmojis -------------------------------------------------------

/// List all emojis for a guild.
#[derive(Debug, Clone)]
pub struct GetGuildEmojis {
	guild_id: Id<GuildMarker>,
}

impl GetGuildEmojis {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildEmojis {
	type Output = Vec<Emoji>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/emojis", self.guild_id);
		let route_key = format!("GET /guilds/{}/emojis", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Emoji>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildEmoji --------------------------------------------------------

/// Get a single guild emoji by ID.
#[derive(Debug, Clone)]
pub struct GetGuildEmoji {
	guild_id: Id<GuildMarker>,
	emoji_id: Id<EmojiMarker>,
}

impl GetGuildEmoji {
	pub fn new(guild_id: Id<GuildMarker>, emoji_id: Id<EmojiMarker>) -> Self {
		Self { guild_id, emoji_id }
	}
}

impl IntoDiscordRequest for GetGuildEmoji {
	type Output = Emoji;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/emojis/{}", self.guild_id, self.emoji_id);
		let route_key = format!("GET /guilds/{}/emojis", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Emoji, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateGuildEmoji -----------------------------------------------------

/// Create a guild emoji.
#[derive(Debug, Clone, Serialize)]
pub struct CreateGuildEmoji {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	pub name: String,
	/// Base64-encoded image data URI.
	pub image: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub roles: Option<Vec<Id<RoleMarker>>>,
}

impl CreateGuildEmoji {
	pub fn new(
		guild_id: Id<GuildMarker>,
		name: impl Into<String>,
		image: impl Into<String>,
	) -> Self {
		Self {
			guild_id,
			name: name.into(),
			image: image.into(),
			roles: None,
		}
	}

	/// Restrict the emoji to the given roles.
	pub fn roles(mut self, roles: Vec<Id<RoleMarker>>) -> Self {
		self.roles = Some(roles);
		self
	}
}

impl IntoDiscordRequest for CreateGuildEmoji {
	type Output = Emoji;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/emojis", self.guild_id);
		let route_key = format!("POST /guilds/{}/emojis", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Emoji, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteGuildEmoji -----------------------------------------------------

/// Delete a guild emoji.
#[derive(Debug, Clone)]
pub struct DeleteGuildEmoji {
	guild_id: Id<GuildMarker>,
	emoji_id: Id<EmojiMarker>,
}

impl DeleteGuildEmoji {
	pub fn new(guild_id: Id<GuildMarker>, emoji_id: Id<EmojiMarker>) -> Self {
		Self { guild_id, emoji_id }
	}
}

impl IntoDiscordRequest for DeleteGuildEmoji {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/emojis/{}", self.guild_id, self.emoji_id);
		let route_key = format!("DELETE /guilds/{}/emojis", self.guild_id);
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

// ---- UpdateGuildEmoji -----------------------------------------------------

/// Update a guild emoji's name or roles.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateGuildEmoji {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip)]
	emoji_id: Id<EmojiMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub roles: Option<Vec<Id<RoleMarker>>>,
}

impl UpdateGuildEmoji {
	pub fn new(guild_id: Id<GuildMarker>, emoji_id: Id<EmojiMarker>) -> Self {
		Self {
			guild_id,
			emoji_id,
			name: None,
			roles: None,
		}
	}

	/// Set the new emoji name.
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	/// Set the roles that can use this emoji.
	pub fn roles(mut self, roles: Vec<Id<RoleMarker>>) -> Self {
		self.roles = Some(roles);
		self
	}
}

impl IntoDiscordRequest for UpdateGuildEmoji {
	type Output = Emoji;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/emojis/{}", self.guild_id, self.emoji_id);
		let route_key = format!("PATCH /guilds/{}/emojis", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Emoji, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Stage Instances
// ===========================================================================

// ---- CreateStageInstance --------------------------------------------------

/// Create a stage instance (go live in a stage channel).
#[derive(Debug, Clone, Serialize)]
pub struct CreateStageInstance {
	pub channel_id: Id<ChannelMarker>,
	pub topic: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub privacy_level: Option<u8>,
}

impl CreateStageInstance {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		topic: impl Into<String>,
	) -> Self {
		Self {
			channel_id,
			topic: topic.into(),
			privacy_level: None,
		}
	}

	/// Set the privacy level (2 = guild only).
	pub fn privacy_level(mut self, level: u8) -> Self {
		self.privacy_level = Some(level);
		self
	}
}

impl IntoDiscordRequest for CreateStageInstance {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = "stage-instances".to_string();
		let route_key = "POST /stage-instances".to_string();
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetStageInstance -----------------------------------------------------

/// Get a stage instance by channel ID.
#[derive(Debug, Clone)]
pub struct GetStageInstance {
	channel_id: Id<ChannelMarker>,
}

impl GetStageInstance {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for GetStageInstance {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("stage-instances/{}", self.channel_id);
		let route_key = format!("GET /stage-instances/{}", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- UpdateStageInstance --------------------------------------------------

/// Update a stage instance.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateStageInstance {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub topic: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub privacy_level: Option<u8>,
}

impl UpdateStageInstance {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self {
		Self {
			channel_id,
			topic: None,
			privacy_level: None,
		}
	}

	/// Set the new topic.
	pub fn topic(mut self, topic: impl Into<String>) -> Self {
		self.topic = Some(topic.into());
		self
	}

	/// Set the privacy level.
	pub fn privacy_level(mut self, level: u8) -> Self {
		self.privacy_level = Some(level);
		self
	}
}

impl IntoDiscordRequest for UpdateStageInstance {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("stage-instances/{}", self.channel_id);
		let route_key = format!("PATCH /stage-instances/{}", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteStageInstance --------------------------------------------------

/// Delete a stage instance.
#[derive(Debug, Clone)]
pub struct DeleteStageInstance {
	channel_id: Id<ChannelMarker>,
}

impl DeleteStageInstance {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for DeleteStageInstance {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("stage-instances/{}", self.channel_id);
		let route_key = format!("DELETE /stage-instances/{}", self.channel_id);
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
// Scheduled Events
// ===========================================================================

// ---- GetGuildScheduledEvents ----------------------------------------------

/// List all scheduled events for a guild.
#[derive(Debug, Clone)]
pub struct GetGuildScheduledEvents {
	guild_id: Id<GuildMarker>,
	with_user_count: bool,
}

impl GetGuildScheduledEvents {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			with_user_count: false,
		}
	}

	/// Include the number of users subscribed to each event.
	pub fn with_user_count(mut self, with: bool) -> Self {
		self.with_user_count = with;
		self
	}
}

impl IntoDiscordRequest for GetGuildScheduledEvents {
	type Output = Vec<GuildScheduledEvent>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let query = if self.with_user_count {
			"?with_user_count=true"
		} else {
			""
		};
		let path =
			format!("guilds/{}/scheduled-events{}", self.guild_id, query);
		let route_key =
			format!("GET /guilds/{}/scheduled-events", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(
		bytes: &[u8],
	) -> Result<Vec<GuildScheduledEvent>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildScheduledEvent -----------------------------------------------

/// Get a single scheduled event.
#[derive(Debug, Clone)]
pub struct GetGuildScheduledEvent {
	guild_id: Id<GuildMarker>,
	event_id: Id<ScheduledEventMarker>,
	with_user_count: bool,
}

impl GetGuildScheduledEvent {
	pub fn new(
		guild_id: Id<GuildMarker>,
		event_id: Id<ScheduledEventMarker>,
	) -> Self {
		Self {
			guild_id,
			event_id,
			with_user_count: false,
		}
	}

	/// Include the number of users subscribed to the event.
	pub fn with_user_count(mut self, with: bool) -> Self {
		self.with_user_count = with;
		self
	}
}

impl IntoDiscordRequest for GetGuildScheduledEvent {
	type Output = GuildScheduledEvent;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let query = if self.with_user_count {
			"?with_user_count=true"
		} else {
			""
		};
		let path = format!(
			"guilds/{}/scheduled-events/{}{}",
			self.guild_id, self.event_id, query
		);
		let route_key =
			format!("GET /guilds/{}/scheduled-events", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<GuildScheduledEvent, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateGuildScheduledEvent --------------------------------------------

/// Create a guild scheduled event.
#[derive(Debug, Clone, Serialize)]
pub struct CreateGuildScheduledEvent {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	pub name: String,
	pub scheduled_start_time: String,
	pub entity_type: u8,
	pub privacy_level: u8,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub channel_id: Option<Id<ChannelMarker>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub scheduled_end_time: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub entity_metadata: Option<serde_json::Value>,
}

impl CreateGuildScheduledEvent {
	pub fn new(
		guild_id: Id<GuildMarker>,
		name: impl Into<String>,
		scheduled_start_time: impl Into<String>,
		entity_type: u8,
		privacy_level: u8,
	) -> Self {
		Self {
			guild_id,
			name: name.into(),
			scheduled_start_time: scheduled_start_time.into(),
			entity_type,
			privacy_level,
			channel_id: None,
			scheduled_end_time: None,
			description: None,
			entity_metadata: None,
		}
	}

	/// Set the channel for the event (voice/stage).
	pub fn channel_id(mut self, id: Id<ChannelMarker>) -> Self {
		self.channel_id = Some(id);
		self
	}

	/// Set the scheduled end time (ISO 8601).
	pub fn scheduled_end_time(mut self, time: impl Into<String>) -> Self {
		self.scheduled_end_time = Some(time.into());
		self
	}

	/// Set the event description.
	pub fn description(mut self, desc: impl Into<String>) -> Self {
		self.description = Some(desc.into());
		self
	}

	/// Set entity metadata (e.g. `{"location": "..."}`).
	pub fn entity_metadata(mut self, meta: serde_json::Value) -> Self {
		self.entity_metadata = Some(meta);
		self
	}
}

impl IntoDiscordRequest for CreateGuildScheduledEvent {
	type Output = GuildScheduledEvent;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/scheduled-events", self.guild_id);
		let route_key =
			format!("POST /guilds/{}/scheduled-events", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<GuildScheduledEvent, JsonError> {
		parse_json(bytes)
	}
}

// ---- UpdateGuildScheduledEvent --------------------------------------------

/// Update a guild scheduled event.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateGuildScheduledEvent {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip)]
	event_id: Id<ScheduledEventMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub scheduled_start_time: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub scheduled_end_time: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub entity_type: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub channel_id: Option<Id<ChannelMarker>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub entity_metadata: Option<serde_json::Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub privacy_level: Option<u8>,
}

impl UpdateGuildScheduledEvent {
	pub fn new(
		guild_id: Id<GuildMarker>,
		event_id: Id<ScheduledEventMarker>,
	) -> Self {
		Self {
			guild_id,
			event_id,
			name: None,
			description: None,
			scheduled_start_time: None,
			scheduled_end_time: None,
			entity_type: None,
			status: None,
			channel_id: None,
			entity_metadata: None,
			privacy_level: None,
		}
	}

	/// Set the event name.
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	/// Set the event description.
	pub fn description(mut self, desc: impl Into<String>) -> Self {
		self.description = Some(desc.into());
		self
	}

	/// Set the scheduled start time (ISO 8601).
	pub fn scheduled_start_time(mut self, time: impl Into<String>) -> Self {
		self.scheduled_start_time = Some(time.into());
		self
	}

	/// Set the scheduled end time (ISO 8601).
	pub fn scheduled_end_time(mut self, time: impl Into<String>) -> Self {
		self.scheduled_end_time = Some(time.into());
		self
	}

	/// Set the entity type.
	pub fn entity_type(mut self, t: u8) -> Self {
		self.entity_type = Some(t);
		self
	}

	/// Set the event status (1 = scheduled, 2 = active, 3 = completed, 4 = canceled).
	pub fn status(mut self, status: u8) -> Self {
		self.status = Some(status);
		self
	}

	/// Set the channel for the event.
	pub fn channel_id(mut self, id: Id<ChannelMarker>) -> Self {
		self.channel_id = Some(id);
		self
	}

	/// Set entity metadata.
	pub fn entity_metadata(mut self, meta: serde_json::Value) -> Self {
		self.entity_metadata = Some(meta);
		self
	}

	/// Set the privacy level.
	pub fn privacy_level(mut self, level: u8) -> Self {
		self.privacy_level = Some(level);
		self
	}
}

impl IntoDiscordRequest for UpdateGuildScheduledEvent {
	type Output = GuildScheduledEvent;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"guilds/{}/scheduled-events/{}",
			self.guild_id, self.event_id
		);
		let route_key =
			format!("PATCH /guilds/{}/scheduled-events", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<GuildScheduledEvent, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteGuildScheduledEvent --------------------------------------------

/// Delete a guild scheduled event.
#[derive(Debug, Clone)]
pub struct DeleteGuildScheduledEvent {
	guild_id: Id<GuildMarker>,
	event_id: Id<ScheduledEventMarker>,
}

impl DeleteGuildScheduledEvent {
	pub fn new(
		guild_id: Id<GuildMarker>,
		event_id: Id<ScheduledEventMarker>,
	) -> Self {
		Self { guild_id, event_id }
	}
}

impl IntoDiscordRequest for DeleteGuildScheduledEvent {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"guilds/{}/scheduled-events/{}",
			self.guild_id, self.event_id
		);
		let route_key =
			format!("DELETE /guilds/{}/scheduled-events", self.guild_id);
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

// ---- GetGuildScheduledEventUsers ------------------------------------------

/// Get users subscribed to a scheduled event.
#[derive(Debug, Clone)]
pub struct GetGuildScheduledEventUsers {
	guild_id: Id<GuildMarker>,
	event_id: Id<ScheduledEventMarker>,
	limit: Option<u16>,
	with_member: bool,
}

impl GetGuildScheduledEventUsers {
	pub fn new(
		guild_id: Id<GuildMarker>,
		event_id: Id<ScheduledEventMarker>,
	) -> Self {
		Self {
			guild_id,
			event_id,
			limit: None,
			with_member: false,
		}
	}

	/// Max number of users to return (1–100, default 100).
	pub fn limit(mut self, limit: u16) -> Self {
		self.limit = Some(limit.min(100));
		self
	}

	/// Include guild member data in each event user object.
	pub fn with_member(mut self, with: bool) -> Self {
		self.with_member = with;
		self
	}
}

impl IntoDiscordRequest for GetGuildScheduledEventUsers {
	type Output = Vec<GuildScheduledEventUser>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let mut query_parts = Vec::new();
		if let Some(limit) = self.limit {
			query_parts.push(format!("limit={}", limit));
		}
		if self.with_member {
			query_parts.push("with_member=true".to_string());
		}
		let query = if query_parts.is_empty() {
			String::new()
		} else {
			format!("?{}", query_parts.join("&"))
		};
		let path = format!(
			"guilds/{}/scheduled-events/{}/users{}",
			self.guild_id, self.event_id, query
		);
		let route_key = format!(
			"GET /guilds/{}/scheduled-events/{}/users",
			self.guild_id, self.event_id
		);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(
		bytes: &[u8],
	) -> Result<Vec<GuildScheduledEventUser>, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Stickers
// ===========================================================================

// ---- GetSticker -----------------------------------------------------------

/// Get a sticker by ID.
#[derive(Debug, Clone)]
pub struct GetSticker {
	sticker_id: Id<GenericMarker>,
}

impl GetSticker {
	pub fn new(sticker_id: Id<GenericMarker>) -> Self { Self { sticker_id } }
}

impl IntoDiscordRequest for GetSticker {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("stickers/{}", self.sticker_id);
		let route_key = "GET /stickers".to_string();
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetNitroStickerPacks -------------------------------------------------

/// Get the list of Nitro sticker packs.
#[derive(Debug, Clone)]
pub struct GetNitroStickerPacks;

impl IntoDiscordRequest for GetNitroStickerPacks {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path: "sticker-packs".to_string(),
			route_key: "GET /sticker-packs".to_string(),
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildStickers -----------------------------------------------------

/// List all stickers in a guild.
#[derive(Debug, Clone)]
pub struct GetGuildStickers {
	guild_id: Id<GuildMarker>,
}

impl GetGuildStickers {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildStickers {
	type Output = Vec<serde_json::Value>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/stickers", self.guild_id);
		let route_key = format!("GET /guilds/{}/stickers", self.guild_id);
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

// ---- GetGuildSticker ------------------------------------------------------

/// Get a single guild sticker.
#[derive(Debug, Clone)]
pub struct GetGuildSticker {
	guild_id: Id<GuildMarker>,
	sticker_id: Id<GenericMarker>,
}

impl GetGuildSticker {
	pub fn new(
		guild_id: Id<GuildMarker>,
		sticker_id: Id<GenericMarker>,
	) -> Self {
		Self {
			guild_id,
			sticker_id,
		}
	}
}

impl IntoDiscordRequest for GetGuildSticker {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path =
			format!("guilds/{}/stickers/{}", self.guild_id, self.sticker_id);
		let route_key = format!("GET /guilds/{}/stickers", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- UpdateGuildSticker ---------------------------------------------------

/// Update a guild sticker.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateGuildSticker {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip)]
	sticker_id: Id<GenericMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tags: Option<String>,
}

impl UpdateGuildSticker {
	pub fn new(
		guild_id: Id<GuildMarker>,
		sticker_id: Id<GenericMarker>,
	) -> Self {
		Self {
			guild_id,
			sticker_id,
			name: None,
			description: None,
			tags: None,
		}
	}

	/// Set the sticker name.
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	/// Set the sticker description.
	pub fn description(mut self, desc: impl Into<String>) -> Self {
		self.description = Some(desc.into());
		self
	}

	/// Set the autocomplete/suggestion tags (comma-separated).
	pub fn tags(mut self, tags: impl Into<String>) -> Self {
		self.tags = Some(tags.into());
		self
	}
}

impl IntoDiscordRequest for UpdateGuildSticker {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path =
			format!("guilds/{}/stickers/{}", self.guild_id, self.sticker_id);
		let route_key = format!("PATCH /guilds/{}/stickers", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteGuildSticker ---------------------------------------------------

/// Delete a guild sticker.
#[derive(Debug, Clone)]
pub struct DeleteGuildSticker {
	guild_id: Id<GuildMarker>,
	sticker_id: Id<GenericMarker>,
}

impl DeleteGuildSticker {
	pub fn new(
		guild_id: Id<GuildMarker>,
		sticker_id: Id<GenericMarker>,
	) -> Self {
		Self {
			guild_id,
			sticker_id,
		}
	}
}

impl IntoDiscordRequest for DeleteGuildSticker {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path =
			format!("guilds/{}/stickers/{}", self.guild_id, self.sticker_id);
		let route_key = format!("DELETE /guilds/{}/stickers", self.guild_id);
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
// Auto Moderation
// ===========================================================================

// ---- GetAutoModerationRules -----------------------------------------------

/// List all auto-moderation rules for a guild.
#[derive(Debug, Clone)]
pub struct GetAutoModerationRules {
	guild_id: Id<GuildMarker>,
}

impl GetAutoModerationRules {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetAutoModerationRules {
	type Output = Vec<AutoModerationRule>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/auto-moderation/rules", self.guild_id);
		let route_key =
			format!("GET /guilds/{}/auto-moderation/rules", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(
		bytes: &[u8],
	) -> Result<Vec<AutoModerationRule>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetAutoModerationRule ------------------------------------------------

/// Get a single auto-moderation rule.
#[derive(Debug, Clone)]
pub struct GetAutoModerationRule {
	guild_id: Id<GuildMarker>,
	rule_id: Id<GenericMarker>,
}

impl GetAutoModerationRule {
	pub fn new(guild_id: Id<GuildMarker>, rule_id: Id<GenericMarker>) -> Self {
		Self { guild_id, rule_id }
	}
}

impl IntoDiscordRequest for GetAutoModerationRule {
	type Output = AutoModerationRule;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"guilds/{}/auto-moderation/rules/{}",
			self.guild_id, self.rule_id
		);
		let route_key =
			format!("GET /guilds/{}/auto-moderation/rules", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<AutoModerationRule, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteAutoModerationRule ---------------------------------------------

/// Delete an auto-moderation rule.
#[derive(Debug, Clone)]
pub struct DeleteAutoModerationRule {
	guild_id: Id<GuildMarker>,
	rule_id: Id<GenericMarker>,
}

impl DeleteAutoModerationRule {
	pub fn new(guild_id: Id<GuildMarker>, rule_id: Id<GenericMarker>) -> Self {
		Self { guild_id, rule_id }
	}
}

impl IntoDiscordRequest for DeleteAutoModerationRule {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"guilds/{}/auto-moderation/rules/{}",
			self.guild_id, self.rule_id
		);
		let route_key =
			format!("DELETE /guilds/{}/auto-moderation/rules", self.guild_id);
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
// Templates
// ===========================================================================

// ---- GetTemplate ----------------------------------------------------------

/// Get a guild template by code.
#[derive(Debug, Clone)]
pub struct GetTemplate {
	template_code: String,
}

impl GetTemplate {
	pub fn new(template_code: impl Into<String>) -> Self {
		Self {
			template_code: template_code.into(),
		}
	}
}

impl IntoDiscordRequest for GetTemplate {
	type Output = Template;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/templates/{}", self.template_code);
		let route_key = "GET /guilds/templates".to_string();
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Template, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildTemplates ----------------------------------------------------

/// List all templates for a guild.
#[derive(Debug, Clone)]
pub struct GetGuildTemplates {
	guild_id: Id<GuildMarker>,
}

impl GetGuildTemplates {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildTemplates {
	type Output = Vec<Template>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/templates", self.guild_id);
		let route_key = format!("GET /guilds/{}/templates", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Template>, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteTemplate -------------------------------------------------------

/// Delete a guild template. Returns the deleted template.
#[derive(Debug, Clone)]
pub struct DeleteTemplate {
	guild_id: Id<GuildMarker>,
	template_code: String,
}

impl DeleteTemplate {
	pub fn new(
		guild_id: Id<GuildMarker>,
		template_code: impl Into<String>,
	) -> Self {
		Self {
			guild_id,
			template_code: template_code.into(),
		}
	}
}

impl IntoDiscordRequest for DeleteTemplate {
	type Output = Template;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"guilds/{}/templates/{}",
			self.guild_id, self.template_code
		);
		let route_key = format!("DELETE /guilds/{}/templates", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Delete,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Template, JsonError> {
		parse_json(bytes)
	}
}

// ---- SyncTemplate ---------------------------------------------------------

/// Sync a guild template to the guild's current state.
#[derive(Debug, Clone)]
pub struct SyncTemplate {
	guild_id: Id<GuildMarker>,
	template_code: String,
}

impl SyncTemplate {
	pub fn new(
		guild_id: Id<GuildMarker>,
		template_code: impl Into<String>,
	) -> Self {
		Self {
			guild_id,
			template_code: template_code.into(),
		}
	}
}

impl IntoDiscordRequest for SyncTemplate {
	type Output = Template;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"guilds/{}/templates/{}",
			self.guild_id, self.template_code
		);
		let route_key = format!("PUT /guilds/{}/templates", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Put,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Template, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Voice & Audit
// ===========================================================================

// ---- GetVoiceRegions ------------------------------------------------------

/// List available voice regions.
#[derive(Debug, Clone)]
pub struct GetVoiceRegions;

impl IntoDiscordRequest for GetVoiceRegions {
	type Output = Vec<VoiceRegion>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path: "voice/regions".to_string(),
			route_key: "GET /voice/regions".to_string(),
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<VoiceRegion>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetAuditLog ----------------------------------------------------------

/// Get a guild's audit log.
#[derive(Debug, Clone)]
pub struct GetAuditLog {
	guild_id: Id<GuildMarker>,
	user_id: Option<Id<UserMarker>>,
	action_type: Option<u8>,
	before: Option<String>,
	after: Option<String>,
	limit: Option<u8>,
}

impl GetAuditLog {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			user_id: None,
			action_type: None,
			before: None,
			after: None,
			limit: None,
		}
	}

	/// Filter by the user who performed the action.
	pub fn user_id(mut self, user_id: Id<UserMarker>) -> Self {
		self.user_id = Some(user_id);
		self
	}

	/// Filter by audit log action type.
	pub fn action_type(mut self, action_type: u8) -> Self {
		self.action_type = Some(action_type);
		self
	}

	/// Get entries before this entry ID.
	pub fn before(mut self, before: impl Into<String>) -> Self {
		self.before = Some(before.into());
		self
	}

	/// Get entries after this entry ID.
	pub fn after(mut self, after: impl Into<String>) -> Self {
		self.after = Some(after.into());
		self
	}

	/// Max number of entries to return (1–100, default 50).
	pub fn limit(mut self, limit: u8) -> Self {
		self.limit = Some(limit.min(100));
		self
	}
}

impl IntoDiscordRequest for GetAuditLog {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let mut query_parts = Vec::new();
		if let Some(user_id) = self.user_id {
			query_parts.push(format!("user_id={}", user_id));
		}
		if let Some(action_type) = self.action_type {
			query_parts.push(format!("action_type={}", action_type));
		}
		if let Some(before) = self.before {
			query_parts.push(format!("before={}", before));
		}
		if let Some(after) = self.after {
			query_parts.push(format!("after={}", after));
		}
		if let Some(limit) = self.limit {
			query_parts.push(format!("limit={}", limit));
		}
		let query = if query_parts.is_empty() {
			String::new()
		} else {
			format!("?{}", query_parts.join("&"))
		};
		let path = format!("guilds/{}/audit-logs{}", self.guild_id, query);
		let route_key = format!("GET /guilds/{}/audit-logs", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Polls
// ===========================================================================

// ---- EndPoll --------------------------------------------------------------

/// Immediately end a poll in a channel.
#[derive(Debug, Clone)]
pub struct EndPoll {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
}

impl EndPoll {
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

impl IntoDiscordRequest for EndPoll {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/polls/{}/expire",
			self.channel_id, self.message_id
		);
		let route_key =
			format!("POST /channels/{}/polls/expire", self.channel_id);
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

// ---- GetAnswerVoters ------------------------------------------------------

/// Get voters for a specific poll answer.
#[derive(Debug, Clone)]
pub struct GetAnswerVoters {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
	answer_id: u8,
	limit: Option<u16>,
	after: Option<Id<UserMarker>>,
}

impl GetAnswerVoters {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		message_id: Id<MessageMarker>,
		answer_id: u8,
	) -> Self {
		Self {
			channel_id,
			message_id,
			answer_id,
			limit: None,
			after: None,
		}
	}

	/// Max number of voters to return.
	pub fn limit(mut self, limit: u16) -> Self {
		self.limit = Some(limit);
		self
	}

	/// Get voters after this user ID (pagination).
	pub fn after(mut self, user_id: Id<UserMarker>) -> Self {
		self.after = Some(user_id);
		self
	}
}

impl IntoDiscordRequest for GetAnswerVoters {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
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
			"channels/{}/polls/{}/answers/{}{}",
			self.channel_id, self.message_id, self.answer_id, query
		);
		let route_key =
			format!("GET /channels/{}/polls/answers", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Guild Welcome / Onboarding
// ===========================================================================

// ---- GetGuildWelcomeScreen ------------------------------------------------

/// Get a guild's welcome screen.
#[derive(Debug, Clone)]
pub struct GetGuildWelcomeScreen {
	guild_id: Id<GuildMarker>,
}

impl GetGuildWelcomeScreen {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildWelcomeScreen {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/welcome-screen", self.guild_id);
		let route_key = format!("GET /guilds/{}/welcome-screen", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildOnboarding ---------------------------------------------------

/// Get a guild's onboarding configuration.
#[derive(Debug, Clone)]
pub struct GetGuildOnboarding {
	guild_id: Id<GuildMarker>,
}

impl GetGuildOnboarding {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildOnboarding {
	type Output = serde_json::Value;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/onboarding", self.guild_id);
		let route_key = format!("GET /guilds/{}/onboarding", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<serde_json::Value, JsonError> {
		parse_json(bytes)
	}
}
