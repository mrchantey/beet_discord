//! Guild, member, role, and ban request types for the Discord REST API.
//!
//! Each struct stores the data needed for one API call and implements
//! [`IntoDiscordRequest`] so it can be dispatched via
//! [`DiscordHttpClient::send`](crate::discord_io::DiscordHttpClient::send).

use crate::prelude::*;
use beet::prelude::*;
use twilight_model::channel::Channel;
use twilight_model::guild::Ban;
use twilight_model::guild::Guild;
use twilight_model::guild::GuildPreview;
use twilight_model::guild::GuildPrune;
use twilight_model::guild::Member;
use twilight_model::guild::Role;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::marker::GenericMarker;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::marker::RoleMarker;
use twilight_model::id::marker::UserMarker;
use twilight_model::user::Connection;
use twilight_model::user::User;

// ===========================================================================
// Guilds
// ===========================================================================

// ---- DeleteGuild ----------------------------------------------------------

/// Delete a guild. The bot must be the owner.
#[derive(Debug, Clone)]
pub struct DeleteGuild {
	guild_id: Id<GuildMarker>,
}

impl DeleteGuild {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for DeleteGuild {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}", self.guild_id);
		let route_key = format!("DELETE /guilds/{}", self.guild_id);
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

// ---- UpdateGuild ----------------------------------------------------------

/// Update guild settings.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateGuild {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub region: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub verification_level: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub default_message_notifications: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub explicit_content_filter: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub afk_channel_id: Option<Option<Id<ChannelMarker>>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub afk_timeout: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon: Option<Option<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner_id: Option<Id<UserMarker>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub splash: Option<Option<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub system_channel_id: Option<Option<Id<ChannelMarker>>>,
}

impl UpdateGuild {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			name: None,
			region: None,
			verification_level: None,
			default_message_notifications: None,
			explicit_content_filter: None,
			afk_channel_id: None,
			afk_timeout: None,
			icon: None,
			owner_id: None,
			splash: None,
			system_channel_id: None,
		}
	}

	/// Set the guild name.
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	/// Set the voice region.
	pub fn region(mut self, region: impl Into<String>) -> Self {
		self.region = Some(region.into());
		self
	}

	/// Set the verification level.
	pub fn verification_level(mut self, level: u8) -> Self {
		self.verification_level = Some(level);
		self
	}

	/// Set the default message notification level.
	pub fn default_message_notifications(mut self, level: u8) -> Self {
		self.default_message_notifications = Some(level);
		self
	}

	/// Set the explicit content filter level.
	pub fn explicit_content_filter(mut self, level: u8) -> Self {
		self.explicit_content_filter = Some(level);
		self
	}

	/// Set the AFK channel.
	pub fn afk_channel_id(mut self, id: Option<Id<ChannelMarker>>) -> Self {
		self.afk_channel_id = Some(id);
		self
	}

	/// Set the AFK timeout in seconds.
	pub fn afk_timeout(mut self, timeout: u32) -> Self {
		self.afk_timeout = Some(timeout);
		self
	}

	/// Set the guild icon (base64-encoded image data, or None to remove).
	pub fn icon(mut self, icon: Option<String>) -> Self {
		self.icon = Some(icon);
		self
	}

	/// Transfer ownership to another user.
	pub fn owner_id(mut self, id: Id<UserMarker>) -> Self {
		self.owner_id = Some(id);
		self
	}

	/// Set the guild splash (base64-encoded image data, or None to remove).
	pub fn splash(mut self, splash: Option<String>) -> Self {
		self.splash = Some(splash);
		self
	}

	/// Set the system channel.
	pub fn system_channel_id(mut self, id: Option<Id<ChannelMarker>>) -> Self {
		self.system_channel_id = Some(id);
		self
	}
}

impl IntoDiscordRequest for UpdateGuild {
	type Output = Guild;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}", self.guild_id);
		let route_key = format!("PATCH /guilds/{}", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Guild, JsonError> {
		parse_json(bytes)
	}
}

// ---- LeaveGuild -----------------------------------------------------------

/// Leave a guild.
#[derive(Debug, Clone)]
pub struct LeaveGuild {
	guild_id: Id<GuildMarker>,
}

impl LeaveGuild {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for LeaveGuild {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("users/@me/guilds/{}", self.guild_id);
		let route_key = format!("DELETE /users/@me/guilds/{}", self.guild_id);
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

// ---- CreateGuildChannel ---------------------------------------------------

/// Create a new channel in a guild.
#[derive(Debug, Clone, Serialize)]
pub struct CreateGuildChannel {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	pub name: String,
	#[serde(rename = "type")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub kind: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub topic: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bitrate: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user_limit: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rate_limit_per_user: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub position: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parent_id: Option<Id<ChannelMarker>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub nsfw: Option<bool>,
}

impl CreateGuildChannel {
	pub fn new(guild_id: Id<GuildMarker>, name: impl Into<String>) -> Self {
		Self {
			guild_id,
			name: name.into(),
			kind: None,
			topic: None,
			bitrate: None,
			user_limit: None,
			rate_limit_per_user: None,
			position: None,
			parent_id: None,
			nsfw: None,
		}
	}

	/// Set the channel type (0 = text, 2 = voice, 4 = category, …).
	pub fn kind(mut self, kind: u8) -> Self {
		self.kind = Some(kind);
		self
	}

	/// Set the channel topic.
	pub fn topic(mut self, topic: impl Into<String>) -> Self {
		self.topic = Some(topic.into());
		self
	}

	/// Set the bitrate (voice channels).
	pub fn bitrate(mut self, bitrate: u32) -> Self {
		self.bitrate = Some(bitrate);
		self
	}

	/// Set the user limit (voice channels).
	pub fn user_limit(mut self, limit: u16) -> Self {
		self.user_limit = Some(limit);
		self
	}

	/// Set the rate limit per user (slowmode) in seconds.
	pub fn rate_limit_per_user(mut self, seconds: u16) -> Self {
		self.rate_limit_per_user = Some(seconds);
		self
	}

	/// Set the sorting position.
	pub fn position(mut self, position: u16) -> Self {
		self.position = Some(position);
		self
	}

	/// Set the parent category channel.
	pub fn parent_id(mut self, id: Id<ChannelMarker>) -> Self {
		self.parent_id = Some(id);
		self
	}

	/// Set whether the channel is NSFW.
	pub fn nsfw(mut self, nsfw: bool) -> Self {
		self.nsfw = Some(nsfw);
		self
	}
}

impl IntoDiscordRequest for CreateGuildChannel {
	type Output = Channel;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/channels", self.guild_id);
		let route_key = format!("POST /guilds/{}/channels", self.guild_id);
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

// ---- GetGuildPreview ------------------------------------------------------

/// Get a guild preview.
#[derive(Debug, Clone)]
pub struct GetGuildPreview {
	guild_id: Id<GuildMarker>,
}

impl GetGuildPreview {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildPreview {
	type Output = GuildPreview;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/preview", self.guild_id);
		let route_key = format!("GET /guilds/{}/preview", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<GuildPreview, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildWebhooks -----------------------------------------------------

/// Get all webhooks in a guild.
#[derive(Debug, Clone)]
pub struct GetGuildWebhooks {
	guild_id: Id<GuildMarker>,
}

impl GetGuildWebhooks {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildWebhooks {
	type Output = Vec<serde_json::Value>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/webhooks", self.guild_id);
		let route_key = format!("GET /guilds/{}/webhooks", self.guild_id);
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
// Guild Members
// ===========================================================================

// ---- GetGuildMembers ------------------------------------------------------

/// List members of a guild with optional pagination.
#[derive(Debug, Clone)]
pub struct GetGuildMembers {
	guild_id: Id<GuildMarker>,
	limit: Option<u16>,
	after: Option<Id<UserMarker>>,
}

impl GetGuildMembers {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			limit: None,
			after: None,
		}
	}

	/// Maximum number of members to return (1–1000, default 1).
	pub fn limit(mut self, limit: u16) -> Self {
		self.limit = Some(limit.min(1000));
		self
	}

	/// Get members after this user ID.
	pub fn after(mut self, id: Id<UserMarker>) -> Self {
		self.after = Some(id);
		self
	}
}

impl IntoDiscordRequest for GetGuildMembers {
	type Output = Vec<Member>;

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
		let path = format!("guilds/{}/members{}", self.guild_id, query);
		let route_key = format!("GET /guilds/{}/members", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Member>, JsonError> {
		parse_json(bytes)
	}
}

// ---- SearchGuildMembers ---------------------------------------------------

/// Search guild members by username or nickname.
#[derive(Debug, Clone)]
pub struct SearchGuildMembers {
	guild_id: Id<GuildMarker>,
	query: String,
	limit: Option<u16>,
}

impl SearchGuildMembers {
	pub fn new(guild_id: Id<GuildMarker>, query: impl Into<String>) -> Self {
		Self {
			guild_id,
			query: query.into(),
			limit: None,
		}
	}

	/// Maximum number of members to return (1–1000, default 1).
	pub fn limit(mut self, limit: u16) -> Self {
		self.limit = Some(limit.min(1000));
		self
	}
}

impl IntoDiscordRequest for SearchGuildMembers {
	type Output = Vec<Member>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let mut query_parts = vec![format!("query={}", self.query)];
		if let Some(limit) = self.limit {
			query_parts.push(format!("limit={}", limit));
		}
		let qs = format!("?{}", query_parts.join("&"));
		let path = format!("guilds/{}/members/search{}", self.guild_id, qs);
		let route_key = format!("GET /guilds/{}/members/search", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Member>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildMember -------------------------------------------------------

/// Get a single guild member.
#[derive(Debug, Clone)]
pub struct GetGuildMember {
	guild_id: Id<GuildMarker>,
	user_id: Id<UserMarker>,
}

impl GetGuildMember {
	pub fn new(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>) -> Self {
		Self { guild_id, user_id }
	}
}

impl IntoDiscordRequest for GetGuildMember {
	type Output = Member;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/members/{}", self.guild_id, self.user_id);
		let route_key = format!("GET /guilds/{}/members", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Member, JsonError> {
		parse_json(bytes)
	}
}

// ---- RemoveGuildMember ----------------------------------------------------

/// Remove (kick) a member from a guild.
#[derive(Debug, Clone)]
pub struct RemoveGuildMember {
	guild_id: Id<GuildMarker>,
	user_id: Id<UserMarker>,
}

impl RemoveGuildMember {
	pub fn new(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>) -> Self {
		Self { guild_id, user_id }
	}
}

impl IntoDiscordRequest for RemoveGuildMember {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/members/{}", self.guild_id, self.user_id);
		let route_key = format!("DELETE /guilds/{}/members", self.guild_id);
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

// ---- UpdateGuildMember ----------------------------------------------------

/// Update attributes of a guild member.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateGuildMember {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip)]
	user_id: Id<UserMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub nick: Option<Option<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub roles: Option<Option<Vec<Id<RoleMarker>>>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mute: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub deaf: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub channel_id: Option<Option<Id<ChannelMarker>>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub communication_disabled_until: Option<Option<String>>,
}

impl UpdateGuildMember {
	pub fn new(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>) -> Self {
		Self {
			guild_id,
			user_id,
			nick: None,
			roles: None,
			mute: None,
			deaf: None,
			channel_id: None,
			communication_disabled_until: None,
		}
	}

	/// Set the member nickname (None to remove).
	pub fn nick(mut self, nick: Option<String>) -> Self {
		self.nick = Some(nick);
		self
	}

	/// Set the member roles (None to remove all).
	pub fn roles(mut self, roles: Option<Vec<Id<RoleMarker>>>) -> Self {
		self.roles = Some(roles);
		self
	}

	/// Server-mute the member.
	pub fn mute(mut self, mute: bool) -> Self {
		self.mute = Some(mute);
		self
	}

	/// Server-deafen the member.
	pub fn deaf(mut self, deaf: bool) -> Self {
		self.deaf = Some(deaf);
		self
	}

	/// Move the member to a voice channel (None to disconnect).
	pub fn channel_id(mut self, id: Option<Id<ChannelMarker>>) -> Self {
		self.channel_id = Some(id);
		self
	}

	/// Set the timeout expiry (ISO 8601 timestamp, or None to remove).
	pub fn communication_disabled_until(
		mut self,
		until: Option<String>,
	) -> Self {
		self.communication_disabled_until = Some(until);
		self
	}
}

impl IntoDiscordRequest for UpdateGuildMember {
	type Output = Member;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/members/{}", self.guild_id, self.user_id);
		let route_key = format!("PATCH /guilds/{}/members", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Member, JsonError> {
		parse_json(bytes)
	}
}

// ---- UpdateCurrentMember --------------------------------------------------

/// Update the current bot user's nickname in a guild.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateCurrentMember {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub nick: Option<Option<String>>,
}

impl UpdateCurrentMember {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			nick: None,
		}
	}

	/// Set the bot's nickname in this guild (None to remove).
	pub fn nick(mut self, nick: Option<String>) -> Self {
		self.nick = Some(nick);
		self
	}
}

impl IntoDiscordRequest for UpdateCurrentMember {
	type Output = Member;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/members/@me", self.guild_id);
		let route_key = format!("PATCH /guilds/{}/members/@me", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Member, JsonError> {
		parse_json(bytes)
	}
}

// ---- AddGuildMemberRole ---------------------------------------------------

/// Add a role to a guild member.
#[derive(Debug, Clone)]
pub struct AddGuildMemberRole {
	guild_id: Id<GuildMarker>,
	user_id: Id<UserMarker>,
	role_id: Id<RoleMarker>,
}

impl AddGuildMemberRole {
	pub fn new(
		guild_id: Id<GuildMarker>,
		user_id: Id<UserMarker>,
		role_id: Id<RoleMarker>,
	) -> Self {
		Self {
			guild_id,
			user_id,
			role_id,
		}
	}
}

impl IntoDiscordRequest for AddGuildMemberRole {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"guilds/{}/members/{}/roles/{}",
			self.guild_id, self.user_id, self.role_id
		);
		let route_key = format!("PUT /guilds/{}/members/roles", self.guild_id);
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

// ---- RemoveGuildMemberRole ------------------------------------------------

/// Remove a role from a guild member.
#[derive(Debug, Clone)]
pub struct RemoveGuildMemberRole {
	guild_id: Id<GuildMarker>,
	user_id: Id<UserMarker>,
	role_id: Id<RoleMarker>,
}

impl RemoveGuildMemberRole {
	pub fn new(
		guild_id: Id<GuildMarker>,
		user_id: Id<UserMarker>,
		role_id: Id<RoleMarker>,
	) -> Self {
		Self {
			guild_id,
			user_id,
			role_id,
		}
	}
}

impl IntoDiscordRequest for RemoveGuildMemberRole {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"guilds/{}/members/{}/roles/{}",
			self.guild_id, self.user_id, self.role_id
		);
		let route_key =
			format!("DELETE /guilds/{}/members/roles", self.guild_id);
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
// Roles
// ===========================================================================

// ---- GetGuildRoles --------------------------------------------------------

/// Get all roles in a guild.
#[derive(Debug, Clone)]
pub struct GetGuildRoles {
	guild_id: Id<GuildMarker>,
}

impl GetGuildRoles {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildRoles {
	type Output = Vec<Role>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/roles", self.guild_id);
		let route_key = format!("GET /guilds/{}/roles", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Role>, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateGuildRole ------------------------------------------------------

/// Create a new role in a guild.
#[derive(Debug, Clone, Serialize)]
pub struct CreateGuildRole {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub permissions: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub color: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hoist: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mentionable: Option<bool>,
}

impl CreateGuildRole {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			name: None,
			permissions: None,
			color: None,
			hoist: None,
			mentionable: None,
		}
	}

	/// Set the role name.
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	/// Set the role permissions (stringified bitfield).
	pub fn permissions(mut self, permissions: impl Into<String>) -> Self {
		self.permissions = Some(permissions.into());
		self
	}

	/// Set the role colour (RGB integer).
	pub fn color(mut self, color: u32) -> Self {
		self.color = Some(color);
		self
	}

	/// Whether the role should be displayed separately in the sidebar.
	pub fn hoist(mut self, hoist: bool) -> Self {
		self.hoist = Some(hoist);
		self
	}

	/// Whether the role should be mentionable.
	pub fn mentionable(mut self, mentionable: bool) -> Self {
		self.mentionable = Some(mentionable);
		self
	}
}

impl IntoDiscordRequest for CreateGuildRole {
	type Output = Role;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/roles", self.guild_id);
		let route_key = format!("POST /guilds/{}/roles", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Role, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteGuildRole ------------------------------------------------------

/// Delete a role from a guild.
#[derive(Debug, Clone)]
pub struct DeleteGuildRole {
	guild_id: Id<GuildMarker>,
	role_id: Id<RoleMarker>,
}

impl DeleteGuildRole {
	pub fn new(guild_id: Id<GuildMarker>, role_id: Id<RoleMarker>) -> Self {
		Self { guild_id, role_id }
	}
}

impl IntoDiscordRequest for DeleteGuildRole {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/roles/{}", self.guild_id, self.role_id);
		let route_key = format!("DELETE /guilds/{}/roles", self.guild_id);
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

// ---- UpdateGuildRole ------------------------------------------------------

/// Update an existing role in a guild.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateGuildRole {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip)]
	role_id: Id<RoleMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub permissions: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub color: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hoist: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mentionable: Option<bool>,
}

impl UpdateGuildRole {
	pub fn new(guild_id: Id<GuildMarker>, role_id: Id<RoleMarker>) -> Self {
		Self {
			guild_id,
			role_id,
			name: None,
			permissions: None,
			color: None,
			hoist: None,
			mentionable: None,
		}
	}

	/// Set the role name.
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	/// Set the role permissions (stringified bitfield).
	pub fn permissions(mut self, permissions: impl Into<String>) -> Self {
		self.permissions = Some(permissions.into());
		self
	}

	/// Set the role colour (RGB integer).
	pub fn color(mut self, color: u32) -> Self {
		self.color = Some(color);
		self
	}

	/// Whether the role should be displayed separately in the sidebar.
	pub fn hoist(mut self, hoist: bool) -> Self {
		self.hoist = Some(hoist);
		self
	}

	/// Whether the role should be mentionable.
	pub fn mentionable(mut self, mentionable: bool) -> Self {
		self.mentionable = Some(mentionable);
		self
	}
}

impl IntoDiscordRequest for UpdateGuildRole {
	type Output = Role;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/roles/{}", self.guild_id, self.role_id);
		let route_key = format!("PATCH /guilds/{}/roles", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Role, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Bans
// ===========================================================================

// ---- GetGuildBans ---------------------------------------------------------

/// Get guild bans with optional pagination.
#[derive(Debug, Clone)]
pub struct GetGuildBans {
	guild_id: Id<GuildMarker>,
	limit: Option<u16>,
	before: Option<Id<UserMarker>>,
	after: Option<Id<UserMarker>>,
}

impl GetGuildBans {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			limit: None,
			before: None,
			after: None,
		}
	}

	/// Maximum number of bans to return (1–1000, default 1000).
	pub fn limit(mut self, limit: u16) -> Self {
		self.limit = Some(limit.min(1000));
		self
	}

	/// Get bans for users before this user ID.
	pub fn before(mut self, id: Id<UserMarker>) -> Self {
		self.before = Some(id);
		self
	}

	/// Get bans for users after this user ID.
	pub fn after(mut self, id: Id<UserMarker>) -> Self {
		self.after = Some(id);
		self
	}
}

impl IntoDiscordRequest for GetGuildBans {
	type Output = Vec<Ban>;

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
		let path = format!("guilds/{}/bans{}", self.guild_id, query);
		let route_key = format!("GET /guilds/{}/bans", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Ban>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildBan ----------------------------------------------------------

/// Get a single guild ban by user ID.
#[derive(Debug, Clone)]
pub struct GetGuildBan {
	guild_id: Id<GuildMarker>,
	user_id: Id<UserMarker>,
}

impl GetGuildBan {
	pub fn new(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>) -> Self {
		Self { guild_id, user_id }
	}
}

impl IntoDiscordRequest for GetGuildBan {
	type Output = Ban;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/bans/{}", self.guild_id, self.user_id);
		let route_key = format!("GET /guilds/{}/bans", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Ban, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateGuildBan -------------------------------------------------------

/// Ban a user from a guild.
#[derive(Debug, Clone, Serialize)]
pub struct CreateGuildBan {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip)]
	user_id: Id<UserMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub delete_message_seconds: Option<u32>,
}

impl CreateGuildBan {
	pub fn new(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>) -> Self {
		Self {
			guild_id,
			user_id,
			delete_message_seconds: None,
		}
	}

	/// Number of seconds to delete messages for (0–604800).
	pub fn delete_message_seconds(mut self, seconds: u32) -> Self {
		self.delete_message_seconds = Some(seconds);
		self
	}
}

impl IntoDiscordRequest for CreateGuildBan {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/bans/{}", self.guild_id, self.user_id);
		let route_key = format!("PUT /guilds/{}/bans", self.guild_id);
		let body = if self.delete_message_seconds.is_some() {
			json_body(&self)?
		} else {
			RequestBody::None
		};
		Ok(DiscordRequest {
			method: HttpMethod::Put,
			path,
			route_key,
			body,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<(), JsonError> {
		parse_empty(bytes)
	}
}

// ---- DeleteGuildBan -------------------------------------------------------

/// Remove a ban from a guild.
#[derive(Debug, Clone)]
pub struct DeleteGuildBan {
	guild_id: Id<GuildMarker>,
	user_id: Id<UserMarker>,
}

impl DeleteGuildBan {
	pub fn new(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>) -> Self {
		Self { guild_id, user_id }
	}
}

impl IntoDiscordRequest for DeleteGuildBan {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/bans/{}", self.guild_id, self.user_id);
		let route_key = format!("DELETE /guilds/{}/bans", self.guild_id);
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
// Current User extras
// ===========================================================================

// ---- UpdateCurrentUser ----------------------------------------------------

/// Update the current bot user.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateCurrentUser {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub username: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub avatar: Option<Option<String>>,
}

impl UpdateCurrentUser {
	pub fn new() -> Self {
		Self {
			username: None,
			avatar: None,
		}
	}

	/// Set the bot username.
	pub fn username(mut self, username: impl Into<String>) -> Self {
		self.username = Some(username.into());
		self
	}

	/// Set the bot avatar (base64-encoded image data, or None to remove).
	pub fn avatar(mut self, avatar: Option<String>) -> Self {
		self.avatar = Some(avatar);
		self
	}
}

impl Default for UpdateCurrentUser {
	fn default() -> Self { Self::new() }
}

impl IntoDiscordRequest for UpdateCurrentUser {
	type Output = User;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		Ok(DiscordRequest {
			method: HttpMethod::Patch,
			path: "users/@me".to_string(),
			route_key: "PATCH /users/@me".to_string(),
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<User, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetCurrentUserGuildMember --------------------------------------------

/// Get the current bot user's member object in a guild.
#[derive(Debug, Clone)]
pub struct GetCurrentUserGuildMember {
	guild_id: Id<GuildMarker>,
}

impl GetCurrentUserGuildMember {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetCurrentUserGuildMember {
	type Output = Member;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("users/@me/guilds/{}/member", self.guild_id);
		let route_key =
			format!("GET /users/@me/guilds/{}/member", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Member, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetCurrentUserConnections --------------------------------------------

/// Get the current user's connections.
#[derive(Debug, Clone)]
pub struct GetCurrentUserConnections;

impl IntoDiscordRequest for GetCurrentUserConnections {
	type Output = Vec<Connection>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path: "users/@me/connections".to_string(),
			route_key: "GET /users/@me/connections".to_string(),
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Vec<Connection>, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreatePrivateChannel -------------------------------------------------

/// Open a DM channel with a user.
#[derive(Debug, Clone, Serialize)]
pub struct CreatePrivateChannel {
	pub recipient_id: Id<UserMarker>,
}

impl CreatePrivateChannel {
	pub fn new(recipient_id: Id<UserMarker>) -> Self { Self { recipient_id } }
}

impl IntoDiscordRequest for CreatePrivateChannel {
	type Output = Channel;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path: "users/@me/channels".to_string(),
			route_key: "POST /users/@me/channels".to_string(),
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<Channel, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetUser --------------------------------------------------------------

/// Get a user by ID.
#[derive(Debug, Clone)]
pub struct GetUser {
	user_id: Id<UserMarker>,
}

impl GetUser {
	pub fn new(user_id: Id<UserMarker>) -> Self { Self { user_id } }
}

impl IntoDiscordRequest for GetUser {
	type Output = User;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("users/{}", self.user_id);
		let route_key = format!("GET /users/{}", self.user_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<User, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Guild Misc
// ===========================================================================

// ---- GetGuildPruneCount ---------------------------------------------------

/// Get the number of members that would be pruned.
#[derive(Debug, Clone)]
pub struct GetGuildPruneCount {
	guild_id: Id<GuildMarker>,
	days: Option<u16>,
	include_roles: Option<Vec<Id<RoleMarker>>>,
}

impl GetGuildPruneCount {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			days: None,
			include_roles: None,
		}
	}

	/// Number of days to count prune for (1–30, default 7).
	pub fn days(mut self, days: u16) -> Self {
		self.days = Some(days);
		self
	}

	/// Include members with these roles in the prune count.
	pub fn include_roles(mut self, roles: Vec<Id<RoleMarker>>) -> Self {
		self.include_roles = Some(roles);
		self
	}
}

impl IntoDiscordRequest for GetGuildPruneCount {
	type Output = GuildPrune;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let mut query_parts = Vec::new();
		if let Some(days) = self.days {
			query_parts.push(format!("days={}", days));
		}
		if let Some(roles) = self.include_roles {
			let ids: Vec<String> =
				roles.iter().map(|r| r.to_string()).collect();
			query_parts.push(format!("include_roles={}", ids.join(",")));
		}
		let query = if query_parts.is_empty() {
			String::new()
		} else {
			format!("?{}", query_parts.join("&"))
		};
		let path = format!("guilds/{}/prune{}", self.guild_id, query);
		let route_key = format!("GET /guilds/{}/prune", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<GuildPrune, JsonError> {
		parse_json(bytes)
	}
}

// ---- CreateGuildPrune -----------------------------------------------------

/// Begin a guild prune operation.
#[derive(Debug, Clone, Serialize)]
pub struct CreateGuildPrune {
	#[serde(skip)]
	guild_id: Id<GuildMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub days: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub compute_prune_count: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub include_roles: Option<Vec<Id<RoleMarker>>>,
}

impl CreateGuildPrune {
	pub fn new(guild_id: Id<GuildMarker>) -> Self {
		Self {
			guild_id,
			days: None,
			compute_prune_count: None,
			include_roles: None,
		}
	}

	/// Number of days to prune (1–30, default 7).
	pub fn days(mut self, days: u16) -> Self {
		self.days = Some(days);
		self
	}

	/// Whether to return the `pruned` count in the response.
	pub fn compute_prune_count(mut self, compute: bool) -> Self {
		self.compute_prune_count = Some(compute);
		self
	}

	/// Include members with these roles in the prune.
	pub fn include_roles(mut self, roles: Vec<Id<RoleMarker>>) -> Self {
		self.include_roles = Some(roles);
		self
	}
}

impl IntoDiscordRequest for CreateGuildPrune {
	type Output = GuildPrune;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/prune", self.guild_id);
		let route_key = format!("POST /guilds/{}/prune", self.guild_id);
		Ok(DiscordRequest {
			method: HttpMethod::Post,
			path,
			route_key,
			body: json_body(&self)?,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<GuildPrune, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildIntegrations -------------------------------------------------

/// Get guild integrations.
#[derive(Debug, Clone)]
pub struct GetGuildIntegrations {
	guild_id: Id<GuildMarker>,
}

impl GetGuildIntegrations {
	pub fn new(guild_id: Id<GuildMarker>) -> Self { Self { guild_id } }
}

impl IntoDiscordRequest for GetGuildIntegrations {
	type Output = Vec<serde_json::Value>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("guilds/{}/integrations", self.guild_id);
		let route_key = format!("GET /guilds/{}/integrations", self.guild_id);
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

// ---- DeleteGuildIntegration -----------------------------------------------

/// Delete a guild integration.
#[derive(Debug, Clone)]
pub struct DeleteGuildIntegration {
	guild_id: Id<GuildMarker>,
	integration_id: Id<GenericMarker>,
}

impl DeleteGuildIntegration {
	pub fn new(
		guild_id: Id<GuildMarker>,
		integration_id: Id<GenericMarker>,
	) -> Self {
		Self {
			guild_id,
			integration_id,
		}
	}
}

impl IntoDiscordRequest for DeleteGuildIntegration {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"guilds/{}/integrations/{}",
			self.guild_id, self.integration_id
		);
		let route_key =
			format!("DELETE /guilds/{}/integrations", self.guild_id);
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
