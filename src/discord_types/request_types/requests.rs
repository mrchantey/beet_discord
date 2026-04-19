//! Concrete request types for the Discord REST API.
//!
//! Each struct stores the data needed for one API call and implements
//! [`IntoDiscordRequest`] so it can be dispatched via
//! [`DiscordHttpClient::send`](crate::discord_io::DiscordHttpClient::send).
//!
//! Helper utilities ([`json_body`], [`parse_json`], [`parse_empty`], …) live
//! in [`super::custom`] and are used here to keep implementations DRY.

use crate::prelude::*;
use beet::prelude::*;
use twilight_model::application::command::Command as ApplicationCommand;
use twilight_model::channel::Channel;
use twilight_model::channel::message::Message;
use twilight_model::channel::message::component::Component;
use twilight_model::channel::message::embed::Embed;
use twilight_model::guild::Guild;
use twilight_model::http::interaction::InteractionResponse;
use twilight_model::id::Id;
use twilight_model::id::marker::ApplicationMarker;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::marker::InteractionMarker;
use twilight_model::id::marker::MessageMarker;
use twilight_model::user::CurrentUser;
use twilight_model::user::CurrentUserGuild;

// ===========================================================================
// Messages
// ===========================================================================

// ---- CreateMessage --------------------------------------------------------

/// Create a message in a channel.
///
/// ```ignore
/// let msg = CreateMessage::new(channel_id)
///     .content("Hello!")
///     .reply_to(some_message_id);
/// let created: Message = http.send(msg).await?;
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct CreateMessage {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embeds: Option<Vec<Embed>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub message_reference: Option<CreateMessageReference>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<Vec<Component>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub flags: Option<u32>,
}

impl CreateMessage {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self {
		Self {
			channel_id,
			content: None,
			embeds: None,
			message_reference: None,
			components: None,
			flags: None,
		}
	}

	/// Set the text content of the message.
	pub fn content(mut self, text: impl Into<String>) -> Self {
		self.content = Some(text.into());
		self
	}

	/// Append an embed to the message.
	pub fn embed(mut self, embed: Embed) -> Self {
		self.embeds.get_or_insert_with(Vec::new).push(embed);
		self
	}

	/// Mark the message as a reply to another message.
	pub fn reply_to(mut self, message_id: Id<MessageMarker>) -> Self {
		self.message_reference = Some(CreateMessageReference {
			message_id: Some(message_id),
			channel_id: None,
			guild_id: None,
			fail_if_not_exists: false,
		});
		self
	}

	/// Append a component row to the message.
	pub fn component_row(mut self, row: Component) -> Self {
		self.components.get_or_insert_with(Vec::new).push(row);
		self
	}

	/// Set message flags (e.g. suppress embeds).
	pub fn flags(mut self, flags: u32) -> Self {
		self.flags = Some(flags);
		self
	}
}

impl IntoDiscordRequest for CreateMessage {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/messages", self.channel_id);
		let route_key = format!("POST /channels/{}/messages", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Message, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateMessageWithFile ------------------------------------------------

/// Create a message with a file attachment (multipart upload).
///
/// ```ignore
/// let req = CreateMessageWithFile::new(channel_id, "logo.png", bytes)
///     .content("Here's the file!");
/// let msg: Message = http.send(req).await?;
/// ```
#[derive(Debug, Clone)]
pub struct CreateMessageWithFile {
	channel_id: Id<ChannelMarker>,
	content: Option<String>,
	filename: String,
	file_data: Vec<u8>,
}

impl CreateMessageWithFile {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		filename: impl Into<String>,
		file_data: Vec<u8>,
	) -> Self {
		Self {
			channel_id,
			content: None,
			filename: filename.into(),
			file_data,
		}
	}

	pub fn content(mut self, text: impl Into<String>) -> Self {
		self.content = Some(text.into());
		self
	}
}

impl IntoDiscordRequest for CreateMessageWithFile {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/messages", self.channel_id);
		let route_key = format!("POST /channels/{}/messages", self.channel_id);
		let boundary = generate_boundary();
		let data = build_multipart(
			&boundary,
			self.content.as_deref(),
			&self.filename,
			&self.file_data,
		);
		let content_type =
			format!("multipart/form-data; boundary={}", boundary);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: RequestBody::Raw { content_type, data },
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Message, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetChannelMessages ---------------------------------------------------

/// Fetch messages from a channel with optional pagination.
///
/// ```ignore
/// let msgs: Vec<Message> = http.send(
///     GetChannelMessages::new(channel_id).limit(100).before(some_id)
/// ).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetChannelMessages {
	channel_id: Id<ChannelMarker>,
	limit: Option<u16>,
	before: Option<Id<MessageMarker>>,
	after: Option<Id<MessageMarker>>,
	around: Option<Id<MessageMarker>>,
}

impl GetChannelMessages {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self {
		Self {
			channel_id,
			limit: None,
			before: None,
			after: None,
			around: None,
		}
	}

	/// Maximum number of messages to return (1–100, default 50).
	pub fn limit(mut self, limit: u16) -> Self {
		self.limit = Some(limit.min(100));
		self
	}

	/// Get messages before this message ID.
	pub fn before(mut self, id: Id<MessageMarker>) -> Self {
		self.before = Some(id);
		self
	}

	/// Get messages after this message ID.
	pub fn after(mut self, id: Id<MessageMarker>) -> Self {
		self.after = Some(id);
		self
	}

	/// Get messages around this message ID.
	pub fn around(mut self, id: Id<MessageMarker>) -> Self {
		self.around = Some(id);
		self
	}
}

impl IntoDiscordRequest for GetChannelMessages {
	type Output = Vec<Message>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let mut query_parts = Vec::new();
		if let Some(limit) = self.limit {
			query_parts.push(format!("limit={}", limit));
		}
		if let Some(before) = self.before {
			query_parts.push(format!("before={}", before));
		}
		if let Some(after) = self.after {
			query_parts.push(format!("after={}", after));
		}
		if let Some(around) = self.around {
			query_parts.push(format!("around={}", around));
		}
		let query = if query_parts.is_empty() {
			String::new()
		} else {
			format!("?{}", query_parts.join("&"))
		};
		let path = format!("channels/{}/messages{}", self.channel_id, query);
		let route_key = format!("GET /channels/{}/messages", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Message>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetMessage -----------------------------------------------------------

/// Get a single message by ID.
///
/// ```ignore
/// let msg: Message = http.send(GetMessage::new(channel_id, message_id)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetMessage {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
}

impl GetMessage {
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

impl IntoDiscordRequest for GetMessage {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/messages/{}",
			self.channel_id, self.message_id
		);
		let route_key = format!("GET /channels/{}/messages", self.channel_id);
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

// ---- DeleteMessage --------------------------------------------------------

/// Delete a message.
///
/// ```ignore
/// http.send(DeleteMessage::new(channel_id, message_id)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct DeleteMessage {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
}

impl DeleteMessage {
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

impl IntoDiscordRequest for DeleteMessage {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/messages/{}",
			self.channel_id, self.message_id
		);
		let route_key =
			format!("DELETE /channels/{}/messages", self.channel_id);
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

// ---- EditMessage ----------------------------------------------------------

/// Edit an existing message.
///
/// ```ignore
/// let edited: Message = http.send(
///     EditMessage::new(channel_id, message_id).content("updated!")
/// ).await?;
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct EditMessage {
	#[serde(skip)]
	channel_id: Id<ChannelMarker>,
	#[serde(skip)]
	message_id: Id<MessageMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embeds: Option<Vec<Embed>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<Vec<Component>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub flags: Option<u32>,
}

impl EditMessage {
	pub fn new(
		channel_id: Id<ChannelMarker>,
		message_id: Id<MessageMarker>,
	) -> Self {
		Self {
			channel_id,
			message_id,
			content: None,
			embeds: None,
			components: None,
			flags: None,
		}
	}

	pub fn content(mut self, text: impl Into<String>) -> Self {
		self.content = Some(text.into());
		self
	}

	pub fn embed(mut self, embed: Embed) -> Self {
		self.embeds.get_or_insert_with(Vec::new).push(embed);
		self
	}

	pub fn component_row(mut self, row: Component) -> Self {
		self.components.get_or_insert_with(Vec::new).push(row);
		self
	}
}

impl IntoDiscordRequest for EditMessage {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/messages/{}",
			self.channel_id, self.message_id
		);
		let route_key = format!("PATCH /channels/{}/messages", self.channel_id);
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

// ===========================================================================
// Channels
// ===========================================================================

// ---- GetChannel -----------------------------------------------------------

/// Get channel information.
///
/// ```ignore
/// let ch: Channel = http.send(GetChannel::new(channel_id)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetChannel {
	channel_id: Id<ChannelMarker>,
}

impl GetChannel {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for GetChannel {
	type Output = Channel;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}", self.channel_id);
		let route_key = format!("GET /channels/{}", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Channel, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateTypingTrigger --------------------------------------------------

/// Fire a *Typing Start* event in a channel.
///
/// ```ignore
/// http.send(CreateTypingTrigger::new(channel_id)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct CreateTypingTrigger {
	channel_id: Id<ChannelMarker>,
}

impl CreateTypingTrigger {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for CreateTypingTrigger {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/typing", self.channel_id);
		let route_key = format!("POST /channels/{}/typing", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ---- GetPins --------------------------------------------------------------

/// Get all pinned messages in a channel.
///
/// ```ignore
/// let pins: Vec<Message> = http.send(GetPins::new(channel_id)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetPins {
	channel_id: Id<ChannelMarker>,
}

impl GetPins {
	pub fn new(channel_id: Id<ChannelMarker>) -> Self { Self { channel_id } }
}

impl IntoDiscordRequest for GetPins {
	type Output = Vec<Message>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("channels/{}/pins", self.channel_id);
		let route_key = format!("GET /channels/{}/pins", self.channel_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Message>, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreatePin ------------------------------------------------------------

/// Pin a message in a channel.
///
/// ```ignore
/// http.send(CreatePin::new(channel_id, message_id)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct CreatePin {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
}

impl CreatePin {
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

impl IntoDiscordRequest for CreatePin {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path =
			format!("channels/{}/pins/{}", self.channel_id, self.message_id);
		let route_key = format!("PUT /channels/{}/pins", self.channel_id);
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

// ---- DeletePin ------------------------------------------------------------

/// Unpin a message in a channel.
///
/// ```ignore
/// http.send(DeletePin::new(channel_id, message_id)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct DeletePin {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
}

impl DeletePin {
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

impl IntoDiscordRequest for DeletePin {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path =
			format!("channels/{}/pins/{}", self.channel_id, self.message_id);
		let route_key = format!("DELETE /channels/{}/pins", self.channel_id);
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
// Guilds
// ===========================================================================

// ---- GetGuild -------------------------------------------------------------

/// Get guild information (with approximate member counts by default).
///
/// ```ignore
/// let guild: Guild = http.send(GetGuild::new(guild_id)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetGuild {
	guild_id: Id<GuildMarker>,
	with_counts: bool,
}

impl GetGuild {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			with_counts: true,
		}
	}

	/// Whether to include approximate member and presence counts
	/// (default: `true`).
	pub fn with_counts(mut self, with_counts: bool) -> Self {
		self.with_counts = with_counts;
		self
	}
}

impl IntoDiscordRequest for GetGuild {
	type Output = Guild;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = if self.with_counts {
			format!("guilds/{}?with_counts=true", self.guild_id)
		} else {
			format!("guilds/{}", self.guild_id)
		};
		let route_key = format!("GET /guilds/{}", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Guild, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildChannels -----------------------------------------------------

/// List all channels in a guild.
///
/// ```ignore
/// let channels: Vec<Channel> = http.send(
///     GetGuildChannels::new(guild_id)
/// ).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetGuildChannels {
	guild_id: Id<GuildMarker>,
}

impl GetGuildChannels {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildChannels {
	type Output = Vec<Channel>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/channels", self.guild_id);
		let route_key = format!("GET /guilds/{}/channels", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Channel>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetCurrentUserGuilds -------------------------------------------------

/// List guilds the current user (bot) is a member of.
///
/// ```ignore
/// let guilds: Vec<CurrentUserGuild> = http.send(
///     GetCurrentUserGuilds::new().limit(100)
/// ).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetCurrentUserGuilds {
	limit: Option<u16>,
	before: Option<Id<GuildMarker>>,
	after: Option<Id<GuildMarker>>,
}

impl GetCurrentUserGuilds {
	pub fn new() -> Self {
		Self {
			limit: None,
			before: None,
			after: None,
		}
	}

	/// Maximum number of guilds to return (1–200, default 200).
	pub fn limit(mut self, limit: u16) -> Self {
		self.limit = Some(limit.min(200));
		self
	}

	/// Get guilds before this guild ID.
	pub fn before(mut self, id: Id<GuildMarker>) -> Self {
		self.before = Some(id);
		self
	}

	/// Get guilds after this guild ID.
	pub fn after(mut self, id: Id<GuildMarker>) -> Self {
		self.after = Some(id);
		self
	}
}

impl Default for GetCurrentUserGuilds {
	fn default() -> Self { Self::new() }
}

impl IntoDiscordRequest for GetCurrentUserGuilds {
	type Output = Vec<CurrentUserGuild>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let mut query_parts = Vec::new();
		if let Some(limit) = self.limit {
			query_parts.push(format!("limit={}", limit));
		}
		if let Some(before) = self.before {
			query_parts.push(format!("before={}", before));
		}
		if let Some(after) = self.after {
			query_parts.push(format!("after={}", after));
		}
		let query = if query_parts.is_empty() {
			String::new()
		} else {
			format!("?{}", query_parts.join("&"))
		};
		let path = format!("users/@me/guilds{}", query);
		let route_key = "GET /users/@me/guilds".to_string();
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(
		bytes: &[u8],
	) -> Result<Vec<CurrentUserGuild>, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Users
// ===========================================================================

// ---- GetCurrentUser -------------------------------------------------------

/// Get the current bot user.
///
/// ```ignore
/// let me: CurrentUser = http.send(GetCurrentUser).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetCurrentUser;

impl IntoDiscordRequest for GetCurrentUser {
	type Output = CurrentUser;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path: "users/@me".to_string(),
			route_key: "GET /users/@me".to_string(),
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<CurrentUser, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Interactions
// ===========================================================================

// ---- CreateInteractionResponse --------------------------------------------

/// Respond to an interaction (initial response).
///
/// ```ignore
/// http.send(CreateInteractionResponse::new(
///     interaction.id,
///     interaction.token.clone(),
///     InteractionResponse::pong(),
/// )).await?;
/// ```
#[derive(Debug, Clone)]
pub struct CreateInteractionResponse {
	interaction_id: Id<InteractionMarker>,
	interaction_token: String,
	response: InteractionResponse,
}

impl CreateInteractionResponse {
	pub fn new(
		interaction_id: Id<InteractionMarker>,
		interaction_token: impl Into<String>,
		response: InteractionResponse,
	) -> Self {
		Self {
			interaction_id,
			interaction_token: interaction_token.into(),
			response,
		}
	}
}

impl IntoDiscordRequest for CreateInteractionResponse {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"interactions/{}/{}/callback",
			self.interaction_id, self.interaction_token
		);
		let route_key = "POST /interactions/callback".to_string();
		let body = serde_json::to_value(&self.response)
			.map(RequestBody::Json)
			.map_err(|e| JsonError(e.to_string()))?;
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ---- EditOriginalInteractionResponse --------------------------------------

/// Edit the original interaction response (for deferred or follow-up).
///
/// ```ignore
/// let msg: Message = http.send(
///     EditOriginalInteractionResponse::new(app_id, token)
///         .content("Done!")
/// ).await?;
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct EditOriginalInteractionResponse {
	#[serde(skip)]
	application_id: Id<ApplicationMarker>,
	#[serde(skip)]
	interaction_token: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embeds: Option<Vec<Embed>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<Vec<Component>>,
}

impl EditOriginalInteractionResponse {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		interaction_token: impl Into<String>,
	) -> Self {
		Self {
			application_id,
			interaction_token: interaction_token.into(),
			content: None,
			embeds: None,
			components: None,
		}
	}

	pub fn content(mut self, text: impl Into<String>) -> Self {
		self.content = Some(text.into());
		self
	}

	pub fn embed(mut self, embed: Embed) -> Self {
		self.embeds.get_or_insert_with(Vec::new).push(embed);
		self
	}

	pub fn component_row(mut self, row: Component) -> Self {
		self.components.get_or_insert_with(Vec::new).push(row);
		self
	}
}

impl IntoDiscordRequest for EditOriginalInteractionResponse {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}/messages/@original",
			self.application_id, self.interaction_token
		);
		let route_key =
			"PATCH /webhooks/interaction/messages/@original".to_string();
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

// ===========================================================================
// Application Commands
// ===========================================================================

// ---- SetGlobalCommands ----------------------------------------------------

/// Register (or overwrite) global application commands.
///
/// ```ignore
/// let registered: Vec<ApplicationCommand> = http.send(
///     SetGlobalCommands::new(app_id, commands)
/// ).await?;
/// ```
#[derive(Debug, Clone)]
pub struct SetGlobalCommands {
	application_id: Id<ApplicationMarker>,
	commands: Vec<ApplicationCommand>,
}

impl SetGlobalCommands {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		commands: Vec<ApplicationCommand>,
	) -> Self {
		Self {
			application_id,
			commands,
		}
	}
}

impl IntoDiscordRequest for SetGlobalCommands {
	type Output = Vec<ApplicationCommand>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("applications/{}/commands", self.application_id);
		let route_key =
			format!("PUT /applications/{}/commands", self.application_id);
		let body = serde_json::to_value(&self.commands)
			.map(RequestBody::Json)
			.map_err(|e| JsonError(e.to_string()))?;
		Ok(DiscordRequest {
			method: HttpMethod::Put,
			path,
			route_key,
			body,
		})
	}

	fn parse_response(
		bytes: &[u8],
	) -> Result<Vec<ApplicationCommand>, JsonError> {
		parse_json(bytes)
	}
}

// ---- SetGuildCommands -----------------------------------------------------

/// Register (or overwrite) guild-scoped application commands.
///
/// ```ignore
/// let registered: Vec<ApplicationCommand> = http.send(
///     SetGuildCommands::new(app_id, guild_id, commands)
/// ).await?;
/// ```
#[derive(Debug, Clone)]
pub struct SetGuildCommands {
	application_id: Id<ApplicationMarker>,
	guild_id: Id<GuildMarker>,
	commands: Vec<ApplicationCommand>,
}

impl SetGuildCommands {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		guild_id: Id<GuildMarker>,
		commands: Vec<ApplicationCommand>,
	) -> Self {
		Self {
			application_id,
			guild_id,
			commands,
		}
	}
}

impl IntoDiscordRequest for SetGuildCommands {
	type Output = Vec<ApplicationCommand>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"applications/{}/guilds/{}/commands",
			self.application_id, self.guild_id
		);
		let route_key = format!(
			"PUT /applications/{}/guilds/{}/commands",
			self.application_id, self.guild_id
		);
		let body = serde_json::to_value(&self.commands)
			.map(RequestBody::Json)
			.map_err(|e| JsonError(e.to_string()))?;
		Ok(DiscordRequest {
			method: HttpMethod::Put,
			path,
			route_key,
			body,
		})
	}

	fn parse_response(
		bytes: &[u8],
	) -> Result<Vec<ApplicationCommand>, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Reactions
// ===========================================================================

// ---- CreateReaction -------------------------------------------------------

/// Add a reaction to a message.
///
/// ```ignore
/// // Unicode emoji
/// http.send(CreateReaction::new(channel_id, msg_id, "👍")).await?;
/// // Custom emoji
/// http.send(CreateReaction::new(channel_id, msg_id, "blobcat:123456789")).await?;
/// ```
#[derive(Debug, Clone)]
pub struct CreateReaction {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
	/// Unicode emoji or `name:id` for custom emoji.
	emoji: String,
}

impl CreateReaction {
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

impl IntoDiscordRequest for CreateReaction {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let encoded = url_encode_emoji(&self.emoji);
		let path = format!(
			"channels/{}/messages/{}/reactions/{}/@me",
			self.channel_id, self.message_id, encoded
		);
		let route_key =
			format!("PUT /channels/{}/messages/reactions", self.channel_id);
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

// ---- DeleteOwnReaction ----------------------------------------------------

/// Remove the bot's own reaction from a message.
///
/// ```ignore
/// http.send(DeleteOwnReaction::new(channel_id, msg_id, "👍")).await?;
/// ```
#[derive(Debug, Clone)]
pub struct DeleteOwnReaction {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
	emoji: String,
}

impl DeleteOwnReaction {
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

impl IntoDiscordRequest for DeleteOwnReaction {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let encoded = url_encode_emoji(&self.emoji);
		let path = format!(
			"channels/{}/messages/{}/reactions/{}/@me",
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

// ---- DeleteAllReactions ---------------------------------------------------

/// Remove all reactions from a message.
///
/// ```ignore
/// http.send(DeleteAllReactions::new(channel_id, msg_id)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct DeleteAllReactions {
	channel_id: Id<ChannelMarker>,
	message_id: Id<MessageMarker>,
}

impl DeleteAllReactions {
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

impl IntoDiscordRequest for DeleteAllReactions {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"channels/{}/messages/{}/reactions",
			self.channel_id, self.message_id
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

	// ---- CreateMessage ---------------------------------------------------

	#[test]
	fn create_message_builder() {
		let msg = CreateMessage::new(Id::new(100))
			.content("hello")
			.reply_to(Id::new(12345));

		assert_eq!(msg.content.as_deref(), Some("hello"));
		assert!(msg.message_reference.is_some());
		let reference = msg.message_reference.unwrap();
		assert_eq!(reference.message_id.map(|id| id.get()), Some(12345));
	}

	#[test]
	fn create_message_serializes_without_channel_id() {
		let msg = CreateMessage::new(Id::new(999)).content("test");
		let json = serde_json::to_string(&msg).unwrap();
		assert!(json.contains("\"content\":\"test\""));
		// channel_id should NOT appear in the serialised body
		assert!(!json.contains("channel_id"));
		assert!(!json.contains("999"));
		// Optional fields should be absent
		assert!(!json.contains("embeds"));
		assert!(!json.contains("components"));
		assert!(!json.contains("flags"));
	}

	#[test]
	fn create_message_into_request() {
		let msg = CreateMessage::new(Id::new(42)).content("hi");
		let req = msg.into_discord_request().unwrap();
		assert!(matches!(req.method, HttpMethod::Post));
		assert_eq!(req.path, "channels/42/messages");
		assert_eq!(req.route_key, "POST /channels/42/messages");
		assert!(matches!(req.body, RequestBody::Json(_)));
	}

	// ---- CreateMessageWithFile -------------------------------------------

	#[test]
	fn create_message_with_file_into_request() {
		let req = CreateMessageWithFile::new(
			Id::new(42),
			"test.txt",
			b"hello".to_vec(),
		)
		.content("attached")
		.into_discord_request()
		.unwrap();

		assert!(matches!(req.method, HttpMethod::Post));
		assert_eq!(req.path, "channels/42/messages");
		match &req.body {
			RequestBody::Raw { content_type, data } => {
				assert!(content_type.starts_with("multipart/form-data"));
				let body_str = String::from_utf8_lossy(data);
				assert!(body_str.contains("test.txt"));
				assert!(body_str.contains("attached"));
			}
			_ => panic!("expected Raw body"),
		}
	}

	// ---- GetChannelMessages ----------------------------------------------

	#[test]
	fn get_channel_messages_with_pagination() {
		let req = GetChannelMessages::new(Id::new(1))
			.limit(50)
			.before(Id::new(99))
			.into_discord_request()
			.unwrap();

		assert!(matches!(req.method, HttpMethod::Get));
		assert!(req.path.contains("limit=50"));
		assert!(req.path.contains("before=99"));
		assert!(matches!(req.body, RequestBody::None));
	}

	#[test]
	fn get_channel_messages_no_params() {
		let req = GetChannelMessages::new(Id::new(1))
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "channels/1/messages");
	}

	// ---- DeleteMessage ---------------------------------------------------

	#[test]
	fn delete_message_into_request() {
		let req = DeleteMessage::new(Id::new(10), Id::new(20))
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Delete));
		assert_eq!(req.path, "channels/10/messages/20");
	}

	// ---- EditMessage -----------------------------------------------------

	#[test]
	fn edit_message_into_request() {
		let req = EditMessage::new(Id::new(10), Id::new(20))
			.content("updated")
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Patch));
		assert_eq!(req.path, "channels/10/messages/20");
		match &req.body {
			RequestBody::Json(v) => {
				assert_eq!(v["content"], "updated");
				// skip fields should not be present
				assert!(v.get("channel_id").is_none());
				assert!(v.get("message_id").is_none());
			}
			_ => panic!("expected Json body"),
		}
	}

	// ---- GetChannel ------------------------------------------------------

	#[test]
	fn get_channel_into_request() {
		let req = GetChannel::new(Id::new(5)).into_discord_request().unwrap();
		assert!(matches!(req.method, HttpMethod::Get));
		assert_eq!(req.path, "channels/5");
	}

	// ---- CreateTypingTrigger ---------------------------------------------

	#[test]
	fn typing_trigger_into_request() {
		let req = CreateTypingTrigger::new(Id::new(7))
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Post));
		assert_eq!(req.path, "channels/7/typing");
	}

	// ---- GetGuild --------------------------------------------------------

	#[test]
	fn get_guild_defaults_with_counts() {
		let req = GetGuild::new(Id::new(3)).into_discord_request().unwrap();
		assert!(req.path.contains("with_counts=true"));
	}

	#[test]
	fn get_guild_without_counts() {
		let req = GetGuild::new(Id::new(3))
			.with_counts(false)
			.into_discord_request()
			.unwrap();
		assert!(!req.path.contains("with_counts"));
	}

	// ---- GetGuildChannels ------------------------------------------------

	#[test]
	fn get_guild_channels_into_request() {
		let req = GetGuildChannels::new(Id::new(3))
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "guilds/3/channels");
	}

	// ---- GetCurrentUserGuilds --------------------------------------------

	#[test]
	fn get_current_user_guilds_no_params() {
		let req = GetCurrentUserGuilds::new().into_discord_request().unwrap();
		assert_eq!(req.path, "users/@me/guilds");
	}

	#[test]
	fn get_current_user_guilds_with_limit() {
		let req = GetCurrentUserGuilds::new()
			.limit(50)
			.into_discord_request()
			.unwrap();
		assert!(req.path.contains("limit=50"));
	}

	// ---- GetCurrentUser --------------------------------------------------

	#[test]
	fn get_current_user_into_request() {
		let req = GetCurrentUser.into_discord_request().unwrap();
		assert_eq!(req.path, "users/@me");
	}

	// ---- CreateInteractionResponse ---------------------------------------

	#[test]
	fn create_interaction_response_into_request() {
		let resp = InteractionResponse::pong();
		let req = CreateInteractionResponse::new(Id::new(1), "token123", resp)
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Post));
		assert_eq!(req.path, "interactions/1/token123/callback");
	}

	// ---- SetGlobalCommands -----------------------------------------------

	#[test]
	fn set_global_commands_into_request() {
		let req = SetGlobalCommands::new(Id::new(1), vec![])
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Put));
		assert_eq!(req.path, "applications/1/commands");
	}

	// ---- SetGuildCommands ------------------------------------------------

	#[test]
	fn set_guild_commands_into_request() {
		let req = SetGuildCommands::new(Id::new(1), Id::new(2), vec![])
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Put));
		assert_eq!(req.path, "applications/1/guilds/2/commands");
	}

	// ---- CreateReaction --------------------------------------------------

	#[test]
	fn create_reaction_unicode() {
		let req = CreateReaction::new(Id::new(1), Id::new(2), "👍")
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Put));
		assert!(req.path.contains("%F0%9F%91%8D"));
		assert!(req.path.ends_with("/@me"));
	}

	#[test]
	fn create_reaction_custom() {
		let req = CreateReaction::new(Id::new(1), Id::new(2), "blob:12345")
			.into_discord_request()
			.unwrap();
		assert!(req.path.contains("blob:12345"));
	}

	// ---- DeleteAllReactions ----------------------------------------------

	#[test]
	fn delete_all_reactions_into_request() {
		let req = DeleteAllReactions::new(Id::new(1), Id::new(2))
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Delete));
		assert_eq!(req.path, "channels/1/messages/2/reactions");
	}

	// ---- Pins ------------------------------------------------------------

	#[test]
	fn get_pins_into_request() {
		let req = GetPins::new(Id::new(42)).into_discord_request().unwrap();
		assert!(matches!(req.method, HttpMethod::Get));
		assert_eq!(req.path, "channels/42/pins");
	}

	#[test]
	fn create_pin_into_request() {
		let req = CreatePin::new(Id::new(1), Id::new(2))
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Put));
		assert_eq!(req.path, "channels/1/pins/2");
	}

	#[test]
	fn delete_pin_into_request() {
		let req = DeletePin::new(Id::new(1), Id::new(2))
			.into_discord_request()
			.unwrap();
		assert!(matches!(req.method, HttpMethod::Delete));
		assert_eq!(req.path, "channels/1/pins/2");
	}
}
