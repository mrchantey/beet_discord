//! Interaction response, followup, and application command request types for
//! the Discord REST API.
//!
//! These complement the core interaction types in [`super::requests`] with
//! the remaining endpoints for interaction responses, followup messages,
//! global/guild commands, and command permissions.

use crate::prelude::*;
use beet::prelude::*;
use twilight_model::application::command::Command as ApplicationCommand;
use twilight_model::application::command::permissions::GuildCommandPermissions;
use twilight_model::channel::message::Message;
use twilight_model::channel::message::component::Component;
use twilight_model::channel::message::embed::Embed;
use twilight_model::id::Id;
use twilight_model::id::marker::ApplicationMarker;
use twilight_model::id::marker::CommandMarker;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::marker::MessageMarker;

// ===========================================================================
// Interaction Responses
// ===========================================================================

// ---- GetOriginalInteractionResponse ---------------------------------------

/// Fetch the original interaction response message.
#[derive(Debug, Clone)]
pub struct GetOriginalInteractionResponse {
	application_id: Id<ApplicationMarker>,
	interaction_token: String,
}

impl GetOriginalInteractionResponse {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		interaction_token: impl Into<String>,
	) -> Self {
		Self {
			application_id,
			interaction_token: interaction_token.into(),
		}
	}
}

impl IntoDiscordRequest for GetOriginalInteractionResponse {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}/messages/@original",
			self.application_id, self.interaction_token
		);
		let route_key =
			"GET /webhooks/interaction/messages/@original".to_string();
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

// ---- DeleteOriginalInteractionResponse ------------------------------------

/// Delete the original interaction response.
#[derive(Debug, Clone)]
pub struct DeleteOriginalInteractionResponse {
	application_id: Id<ApplicationMarker>,
	interaction_token: String,
}

impl DeleteOriginalInteractionResponse {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		interaction_token: impl Into<String>,
	) -> Self {
		Self {
			application_id,
			interaction_token: interaction_token.into(),
		}
	}
}

impl IntoDiscordRequest for DeleteOriginalInteractionResponse {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}/messages/@original",
			self.application_id, self.interaction_token
		);
		let route_key =
			"DELETE /webhooks/interaction/messages/@original".to_string();
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
// Followups
// ===========================================================================

// ---- CreateFollowup -------------------------------------------------------

/// Create a followup message for an interaction.
#[derive(Debug, Clone, Serialize)]
pub struct CreateFollowup {
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
	#[serde(skip_serializing_if = "Option::is_none")]
	pub flags: Option<u64>,
}

impl CreateFollowup {
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

	pub fn flags(mut self, flags: u64) -> Self {
		self.flags = Some(flags);
		self
	}
}

impl IntoDiscordRequest for CreateFollowup {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}",
			self.application_id, self.interaction_token
		);
		let route_key = "POST /webhooks/interaction".to_string();
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

// ---- GetFollowup ----------------------------------------------------------

/// Fetch a followup message for an interaction.
#[derive(Debug, Clone)]
pub struct GetFollowup {
	application_id: Id<ApplicationMarker>,
	interaction_token: String,
	message_id: Id<MessageMarker>,
}

impl GetFollowup {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		interaction_token: impl Into<String>,
		message_id: Id<MessageMarker>,
	) -> Self {
		Self {
			application_id,
			interaction_token: interaction_token.into(),
			message_id,
		}
	}
}

impl IntoDiscordRequest for GetFollowup {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}/messages/{}",
			self.application_id, self.interaction_token, self.message_id
		);
		let route_key = "GET /webhooks/interaction/messages".to_string();
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

// ---- UpdateFollowup -------------------------------------------------------

/// Edit a followup message for an interaction.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateFollowup {
	#[serde(skip)]
	application_id: Id<ApplicationMarker>,
	#[serde(skip)]
	interaction_token: String,
	#[serde(skip)]
	message_id: Id<MessageMarker>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embeds: Option<Vec<Embed>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<Vec<Component>>,
}

impl UpdateFollowup {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		interaction_token: impl Into<String>,
		message_id: Id<MessageMarker>,
	) -> Self {
		Self {
			application_id,
			interaction_token: interaction_token.into(),
			message_id,
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

impl IntoDiscordRequest for UpdateFollowup {
	type Output = Message;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}/messages/{}",
			self.application_id, self.interaction_token, self.message_id
		);
		let route_key = "PATCH /webhooks/interaction/messages".to_string();
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

// ---- DeleteFollowup -------------------------------------------------------

/// Delete a followup message for an interaction.
#[derive(Debug, Clone)]
pub struct DeleteFollowup {
	application_id: Id<ApplicationMarker>,
	interaction_token: String,
	message_id: Id<MessageMarker>,
}

impl DeleteFollowup {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		interaction_token: impl Into<String>,
		message_id: Id<MessageMarker>,
	) -> Self {
		Self {
			application_id,
			interaction_token: interaction_token.into(),
			message_id,
		}
	}
}

impl IntoDiscordRequest for DeleteFollowup {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"webhooks/{}/{}/messages/{}",
			self.application_id, self.interaction_token, self.message_id
		);
		let route_key = "DELETE /webhooks/interaction/messages".to_string();
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
// Global Application Commands
// ===========================================================================

// ---- GetGlobalCommands ----------------------------------------------------

/// Fetch all global application commands.
#[derive(Debug, Clone)]
pub struct GetGlobalCommands {
	application_id: Id<ApplicationMarker>,
}

impl GetGlobalCommands {
	pub fn new(application_id: Id<ApplicationMarker>) -> Self {
		Self { application_id }
	}
}

impl IntoDiscordRequest for GetGlobalCommands {
	type Output = Vec<ApplicationCommand>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!("applications/{}/commands", self.application_id);
		let route_key =
			format!("GET /applications/{}/commands", self.application_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(
		bytes: &[u8],
	) -> Result<Vec<ApplicationCommand>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGlobalCommand -----------------------------------------------------

/// Fetch a single global application command by ID.
#[derive(Debug, Clone)]
pub struct GetGlobalCommand {
	application_id: Id<ApplicationMarker>,
	command_id: Id<CommandMarker>,
}

impl GetGlobalCommand {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		command_id: Id<CommandMarker>,
	) -> Self {
		Self {
			application_id,
			command_id,
		}
	}
}

impl IntoDiscordRequest for GetGlobalCommand {
	type Output = ApplicationCommand;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"applications/{}/commands/{}",
			self.application_id, self.command_id
		);
		let route_key =
			format!("GET /applications/{}/commands", self.application_id);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<ApplicationCommand, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteGlobalCommand --------------------------------------------------

/// Delete a global application command.
#[derive(Debug, Clone)]
pub struct DeleteGlobalCommand {
	application_id: Id<ApplicationMarker>,
	command_id: Id<CommandMarker>,
}

impl DeleteGlobalCommand {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		command_id: Id<CommandMarker>,
	) -> Self {
		Self {
			application_id,
			command_id,
		}
	}
}

impl IntoDiscordRequest for DeleteGlobalCommand {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"applications/{}/commands/{}",
			self.application_id, self.command_id
		);
		let route_key =
			format!("DELETE /applications/{}/commands", self.application_id);
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
// Guild Application Commands
// ===========================================================================

// ---- GetGuildCommands -----------------------------------------------------

/// Fetch all guild-scoped application commands.
#[derive(Debug, Clone)]
pub struct GetGuildCommands {
	application_id: Id<ApplicationMarker>,
	guild_id: Id<GuildMarker>,
}

impl GetGuildCommands {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		guild_id: Id<GuildMarker>,
	) -> Self {
		Self {
			application_id,
			guild_id,
		}
	}
}

impl IntoDiscordRequest for GetGuildCommands {
	type Output = Vec<ApplicationCommand>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"applications/{}/guilds/{}/commands",
			self.application_id, self.guild_id
		);
		let route_key = format!(
			"GET /applications/{}/guilds/{}/commands",
			self.application_id, self.guild_id
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
	) -> Result<Vec<ApplicationCommand>, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildCommand ------------------------------------------------------

/// Fetch a single guild-scoped application command by ID.
#[derive(Debug, Clone)]
pub struct GetGuildCommand {
	application_id: Id<ApplicationMarker>,
	guild_id: Id<GuildMarker>,
	command_id: Id<CommandMarker>,
}

impl GetGuildCommand {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		guild_id: Id<GuildMarker>,
		command_id: Id<CommandMarker>,
	) -> Self {
		Self {
			application_id,
			guild_id,
			command_id,
		}
	}
}

impl IntoDiscordRequest for GetGuildCommand {
	type Output = ApplicationCommand;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"applications/{}/guilds/{}/commands/{}",
			self.application_id, self.guild_id, self.command_id
		);
		let route_key = format!(
			"GET /applications/{}/guilds/{}/commands",
			self.application_id, self.guild_id
		);
		Ok(DiscordRequest {
			method: HttpMethod::Get,
			path,
			route_key,
			body: RequestBody::None,
		})
	}

	fn parse_response(bytes: &[u8]) -> Result<ApplicationCommand, JsonError> {
		parse_json(bytes)
	}
}

// ---- DeleteGuildCommand ---------------------------------------------------

/// Delete a guild-scoped application command.
#[derive(Debug, Clone)]
pub struct DeleteGuildCommand {
	application_id: Id<ApplicationMarker>,
	guild_id: Id<GuildMarker>,
	command_id: Id<CommandMarker>,
}

impl DeleteGuildCommand {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		guild_id: Id<GuildMarker>,
		command_id: Id<CommandMarker>,
	) -> Self {
		Self {
			application_id,
			guild_id,
			command_id,
		}
	}
}

impl IntoDiscordRequest for DeleteGuildCommand {
	type Output = ();

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"applications/{}/guilds/{}/commands/{}",
			self.application_id, self.guild_id, self.command_id
		);
		let route_key = format!(
			"DELETE /applications/{}/guilds/{}/commands",
			self.application_id, self.guild_id
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
// Command Permissions
// ===========================================================================

// ---- GetCommandPermissions ------------------------------------------------

/// Fetch permissions for a specific command in a guild.
#[derive(Debug, Clone)]
pub struct GetCommandPermissions {
	application_id: Id<ApplicationMarker>,
	guild_id: Id<GuildMarker>,
	command_id: Id<CommandMarker>,
}

impl GetCommandPermissions {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		guild_id: Id<GuildMarker>,
		command_id: Id<CommandMarker>,
	) -> Self {
		Self {
			application_id,
			guild_id,
			command_id,
		}
	}
}

impl IntoDiscordRequest for GetCommandPermissions {
	type Output = GuildCommandPermissions;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"applications/{}/guilds/{}/commands/{}/permissions",
			self.application_id, self.guild_id, self.command_id
		);
		let route_key = format!(
			"GET /applications/{}/guilds/{}/commands/permissions",
			self.application_id, self.guild_id
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
	) -> Result<GuildCommandPermissions, JsonError> {
		parse_json(bytes)
	}
}

// ---- GetGuildCommandPermissions -------------------------------------------

/// Fetch permissions for all commands in a guild.
#[derive(Debug, Clone)]
pub struct GetGuildCommandPermissions {
	application_id: Id<ApplicationMarker>,
	guild_id: Id<GuildMarker>,
}

impl GetGuildCommandPermissions {
	pub fn new(
		application_id: Id<ApplicationMarker>,
		guild_id: Id<GuildMarker>,
	) -> Self {
		Self {
			application_id,
			guild_id,
		}
	}
}

impl IntoDiscordRequest for GetGuildCommandPermissions {
	type Output = Vec<GuildCommandPermissions>;

	fn into_discord_request(self) -> Result<DiscordRequest, JsonError> {
		let path = format!(
			"applications/{}/guilds/{}/commands/permissions",
			self.application_id, self.guild_id
		);
		let route_key = format!(
			"GET /applications/{}/guilds/{}/commands/permissions",
			self.application_id, self.guild_id
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
	) -> Result<Vec<GuildCommandPermissions>, JsonError> {
		parse_json(bytes)
	}
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
	use super::*;

	fn app_id() -> Id<ApplicationMarker> { Id::new(100) }
	fn guild_id() -> Id<GuildMarker> { Id::new(200) }
	fn cmd_id() -> Id<CommandMarker> { Id::new(300) }
	fn msg_id() -> Id<MessageMarker> { Id::new(400) }

	// ---- Interaction Responses ----

	#[test]
	fn get_original_interaction_response_into_request() {
		let req = GetOriginalInteractionResponse::new(app_id(), "tok123")
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "webhooks/100/tok123/messages/@original");
		assert_eq!(
			req.route_key,
			"GET /webhooks/interaction/messages/@original"
		);
		assert!(matches!(req.method, HttpMethod::Get));
		assert!(matches!(req.body, RequestBody::None));
	}

	#[test]
	fn delete_original_interaction_response_into_request() {
		let req = DeleteOriginalInteractionResponse::new(app_id(), "tok123")
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "webhooks/100/tok123/messages/@original");
		assert_eq!(
			req.route_key,
			"DELETE /webhooks/interaction/messages/@original"
		);
		assert!(matches!(req.method, HttpMethod::Delete));
		assert!(matches!(req.body, RequestBody::None));
	}

	// ---- Followups ----

	#[test]
	fn create_followup_into_request() {
		let req = CreateFollowup::new(app_id(), "tok123")
			.content("hi")
			.flags(64)
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "webhooks/100/tok123");
		assert_eq!(req.route_key, "POST /webhooks/interaction");
		assert!(matches!(req.method, HttpMethod::Post));
		match &req.body {
			RequestBody::Json(v) => {
				assert_eq!(v["content"], "hi");
				assert_eq!(v["flags"], 64);
			}
			_ => panic!("expected Json body"),
		}
	}

	#[test]
	fn create_followup_skips_none_fields() {
		let req = CreateFollowup::new(app_id(), "tok123")
			.into_discord_request()
			.unwrap();
		match &req.body {
			RequestBody::Json(v) => {
				assert!(v.get("content").is_none());
				assert!(v.get("embeds").is_none());
				assert!(v.get("components").is_none());
				assert!(v.get("flags").is_none());
			}
			_ => panic!("expected Json body"),
		}
	}

	#[test]
	fn get_followup_into_request() {
		let req = GetFollowup::new(app_id(), "tok123", msg_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "webhooks/100/tok123/messages/400");
		assert_eq!(req.route_key, "GET /webhooks/interaction/messages");
		assert!(matches!(req.method, HttpMethod::Get));
		assert!(matches!(req.body, RequestBody::None));
	}

	#[test]
	fn update_followup_into_request() {
		let req = UpdateFollowup::new(app_id(), "tok123", msg_id())
			.content("edited")
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "webhooks/100/tok123/messages/400");
		assert_eq!(req.route_key, "PATCH /webhooks/interaction/messages");
		assert!(matches!(req.method, HttpMethod::Patch));
		match &req.body {
			RequestBody::Json(v) => {
				assert_eq!(v["content"], "edited");
			}
			_ => panic!("expected Json body"),
		}
	}

	#[test]
	fn delete_followup_into_request() {
		let req = DeleteFollowup::new(app_id(), "tok123", msg_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "webhooks/100/tok123/messages/400");
		assert_eq!(req.route_key, "DELETE /webhooks/interaction/messages");
		assert!(matches!(req.method, HttpMethod::Delete));
		assert!(matches!(req.body, RequestBody::None));
	}

	// ---- Global Commands ----

	#[test]
	fn get_global_commands_into_request() {
		let req = GetGlobalCommands::new(app_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "applications/100/commands");
		assert_eq!(req.route_key, "GET /applications/100/commands");
		assert!(matches!(req.method, HttpMethod::Get));
		assert!(matches!(req.body, RequestBody::None));
	}

	#[test]
	fn get_global_command_into_request() {
		let req = GetGlobalCommand::new(app_id(), cmd_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "applications/100/commands/300");
		assert_eq!(req.route_key, "GET /applications/100/commands");
		assert!(matches!(req.method, HttpMethod::Get));
	}

	#[test]
	fn delete_global_command_into_request() {
		let req = DeleteGlobalCommand::new(app_id(), cmd_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "applications/100/commands/300");
		assert_eq!(req.route_key, "DELETE /applications/100/commands");
		assert!(matches!(req.method, HttpMethod::Delete));
		assert!(matches!(req.body, RequestBody::None));
	}

	// ---- Guild Commands ----

	#[test]
	fn get_guild_commands_into_request() {
		let req = GetGuildCommands::new(app_id(), guild_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "applications/100/guilds/200/commands");
		assert_eq!(req.route_key, "GET /applications/100/guilds/200/commands");
		assert!(matches!(req.method, HttpMethod::Get));
	}

	#[test]
	fn get_guild_command_into_request() {
		let req = GetGuildCommand::new(app_id(), guild_id(), cmd_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "applications/100/guilds/200/commands/300");
		assert_eq!(req.route_key, "GET /applications/100/guilds/200/commands");
	}

	#[test]
	fn delete_guild_command_into_request() {
		let req = DeleteGuildCommand::new(app_id(), guild_id(), cmd_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(req.path, "applications/100/guilds/200/commands/300");
		assert_eq!(
			req.route_key,
			"DELETE /applications/100/guilds/200/commands"
		);
		assert!(matches!(req.method, HttpMethod::Delete));
		assert!(matches!(req.body, RequestBody::None));
	}

	// ---- Command Permissions ----

	#[test]
	fn get_command_permissions_into_request() {
		let req = GetCommandPermissions::new(app_id(), guild_id(), cmd_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(
			req.path,
			"applications/100/guilds/200/commands/300/permissions"
		);
		assert_eq!(
			req.route_key,
			"GET /applications/100/guilds/200/commands/permissions"
		);
		assert!(matches!(req.method, HttpMethod::Get));
		assert!(matches!(req.body, RequestBody::None));
	}

	#[test]
	fn get_guild_command_permissions_into_request() {
		let req = GetGuildCommandPermissions::new(app_id(), guild_id())
			.into_discord_request()
			.unwrap();
		assert_eq!(
			req.path,
			"applications/100/guilds/200/commands/permissions"
		);
		assert_eq!(
			req.route_key,
			"GET /applications/100/guilds/200/commands/permissions"
		);
		assert!(matches!(req.method, HttpMethod::Get));
		assert!(matches!(req.body, RequestBody::None));
	}
}
